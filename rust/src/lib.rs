//! This crate provides an Obsidian friendly interface to retrieve
//! Slack messages from the Slack web api and save them to your Obsidian
//! vault without needing to create apps in Slack itself
//!
//! This is possible by using Slack's web interface's 'xoxc' token and
//! corresponding 'xoxd' cookie.

mod errors;
mod slack_http_client;
mod slack_url;
mod utils;

use do_notation::m;
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

/// Interface to get slack messages and save to your obsidian vault
///
/// You can get the `api_token` and `cookie` from Slack's web interface by:
/// 1. Navigate to and login to your Slack workspace in your browser
/// 2. Open your browsers developer console
/// 3. Paste the following snippet into the console:
/// ```javascript
/// JSON.parse(localStorage.localConfig_v2).teams[document.location.pathname.match(/^\/client\/(T[A-Z0-9]+)/)[1]].token
/// ```
/// 4. Save the result. This is your `api_token`
/// 5. In the developer console, navigate to the cookies
/// 6. Find the cookie with key 'd' (it's valuel should start with 'xoxd...')
/// 7. Save the cookies value. This is your `cookie`
///
/// The `url` is a valid slack url. Most commonly, this is retrieved from Slack's
/// web interface/web app by right clicking any message/thread and copying it's
/// link
///
/// The `vault` is the Obisidian vault to save the messages to. See:
/// https://marcus.se.net/obsidian-plugin-docs/vault
///
/// Panics:
/// The function is designed to catch all errors and display an alert in Obsidian
/// with the error.
#[wasm_bindgen]
pub async fn get_slack_message(api_token: String, cookie: String, url: String, vault: Vault) {
    // separate calls for intermediate results due to `and_then` closures not allowing await
    let results_from_api = get_results_from_api(api_token, cookie, url).await;

    let file_creation_result = create_file_from_result(results_from_api, vault).await;

    let clipboard_save_result = save_to_clipboard(file_creation_result).await;

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

async fn get_results_from_api(
    api_token: String,
    cookie: String,
    url: String,
) -> Result<(JsValue, SlackUrl), errors::SlackError> {
    match m! {
        config <- SlackHttpClientConfig::new(get_api_base(), api_token, cookie);
        let client = SlackHttpClient::<Promise>::new(config, make_request);
        slack_url <- SlackUrl::new(&url);
        let ts = wasm_bindgen_futures::JsFuture::from(
            client.get_conversation_replies_using_ts(&slack_url),
        );
        let thread_ts = client.get_conversation_replies_using_thread_ts(&slack_url);
        return (client, slack_url, ts, thread_ts);
    } {
        Ok((_, url, ts_future, thread_ts_option)) => {
            let ts_result = ts_future.await.map_err(errors::SlackError::Js);
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
    }
}

async fn create_file_from_result(
    results_from_api: Result<(JsValue, SlackUrl), errors::SlackError>,
    vault: Vault,
) -> Result<String, errors::SlackError> {
    match m! {
        result <- results_from_api;
        let (result, slack_url) = result;
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
        json_str <- JSON::stringify_with_replacer_and_space(&result, &JsValue::NULL, &JsValue::from_f64(2.0))
            .map_err(errors::SlackError::Js);
        return (json_str, file_name, new_file_path, vault);
    } {
        Ok((json_str, file_name, new_file_path, vault)) => {
            wasm_bindgen_futures::JsFuture::from(vault.create(&new_file_path, json_str))
                .await
                .map_err(errors::SlackError::Js)
                .map(|_| file_name)
        }
        Err(err) => Err(err),
    }
}

async fn save_to_clipboard(
    file_creation_result: Result<String, errors::SlackError>,
) -> Result<JsValue, errors::SlackError> {
    match file_creation_result {
        Ok(file_name) => wasm_bindgen_futures::JsFuture::from(writeText(&file_name))
            .await
            .map_err(errors::SlackError::Js),
        Err(err) => Err(err),
    }
}
