use amplify_derive::Display;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use snafu::{ensure, Snafu};
use std::{borrow::Borrow, collections::HashMap, fmt::Debug, str::FromStr};
use url::Url;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display(
        "Provided api token was invalid. Api token must start with 'xoxc': {api_token}"
    ))]
    InvalidSlackApiToken { api_token: String },

    #[snafu(display("Provided api cookie was invalid. Cookie must start with 'xoxd': {cookie}"))]
    InvalidSlackApiCookie { cookie: String },
}
type Result<T, E = Error> = std::result::Result<T, E>;

pub fn get_api_base() -> Url {
    Url::from_str("https://slack.com/api").unwrap()
}

fn validate_slack_api_token(api_token: &str) -> Result<&str> {
    ensure!(
        api_token.starts_with("xoxc"),
        InvalidSlackApiTokenSnafu { api_token }
    );
    Ok(api_token)
}

fn validate_slack_api_cookie(cookie: &str) -> Result<&str> {
    ensure!(
        cookie.starts_with("xoxd"),
        InvalidSlackApiCookieSnafu { cookie }
    );
    Ok(cookie)
}

#[derive(Debug, Serialize, Deserialize, Display)]
#[display(Debug)]
pub struct RequestUrlParam {
    url: String,
    method: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl RequestUrlParam {
    fn with_url(mut self, url: String) -> Self {
        self.url = url;
        self
    }
}

#[derive(Builder, Debug, Clone)]
pub struct SlackHttpClientConfig {
    api_base: url::Url,
    token: String,
    cookie: String,
    pub feature_flags: SlackHttpClientConfigFeatureFlags,
}

#[derive(Debug, Serialize, Deserialize, Builder, Clone, Display)]
#[display(Debug)]
pub struct SlackHttpClientConfigFeatureFlags {
    pub get_users: bool,
    pub get_channel_info: bool,
    pub get_team_info: bool,
    pub get_file_data: bool,
}

impl SlackHttpClientConfig {
    pub fn new(
        api_base: Url,
        token: String,
        cookie: String,
        feature_flags: SlackHttpClientConfigFeatureFlags,
    ) -> Result<SlackHttpClientConfig> {
        let log_prefix = "rust|SlackHttpClientConfig|new";
        log::info!(
            "{}|api_base={}|token={}|cookie={}",
            &log_prefix,
            api_base,
            &token,
            &cookie
        );

        log::info!("{}|validate token", &log_prefix);
        let token = validate_slack_api_token(token.as_str())?;

        log::info!("{}|validate cookie", &log_prefix);
        let cookie = validate_slack_api_cookie(cookie.as_str())?;

        Ok(SlackHttpClientConfig {
            api_base,
            token: token.to_string(),
            cookie: cookie.to_string(),
            feature_flags,
        })
    }
}

#[allow(non_camel_case_types)]
#[derive(strum_macros::Display)]
pub enum SlackApiQueryParams {
    ts,
    thread_ts,
    channel,
    inclusive,
    pretty,
}

pub struct SlackHttpClient<ClientReturnType> {
    pub config: SlackHttpClientConfig,
    request_func: Box<dyn Fn(RequestUrlParam) -> ClientReturnType>,
}

impl<ClientReturnType> Debug for SlackHttpClient<ClientReturnType> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SlackHttpClient")
            .field("config", &self.config)
            .finish()
    }
}

impl<ClientReturnType> SlackHttpClient<ClientReturnType> {
    pub fn new(
        config: SlackHttpClientConfig,
        request_func: Box<dyn Fn(RequestUrlParam) -> ClientReturnType>,
    ) -> SlackHttpClient<ClientReturnType> {
        SlackHttpClient {
            config,
            request_func,
        }
    }

    fn build_request_uri<I, K, V>(&self, endpoint: &str, iter: I) -> Url
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let mut request_url = self.config.api_base.clone();

        request_url
            .path_segments_mut()
            .expect("Expected a url that can be a base and has path segments, but did not get that. This is a bug")
            .push(endpoint);

        request_url
            .query_pairs_mut()
            .extend_pairs(iter)
            .append_pair(SlackApiQueryParams::pretty.to_string().as_str(), "1");

