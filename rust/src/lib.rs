mod slack_http_client;
mod slack_url;
mod utils;

use js_sys::{JsString, Promise, JSON};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_wasm_bindgen::Serializer;
use slack_http_client::{
    get_api_base, SlackHttpClient, SlackHttpClientConfig, SlackHttpClientError,
};
use slack_url::SlackUrl;
use std::{
    collections::HashMap,
    fmt::format,
    path::{self, Path},
    str::FromStr,
};
use utils::set_panic_hook;
use wasm_bindgen::{convert::FromWasmAbi, prelude::*, JsCast};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
static ATTACHMENT_FOLDER_CONFIG_KEY: &str = "attachmentFolderPath";

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

#[wasm_bindgen()]
extern "C" {
    fn alert(message: &str);
}

#[wasm_bindgen(module = "index")]
extern "C" {
    fn combine_result(timestamp_result: &JsValue, threaded_timestamp_result: &JsValue) -> JsValue;
}

#[wasm_bindgen(module = "obsidian")]
extern "C" {
    fn request(request: JsValue) -> Promise;

    type Notice;

    #[wasm_bindgen(constructor)]
    fn new_with_timeout(message: &str, timeout_in_ms: u32) -> Notice;

    #[wasm_bindgen(constructor)]
    fn new(message: &str) -> Notice;

    pub type Vault;

    #[wasm_bindgen(method)]
    fn getConfig(this: &Vault, key: &str) -> String;

    #[wasm_bindgen(method)]
    fn create(this: &Vault, path: &str, content: JsString) -> Promise;
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
pub async fn get_slack_message(api_token: String, cookie: String, url: String, vault: Vault) {
    let log_prefix = "rust|get_slack_message";

    log::info!("{}|create slack client", log_prefix);
    let client = SlackHttpClient::<Promise>::new(
        SlackHttpClientConfig::new(get_api_base(), api_token, cookie),
        make_request,
    );

    let slack_url = SlackUrl::new(&url);
    log::info!("{}|parsed url|slack_url={:#?}", log_prefix, &slack_url);

    let ts_result = match wasm_bindgen_futures::JsFuture::from(
        client.get_conversation_replies_using_ts(&slack_url),
    )
    .await
    {
        Ok(result) => result,
        Err(err) => err,
    };
    log::info!("{}|ts_result={:#?}", log_prefix, &ts_result);

    let thread_ts_option = match client.get_conversation_replies_using_thread_ts(&slack_url) {
        Ok(promise) => match wasm_bindgen_futures::JsFuture::from(promise).await {
            Ok(result) => Some(result),
            Err(err) => Some(err),
        },
        Err(SlackHttpClientError::ThreadTsWasEmpty) => None,
    };
    log::info!("{}|thread_ts={:#?}", log_prefix, &thread_ts_option);

    let combined_result = combine_result(&ts_result, &ts_result);

    log::info!("{}|create attachments file", log_prefix);
    let attachments_folder = vault.getConfig(ATTACHMENT_FOLDER_CONFIG_KEY);
    let new_file_path = Path::new(&attachments_folder)
        .join(
            vec![
                slack_url.channel_id,
                slack_url.ts,
                slack_url.thread_ts.unwrap_or_default(),
            ]
            .join("-"),
        )
        .with_extension("json");

    match new_file_path.to_str() {
        Some(path) => match wasm_bindgen_futures::JsFuture::from(vault.create(path, JSON::stringify_with_replacer_and_space(&combined_result, &JsValue::UNDEFINED, &JsValue::from_f64(2.0)).expect("There was a problem creating the file for the retrieved slack messages. Unable to stringify combined result of downloaded slack message"))).await {
            Ok(_) => {
                Notice::new_with_timeout("Successfully downloaded slack message and save to attachment file. Attachment file name saved to clipboard", 5000);
                // todo save to clipboard
            }
            Err(err) => alert(&format!("There was a problem creating the file for the retrieved slack messages. Error creating the file for the downloaded messages. Error: {:#?}", err))
        },
        None => alert("There was a problem creating the file for the retrieved slack messages. Error creating new file path for downloaded messages")
    }
}

fn make_request(params: RequestUrlParam) -> Promise {
    let serializer = Serializer::json_compatible();
    request(params.serialize(&serializer).unwrap())
}
