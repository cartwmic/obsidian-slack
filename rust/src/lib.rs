mod utils;

use std::{str::FromStr, path};
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

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
    channel_id: String,
    ts: String,
    thread_ts: String,
    url: reqwest::Url
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

        let mut path_segments = match url.path_segments() {
            Some(segments) => segments,
            None => panic!("{}|unable to parse path segments for slack url|url={}", log_prefix, url)
        };
        // channel id can be prefixed with 'C', 'D', or 'G'. See https://api.slack.com/docs/conversations-api#shared_channels
        let channel_id = path_segments
            .find(|segment| segment.starts_with('C') || segment.starts_with('D') || segment.starts_with('G'))
            .unwrap_or_else(|| panic!("{}|No channel id found in url|url={}", log_prefix, url))
            .to_string();
        
        let res = SlackUrl { 
            channel_id,
            ts: "".to_string(),
            thread_ts: "".to_string(),
            url
        };

        log::info!("{}|slack url={:#?}", log_prefix, res);

        res
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