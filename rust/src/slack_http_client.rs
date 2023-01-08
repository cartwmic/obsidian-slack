use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, collections::HashMap, str::FromStr};
use url::Url;

use crate::errors::{SlackError, SlackHttpClientError};

pub fn get_api_base() -> Url {
    Url::from_str("https://slack.com/api").unwrap()
}

#[derive(Debug, Serialize, Deserialize)]
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

pub trait SlackResponseValidator {
    fn ok(&self) -> Option<bool>;

    fn validate_response(self) -> Result<Self, SlackError>
    where
        Self: Sized,
        Self: std::fmt::Debug,
    {
        if self.ok().unwrap() {
            Ok(self)
        } else {
            log::info!("{:#?}", self);
            Err(SlackError::ResponseNotOk(format!("{:#?}", self)))
        }
    }
}

#[derive(Builder, Debug, Clone)]
pub struct SlackHttpClientConfig {
    api_base: url::Url,
    token: String,
    cookie: String,
    feature_flags: SlackHttpClientConfigFeatureFlags,
}

#[derive(Debug, Serialize, Deserialize, Builder, Clone)]
pub struct SlackHttpClientConfigFeatureFlags {
    pub get_users: bool,
    pub get_reactions: bool,
    pub get_channel_info: bool,
    pub get_attachments: bool,
    pub get_team_info: bool,
}

impl SlackHttpClientConfig {
    pub fn new(
        api_base: Url,
        token: String,
        cookie: String,
        feature_flags: SlackHttpClientConfigFeatureFlags,
    ) -> Result<SlackHttpClientConfig, SlackError> {
        let log_prefix = "rust|SlackHttpClientConfig|new";
        log::info!(
            "{}|api_base={}|token={}|cookie={}",
            &log_prefix,
            api_base,
            &token,
            &cookie
        );

        log::info!("{}|validate token", &log_prefix);
        let token = validate_slack_api_token(token.as_str());
        let cookie = validate_slack_api_cookie(cookie.as_str());

        match (token, cookie) {
            (Ok(the_token), Ok(the_cookie)) => Ok(SlackHttpClientConfig {
                api_base,
                token: the_token.to_string(),
                cookie: the_cookie.to_string(),
                feature_flags,
            }),
            (Err(a), Err(_)) => Err(a),
            (Err(a), Ok(_)) => Err(a),
            (Ok(_), Err(b)) => Err(b),
        }
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
    config: SlackHttpClientConfig,
    request_func: Box<dyn Fn(RequestUrlParam) -> ClientReturnType>,
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

        request_url.path_segments_mut().unwrap().push(endpoint);

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

    pub fn get_conversation_replies(&self, channel_id: &str, timestamp: &str) -> ClientReturnType {
        let log_prefix = "rust|get_conversation_replies";
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

    pub fn get_user_info(&self, user_id: &str) -> ClientReturnType {
        let log_prefix = "rust|get_user_info";
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
}

fn validate_slack_api_token(api_token: &str) -> Result<&str, SlackError> {
    if !api_token.starts_with("xoxc") {
        Err(SlackError::SlackHttpClient(
            SlackHttpClientError::InvalidApiToken("Did not start with 'xoxc'".to_string()),
        ))
    } else {
        Ok(api_token)
    }
}

fn validate_slack_api_cookie(cookie: &str) -> Result<&str, SlackError> {
    if !cookie.starts_with("xoxd") {
        Err(SlackError::SlackHttpClient(
            SlackHttpClientError::InvalidApiCookie("Did not start with 'xoxd'".to_string()),
        ))
    } else {
        Ok(cookie)
    }
}
