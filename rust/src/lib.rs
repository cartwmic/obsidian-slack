mod utils;

use std::{str::{FromStr, Split}, path};
use tuple_conv::RepeatedTuple;
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const THREAD_TS_KEY: &str = "thread_ts";

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn init_wasm(log_level: Option<String>) {
    set_panic_hook();

    let level_string = match log_level {
        Some(level) => level,
        None => log::Level::Info.to_string()
    };

    let level = match log::Level::from_str(&level_string) {
        Ok(level) => level,
        Err(err) => {
            panic!("rust|init| unable to prase provided log level|err={}", err)
        }
    };

    match console_log::init_with_level(level) {
        Ok(_) => (),
        Err(err) => panic!("rust|init| unable to init with level|err={}", err)
    };
}

#[wasm_bindgen]
pub fn get_slack_message(api_token: String, url: String) {
    alert(&SlackHttpClient::get_slack_message(&api_token, &url));
}

struct SlackToken {
    token_string: String,
    pub token: String
}


struct SlackHttpClientConfig {
    api_base: String,
    token: SlackToken,
}
#[derive(Debug)]
struct SlackUrl {
    pub channel_id: String,
    pub ts: String,
    pub thread_ts: String,
    url: reqwest::Url,
    path_segments: Vec<String>
}

impl SlackUrl {
    fn new(url_string: &str) -> Self{
        let log_prefix = "rust|SlackUrl|new";

        let url = match reqwest::Url::parse(url_string) {
            Ok(the_url) => the_url,
            Err(err) => {
                panic!("{}|unable to parse the url string into a reqwest|url_string={}|err={}", log_prefix, url_string, err)
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
            path_segments
        };

        log::info!("{}|slack url={:#?}", log_prefix, res);
        res
    }

    fn parse_path_segments(url: &reqwest::Url) -> Vec<String> {
        let log_prefix = "rust|SlackUrl|parse_path_segments";

        match url.path_segments() {
            Some(segments) => segments,
            None => panic!("{}|unable to parse path segments for slack url|url={}", log_prefix, url)
        }
        .collect::<Vec<&str>>().into_iter().map(String::from).collect::<Vec<String>>()
    }

    fn parse_channel_id(url: &reqwest::Url, path_segments: &Vec<String>) -> String {
        let log_prefix = "rust|SlackUrl|parse_channel_id";

        // channel id can be prefixed with 'C', 'D', or 'G'. See https://api.slack.com/docs/conversations-api#shared_channels
        path_segments
            .iter()
            .find(|segment| segment.starts_with('C') || segment.starts_with('D') || segment.starts_with('G'))
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
    pub slack_http_client_config: SlackHttpClientConfig
}

impl SlackHttpClient {
    pub fn get_slack_message(api_token: &str, url: &str) -> String {
        let log_prefix = "rust|get_slack_message";
        log::info!("{}|api_token={}|url={}", &log_prefix, api_token, url);

        log::info!("{}|parse url", &log_prefix);
        let slack_url = SlackUrl::new(url);

        log::info!("{}|validate token", &log_prefix);

        log::info!("{}|get slack message", &log_prefix);

        let result = "SlackHttpClient:".to_owned() + api_token + ":" + url;

        log::info!("{}|result={}", &log_prefix, result);
        result
    }
}