mod utils;

use js_sys::Promise;
use reqwest::Response;
use std::{
    collections::HashMap,
    fmt::Error,
    future::Future,
    path,
    str::{FromStr, Split},
};
use tuple_conv::RepeatedTuple;
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const THREAD_TS_KEY: &str = "thread_ts";
const API_BASE: &str = "https://slack.com/api/";

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen(module = "obsidian")]
extern "C" {
    fn request(request: js_sys::Object) -> Promise;
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
    if !cookie.starts_with("d=xoxd") {
        panic!(
            "{}|api cookie does not start with 'd=xoxd'. api token invalid|cookie={}",
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
    api_base: reqwest::Url,
    token: String,
    cookie: String,
}
#[derive(Debug)]
struct SlackUrl {
    pub channel_id: String,
    pub ts: String,
    pub thread_ts: String,
    url: reqwest::Url,
    path_segments: Vec<String>,
}

impl SlackUrl {
    fn new(url_string: &str) -> Self {
        let log_prefix = "rust|SlackUrl|new";

        let url = match reqwest::Url::parse(url_string) {
            Ok(the_url) => the_url,
            Err(err) => {
                panic!(
                    "{}|unable to parse the url string into a reqwest|url_string={}|err={}",
                    log_prefix, url_string, err
                )
            }
        };

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

    fn parse_path_segments(url: &reqwest::Url) -> Vec<String> {
        let log_prefix = "rust|SlackUrl|parse_path_segments";

        match url.path_segments() {
            Some(segments) => segments,
            None => panic!(
                "{}|unable to parse path segments for slack url|url={}",
                log_prefix, url
            ),
        }
        .collect::<Vec<&str>>()
        .into_iter()
        .map(String::from)
        .collect::<Vec<String>>()
    }

    fn parse_channel_id(url: &reqwest::Url, path_segments: &Vec<String>) -> String {
        let log_prefix = "rust|SlackUrl|parse_channel_id";

        // channel id can be prefixed with 'C', 'D', or 'G'. See https://api.slack.com/docs/conversations-api#shared_channels
        path_segments
            .iter()
            .find(|segment| {
                segment.starts_with('C') || segment.starts_with('D') || segment.starts_with('G')
            })
            .unwrap_or_else(|| panic!("{}|No channel id found in url|url={}", log_prefix, url))
            .to_string()
    }

    fn parse_ts(url: &reqwest::Url, path_segments: &Vec<String>) -> String {
        let log_prefix = "rust|SlackUrl|parse_ts";

        path_segments
            .iter()
            .find(|segment| segment.starts_with('p'))
            .unwrap_or_else(|| panic!("{}|No ts found in url|url={}", log_prefix, url))
            .split_terminator('p')
            .last()
            .unwrap_or_else(|| panic!("{}|ts value is malformed in url|url={}", log_prefix, url))
            .split_at(10)
            .to_vec()
            .join(".")
    }

    fn parse_thread_ts(url: &reqwest::Url) -> Option<String> {
        url.query_pairs()
            .find(|(key, _)| key == THREAD_TS_KEY)
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
                api_base: reqwest::Url::parse(API_BASE).unwrap_or_else(|err| {
                    panic!(
                        "{}|Unable to parse base url for api|API_BASE={}|err={}",
                        log_prefix, API_BASE, err
                    )
                }),
                token: api_token.to_owned(),
                cookie: cookie.to_owned(),
            },
        }
    }

    pub async fn get_conversation_replies(&self, url: &str) -> Result<JsValue, JsValue> {
        // let log_prefix = "rust|get_slack_message";
        // log::info!("{}|url={}", &log_prefix, url);

        // log::info!("{}|parse url", &log_prefix);
        // let slack_url = SlackUrl::new(url);

        // log::info!("{}|get slack message", &log_prefix);
        // let client = reqwest::Client::new();
        // let mut request_url = self
        //     .slack_http_client_config
        //     .api_base
        //     .join("conversations.replies")
        //     .unwrap_or_else(|err| panic!("{}|unable to parse request url", log_prefix));
        // request_url.set_query(Some(
        //     format!(
        //         "{}={}&{}={}&{}={}&{}={}",
        //         SlackApiQueryParams::channel,
        //         slack_url.channel_id.as_str(),
        //         SlackApiQueryParams::ts,
        //         slack_url.ts,
        //         SlackApiQueryParams::pretty,
        //         "1",
        //         SlackApiQueryParams::inclusive,
        //         "true"
        //     )
        //     .as_str(),
        // ));

        let the_request = js_sys::Object::new();
        let headers = js_sys::Object::new();

        js_sys::Reflect::set(
            &headers,
            &JsValue::from("content-type"),
            &JsValue::from("application/x-www-form-urlencoded"),
        );
        js_sys::Reflect::set(
            &headers,
            &JsValue::from("cookie"),
            &JsValue::from("d=xoxd-NImkt4e5%2FBZJ8cm8bPd9JWAZ5ATSvnUwE%2FHGRV4E%2FyCdFSbaclP0Xw6p0MwCij7dVH0sG9oLVrO8uVW9DOUP2AmGituX8NwJgd8iVSOnjWCqR%2F%2Fx0KraMm%2FYuBZCJWfVDKxA8df9Yz6OX5XB2qPXA0c9F1DvLbYDZP7btXloR8RdQoEIb5dBdQ%3D%3D;"),
        );

        js_sys::Reflect::set(&the_request, &JsValue::from("url"), &JsValue::from("https://axon.slack.com/api/conversations.replies?channel=C01ENB4KP26&ts=1671055784.980429&pretty=1&inclusive=true"));
        js_sys::Reflect::set(
            &the_request,
            &JsValue::from("method"),
            &JsValue::from("POST"),
        );
        js_sys::Reflect::set(&the_request, &JsValue::from("headers"), &headers);
        js_sys::Reflect::set(&the_request, &JsValue::from("body"), &JsValue::from("token=xoxc-4684147883-3183999236788-4411640857313-b8215c23899763f5f3e048dedb3d8e2cdee8957a7f2eaafa7d81eccda9ca35d7"));

        let promise = request(the_request);
        wasm_bindgen_futures::JsFuture::from(promise).await

        // let body = HashMap::from([("token", self.slack_http_client_config.token.as_str())]);
        // let request = client
        //     .post(request_url)
        //     .form(&body)
        //     .header("cookie", self.slack_http_client_config.cookie.as_str())
        //     .header("content-length", "")
        //     .build()
        //     .unwrap();
        // log::info!(
        //     "{}|body={:#?}",
        //     log_prefix,
        //     std::str::from_utf8(request.body().expect("msg").as_bytes().expect("msg"))
        // );
        // client.execute(request).await
    }
}