        request_url
    }

    fn build_base_post_request(&self) -> RequestUrlParam {
        RequestUrlParam {
            url: "".to_string(),
            method: "POST".to_string(),
            headers: HashMap::from([
                (
                    "content-type".to_string(),
                    "application/x-www-form-urlencoded".to_string(),
                ),
                ("cookie".to_string(), "d=".to_string() + &self.config.cookie),
            ]),
            body: Some(format!("token={}", self.config.token)),
        }
    }

    fn build_base_get_request(&self) -> RequestUrlParam {
        RequestUrlParam {
            url: "".to_string(),
            method: "GET".to_string(),
            headers: HashMap::from([
                (
                    "content-type".to_string(),
                    "application/x-www-form-urlencoded".to_string(),
                ),
                ("cookie".to_string(), "d=".to_string() + &self.config.cookie),
                (
                    "authorization".to_string(),
                    format!("Bearer {}", self.config.token),
                ),
            ]),
            body: None,
        }
    }

    pub fn get_file_data(&self, file_url: &str) -> ClientReturnType {
        let log_prefix = "rust|get_file_data";
        log::info!("{log_prefix}|file_url={file_url}");

        log::info!("{}|build request object", &log_prefix);
        let the_request = self.build_base_get_request().with_url(file_url.to_string());

        log::info!("{}|submit request|request={:#?}", &log_prefix, the_request);
        (self.request_func)(the_request)
    }

    pub fn get_conversations_replies(&self, channel_id: &str, timestamp: &str) -> ClientReturnType {
        let log_prefix = "rust|get_conversations_replies";
        log::info!(
            "{}|channel_id={}|timestamp={}",
            &log_prefix,
            channel_id,
            timestamp
        );

        log::info!("{}|build request url", &log_prefix);
        let request_url = self.build_request_uri(
            "conversations.replies",
            vec![
                (SlackApiQueryParams::channel.to_string(), channel_id),
                (SlackApiQueryParams::ts.to_string(), timestamp),
                (SlackApiQueryParams::inclusive.to_string(), "true"),
            ],
        );

        log::info!("{}|build request object", &log_prefix);
        let the_request = self
            .build_base_post_request()
            .with_url(request_url.to_string());

        log::info!("{}|submit request|request={:#?}", &log_prefix, the_request);
        (self.request_func)(the_request)
    }

    pub fn get_users_info(&self, user_id: &str) -> ClientReturnType {
        let log_prefix = "rust|get_users_info";
        log::info!("{}|user_id={}", &log_prefix, user_id);

        log::info!("{}|build request url", &log_prefix);
        let request_url = self.build_request_uri("users.info", vec![("user", user_id)]);

        log::info!("{}|build request object", &log_prefix);
        let the_request = self
            .build_base_get_request()
            .with_url(request_url.to_string());

        log::info!("{}|submit request|request={:#?}", &log_prefix, the_request);
        (self.request_func)(the_request)
    }

    pub fn get_conversations_info(&self, channel_id: &str) -> ClientReturnType {
        let log_prefix = "rust|get_conversations_info";
        log::info!("{}|channel_id={}", &log_prefix, channel_id);

        log::info!("{}|build request url", &log_prefix);
        let request_url =
            self.build_request_uri("conversations.info", vec![("channel", channel_id)]);

        log::info!("{}|build request object", &log_prefix);
        let the_request = self
            .build_base_get_request()
            .with_url(request_url.to_string());

        log::info!("{}|submit request|request={:#?}", &log_prefix, the_request);
        (self.request_func)(the_request)
    }

    pub fn get_team_info(&self, team_id: &str) -> ClientReturnType {
        let log_prefix = "rust|get_team_info";
        log::info!("{}|team_id={}", &log_prefix, team_id);

        log::info!("{}|build request url", &log_prefix);
        let request_url = self.build_request_uri("team.info", vec![("team", team_id)]);

        log::info!("{}|build request object", &log_prefix);
        let the_request = self
            .build_base_get_request()
            .with_url(request_url.to_string());

        log::info!("{}|submit request|request={:#?}", &log_prefix, the_request);
        (self.request_func)(the_request)
    }
}
