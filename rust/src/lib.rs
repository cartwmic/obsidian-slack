mod slack_http_client;
mod slack_url;
mod utils;

use js_sys::Promise;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::Serializer;
use slack_http_client::{
    get_api_base, SlackHttpClient, SlackHttpClientConfig, SlackHttpClientError,
};
use slack_url::SlackUrl;
use std::{collections::HashMap, str::FromStr};
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestUrlParam {
    url: String,
    method: String,
    headers: HashMap<String, String>,
    body: String,
}

impl RequestUrlParam {
    fn with_url(mut self, url: String) -> Self {
        self.url = url;
        self
    }
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
pub async fn get_slack_message(api_token: String, cookie: String, url: String) -> js_sys::Object {
    let client = SlackHttpClient::<Promise>::new(
        SlackHttpClientConfig::new(get_api_base(), api_token, cookie),
        make_request,
    );

    let slack_url = SlackUrl::new(&url);

    let ts_result = match wasm_bindgen_futures::JsFuture::from(
        client.get_conversation_replies_using_ts(&slack_url),
    )
    .await
    {
        Ok(result) => result,
        Err(err) => err,
    };

    let thread_ts_option = match client.get_conversation_replies_using_thread_ts(&slack_url) {
        Ok(promise) => match wasm_bindgen_futures::JsFuture::from(promise).await {
            Ok(result) => Some(result),
            Err(err) => Some(err),
        },
        Err(SlackHttpClientError::ThreadTsWasEmpty) => None,
    };

    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &JsValue::from_str("timestamp"), &ts_result);

    match thread_ts_option {
        Some(result) => js_sys::Reflect::set(&obj, &JsValue::from_str("thread_timestamp"), &result),
        None => js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("thread_timestamp"),
            &JsValue::null(),
        ),
    };

    obj
}

fn make_request(params: RequestUrlParam) -> Promise {
    let serializer = Serializer::json_compatible();
    request(params.serialize(&serializer).unwrap())
}
