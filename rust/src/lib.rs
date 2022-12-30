mod errors;
mod slack_http_client;
mod slack_url;
mod utils;

use js_sys::{JsString, Promise, JSON};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::Serializer;
use slack_http_client::{get_api_base, SlackHttpClient, SlackHttpClientConfig};
use slack_url::SlackUrl;
use std::{collections::HashMap, path::Path, str::FromStr};
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

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

    #[wasm_bindgen(js_namespace = ["navigator", "clipboard"])]
    fn writeText(data: &str) -> Promise;
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
    let results_from_api = match SlackHttpClientConfig::new(get_api_base(), api_token, cookie)
        .map(|config| SlackHttpClient::<Promise>::new(config, make_request))
        .and_then(|client| {
            let slack_url = SlackUrl::new(&url);
            slack_url.map(|url| (client, url))
        })
        .map(|(client, url)| {
            let ts_future = wasm_bindgen_futures::JsFuture::from(
                client.get_conversation_replies_using_ts(&url),
            );
            let thread_ts_option = client.get_conversation_replies_using_thread_ts(&url);
            (client, url, ts_future, thread_ts_option)
        }) {
        Ok((_, url, ts_future, thread_ts_option)) => {
            let ts_result = ts_future.await.map_err(errors::SlackError::JsError);
            let thread_ts = match thread_ts_option {
                Some(promise) => match wasm_bindgen_futures::JsFuture::from(promise).await {
                    Ok(val) => val,
                    Err(err) => err,
                },
                None => JsValue::NULL,
            };
            ts_result.map(|val| (combine_result(&val, &thread_ts), url))
        }
        Err(err) => Err(err),
    };

    let file_creation_result = match results_from_api.and_then(|(result, slack_url)| {
        let attachments_folder = vault.getConfig(ATTACHMENT_FOLDER_CONFIG_KEY);
        let file_name = vec![
            slack_url.channel_id,
            slack_url.ts,
            slack_url.thread_ts.unwrap_or_default(),
        ]
        .join("-")
            + ".json";
        let new_file_path = Path::new(&attachments_folder)
            .join(&file_name)
            .to_str()
            .unwrap()
            .to_string();

        JSON::stringify_with_replacer_and_space(&result, &JsValue::NULL, &JsValue::from_f64(2.0))
            .map_err(errors::SlackError::JsError)
            .map(|json_str| (json_str, file_name, new_file_path))
    }) {
        Ok((json_str, file_name, new_file_path)) => {
            wasm_bindgen_futures::JsFuture::from(vault.create(&new_file_path, json_str))
                .await
                .map_err(errors::SlackError::JsError)
                .map(|_| file_name)
        }
        Err(err) => Err(err),
    };

    let clipboard_save_result = match file_creation_result {
        Ok(file_name) => wasm_bindgen_futures::JsFuture::from(writeText(&file_name))
            .await
            .map_err(errors::SlackError::JsError),
        Err(err) => Err(err),
    };

    match clipboard_save_result {
        Ok(_) => {
            Notice::new_with_timeout("Successfully downloaded slack message and saved to attachment file. Attachment file name saved to clipboard", 5000);
        }
        Err(err) => alert(&format!(
            "There was a problem getting slack messages. Error: {:#?}",
            err
        )),
    }
}

fn make_request(params: RequestUrlParam) -> Promise {
    let serializer = Serializer::json_compatible();
    request(params.serialize(&serializer).unwrap())
}
