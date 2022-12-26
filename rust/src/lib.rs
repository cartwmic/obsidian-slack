mod utils;

use js_sys::Promise;
use log::info;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::Serializer;
use std::{collections::HashMap, path, str::FromStr};
use tuple_conv::RepeatedTuple;
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const THREAD_TS_KEY: &str = "thread_ts";
const API_BASE: &str = "https://slack.com/api/";

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestUrlParam {
    url: String,
    method: String,
    headers: HashMap<String, String>,
    body: String,
}

#[wasm_bindgen(module = "obsidian")]
extern "C" {
    fn request(request: JsValue) -> Promise;
}

#[wasm_bindgen]
pub fn init_wasm(log_level: Option<String>) {
    set_panic_hook();

    let level_string = match log_level {
        Some(level) => level,
        None => log::Level::Info.to_string(),
    };

    let level = match log::Level::from_str(&level_string) {
        Ok(level) => level,
        Err(err) => {
            panic!("rust|init| unable to prase provided log level|err={}", err)
        }
    };

    match console_log::init_with_level(level) {
        Ok(_) => (),
        Err(err) => panic!("rust|init| unable to init with level|err={}", err),
    };
}

#[wasm_bindgen]
pub async fn get_slack_message(api_token: String, cookie: String, url: String) -> JsValue {
    let client = SlackHttpClient::new(&api_token, &cookie);
    // spawn_local(async move {
    match client.get_conversation_replies(&url).await {
        Ok(resp) => resp,
        Err(err) => err,
    }
}

fn validate_slack_api_token(api_token: &str) {
    let log_prefix = "rust|validate_slack_api_token";
    if !api_token.starts_with("xoxc") {
        panic!(
            "{}|api token does not start with 'xoxc'. api token invalid|api_token={}",
            log_prefix, api_token
        )
    }
}

fn validate_slack_api_cookie(cookie: &str) {
    let log_prefix = "rust|validate_slack_api_cookie";
    if !cookie.starts_with("xoxd") {
        panic!(
            "{}|api cookie does not start with 'xoxd'. api token invalid|cookie={}",
            log_prefix, cookie
        )
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

struct SlackHttpClientConfig {
    api_base: url::Url,
    token: String,
    cookie: String,
}
#[derive(Debug)]
struct SlackUrl {
    pub channel_id: String,
    pub ts: String,
    pub thread_ts: String,
    url: url::Url,
    path_segments: Vec<String>,
}

impl SlackUrl {
    fn new(url_string: &str) -> Self {
        let log_prefix = "rust|SlackUrl|new";

        let url = url::Url::from_str(url_string).unwrap();

        let path_segments = SlackUrl::parse_path_segments(&url);
        let channel_id = SlackUrl::parse_channel_id(&url, &path_segments);
        let ts = SlackUrl::parse_ts(&url, &path_segments);
        let thread_ts = SlackUrl::parse_thread_ts(&url).unwrap_or_else(|| ts.clone());

        let res = SlackUrl {
            channel_id,
            ts,
            thread_ts,
            url,
            path_segments,
        };

        log::info!("{}|slack url={:#?}", log_prefix, res);
        res
    }

    fn parse_path_segments(url: &url::Url) -> Vec<String> {
        url.path_segments()
            .unwrap()
            .collect::<Vec<&str>>()
            .into_iter()
            .map(String::from)
            .collect::<Vec<String>>()
    }

    fn parse_channel_id(url: &url::Url, path_segments: &Vec<String>) -> String {
        // channel id can be prefixed with 'C', 'D', or 'G'. See https://api.slack.com/docs/conversations-api#shared_channels
        path_segments
            .iter()
            .find(|segment| {
                segment.starts_with('C') || segment.starts_with('D') || segment.starts_with('G')
            })
            .unwrap()
            .to_string()
    }

    fn parse_ts(url: &url::Url, path_segments: &Vec<String>) -> String {
        path_segments
            .iter()
            .find(|segment| segment.starts_with('p'))
            .unwrap()
            .split_terminator('p')
            .last()
            .unwrap()
            .split_at(10)
            .to_vec()
            .join(".")
    }

    fn parse_thread_ts(url: &url::Url) -> Option<String> {
        url.query_pairs()
            .find(|(key, _)| *key == THREAD_TS_KEY)
            .map(|(_, value)| value.to_string())
    }
}

struct SlackHttpClient {
    pub slack_http_client_config: SlackHttpClientConfig,
}

impl SlackHttpClient {
    pub fn new(api_token: &str, cookie: &str) -> SlackHttpClient {
        let log_prefix = "rust|new";
        log::info!("{}|validate token", &log_prefix);
        validate_slack_api_token(api_token);
        validate_slack_api_cookie(cookie);

        SlackHttpClient {
            slack_http_client_config: SlackHttpClientConfig {
                api_base: url::Url::from_str(API_BASE).unwrap(),
                token: api_token.to_owned(),
                cookie: cookie.to_owned(),
            },
        }
    }

    pub async fn get_conversation_replies(&self, url: &str) -> Result<JsValue, JsValue> {
        let log_prefix = "rust|get_slack_message";
        log::info!("{}|url={}", &log_prefix, url);

        log::info!("{}|parse url", &log_prefix);
        let slack_url = SlackUrl::new(url);

        log::info!("{}|get slack message", &log_prefix);
        let mut request_url = self.slack_http_client_config.api_base.clone();
        request_url
            .path_segments_mut()
            .unwrap()
            .push("conversations.replies");
        request_url
            .query_pairs_mut()
            .append_pair(
                SlackApiQueryParams::channel.to_string().as_str(),
                slack_url.channel_id.as_str(),
            )
            .append_pair(
                SlackApiQueryParams::ts.to_string().as_str(),
                slack_url.ts.as_str(),
            )
            .append_pair(SlackApiQueryParams::pretty.to_string().as_str(), "1")
            .append_pair(SlackApiQueryParams::inclusive.to_string().as_str(), "true");

        log::info!("{}", request_url);

        let the_request = RequestUrlParam {
            url: request_url.to_string(),
            method: "POST".to_string(),
            headers: HashMap::from([
                (
                    "content-type".to_string(),
                    "application/x-www-form-urlencoded".to_string(),
                ),
                (
                    "cookie".to_string(),
                    "d=".to_string() + &self.slack_http_client_config.cookie,
                ),
            ]),
            body: format!("token={}", self.slack_http_client_config.token),
        };
        let serializer = Serializer::json_compatible();
        log::info!("{:#?}", the_request.serialize(&serializer).unwrap());

        let promise = request(the_request.serialize(&serializer).unwrap());
        wasm_bindgen_futures::JsFuture::from(promise).await
    }
}
