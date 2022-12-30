use std::{collections::HashMap, str::FromStr};
use url::Url;

use crate::{
    errors::{SlackError, SlackHttpClientError},
    slack_url::SlackUrl,
    RequestUrlParam,
};

pub fn get_api_base() -> Url {
    Url::from_str("https://slack.com/api/").unwrap()
}

pub struct SlackHttpClientConfig {
    api_base: url::Url,
    token: String,
    cookie: String,
}

impl SlackHttpClientConfig {
    pub fn new(
        api_base: Url,
        token: String,
        cookie: String,
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
            }),
            (Err(a), Err(b)) => Err(a),
            (Err(a), Ok(_)) => Err(a),
            (Ok(_), Err(b)) => Err(b),
        }
    }
}

#[derive(strum_macros::Display)]
enum SlackApiQueryParams {
    ts,
    thread_ts,
    channel,
    inclusive,
    pretty,
}

pub struct SlackHttpClient<ClientReturnType> {
    config: SlackHttpClientConfig,
    request_func: fn(RequestUrlParam) -> ClientReturnType,
}

impl<ClientReturnType> SlackHttpClient<ClientReturnType> {
    pub fn new(
        config: SlackHttpClientConfig,
        request_func: fn(RequestUrlParam) -> ClientReturnType,
    ) -> SlackHttpClient<ClientReturnType> {
        SlackHttpClient {
            config,
            request_func,
        }
    }

    fn build_request_uri(&self, endpoint: &str, channel_id: &str, ts: &str) -> Url {
        let mut request_url = self.config.api_base.clone();

        request_url.path_segments_mut().unwrap().push(endpoint);

        request_url
            .query_pairs_mut()
            .append_pair(
                SlackApiQueryParams::channel.to_string().as_str(),
                channel_id,
            )
            .append_pair(SlackApiQueryParams::ts.to_string().as_str(), ts)
            .append_pair(SlackApiQueryParams::pretty.to_string().as_str(), "1")
            .append_pair(SlackApiQueryParams::inclusive.to_string().as_str(), "true");

        request_url
    }

    fn build_base_request(&self) -> RequestUrlParam {
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
            body: format!("token={}", self.config.token),
        }
    }

    pub fn get_conversation_replies_using_thread_ts(
        &self,
        slack_url: &SlackUrl,
    ) -> Option<ClientReturnType> {
        slack_url
            .thread_ts
            .as_ref()
            .map(|ts| self.get_conversation_replies(&slack_url.channel_id, ts))
    }

    pub fn get_conversation_replies_using_ts(&self, slack_url: &SlackUrl) -> ClientReturnType {
        self.get_conversation_replies(&slack_url.channel_id, &slack_url.ts)
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
        let request_url = self.build_request_uri("conversations.replies", channel_id, timestamp);

        log::info!("{}|build request object", &log_prefix);
        let the_request = self.build_base_request().with_url(request_url.to_string());

        log::info!("{}|submit request|request={:#?}", &log_prefix, the_request);
        (self.request_func)(the_request)
    }
}

fn validate_slack_api_token(api_token: &str) -> Result<&str, SlackError> {
    if !api_token.starts_with("xoxc") {
        Err(SlackError::SlackHttpClientError(
            SlackHttpClientError::InvalidApiToken("Did not start with 'xoxc'".to_string()),
        ))
    } else {
        Ok(api_token)
    }
}

fn validate_slack_api_cookie(cookie: &str) -> Result<&str, SlackError> {
    let log_prefix = "rust|validate_slack_api_cookie";
    if !cookie.starts_with("xoxd") {
        Err(SlackError::SlackHttpClientError(
            SlackHttpClientError::InvalidApiCookie("Did not start with 'xoxd'".to_string()),
        ))
    } else {
        Ok(cookie)
    }
}
