mod utils;

use std::{str::FromStr, panic::panic_any};

use utils::set_panic_hook;
use wasm_bindgen::prelude::*;
use log::{trace, info, warn};
use console_log;

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
            log::error!("{}", err);
            panic!("rust::init:: unable to prase provided log level::err={}", err)
        }
    };

    match console_log::init_with_level(level) {
        Ok(_) => (),
        Err(err) => panic!("rust::init:: unable to init with level::err={}", err)
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

 struct SlackMessageLinkParser;

struct SlackHttpClient {
    pub slack_http_client_config: SlackHttpClientConfig
}

impl SlackHttpClient {
    pub fn get_slack_message(api_token: &str, url: &str) -> String {
        let log_prefix = "rust::get_slack_message";

        log::info!("{}::api_token={}::url={}", &log_prefix, api_token, url);

        log::info!("{}::parse url", &log_prefix);

        log::info!("{}::validate token", &log_prefix);

        log::info!("{}::get slack message", &log_prefix);

        let result = "SlackHttpClient:".to_owned() + api_token + ":" + url;

        log::info!("{}::result={}", &log_prefix, result);
        result
    }
}