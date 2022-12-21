mod utils;

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
pub fn get_slack_message(url: String) {
    alert(&SlackHttpClient::get_slack_message(url));
}

struct SlackToken {
    token_string: String,
    pub token: String
}


struct SlackHttpClientConfig {
    api_base: String,
    token: SlackToken,
}

struct SlackHttpClient {
    pub slack_http_client_config: SlackHttpClientConfig
}

impl SlackHttpClient {
    pub fn get_slack_message(url: String) -> String {
        url
    }
}