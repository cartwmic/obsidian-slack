//! This crate provides an Obsidian friendly interface to retrieve
//! Slack messages from the Slack web api and save them to your Obsidian
//! vault without needing to create apps in Slack itself
//!
//! This is possible by using Slack's web interface's 'xoxc' token and
//! corresponding 'xoxd' cookie.

mod errors;
mod messages;
mod slack_http_client;
mod slack_url;
mod users;
mod utils;

use derive_builder::Builder;
use do_notation::m;
use errors::SlackError;
use js_sys::{JsString, Promise, JSON};
use messages::Message;
use serde::{Deserialize, Serialize};
use slack_http_client::SlackHttpClientConfigFeatureFlags;
use slack_url::SlackUrl;
use std::{collections::HashMap, path::Path, str::FromStr};
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

use crate::{
    messages::MessageAndThread,
    slack_http_client::{get_api_base, SlackHttpClient, SlackHttpClientConfig},
    users::User,
    utils::{create_file_name, make_request},
};

static ATTACHMENT_FOLDER_CONFIG_KEY: &str = "attachmentFolderPath";

#[derive(Debug, Serialize, Deserialize, Clone, Builder)]
struct MessageAndThreadToSave {
    message: Vec<MessageToSave>,
    thread: Vec<MessageToSave>,
}

impl MessageAndThreadToSave {
    fn from_message_and_thread_and_users(
        message_and_thread: &MessageAndThread,
        users: &HashMap<String, User>,
    ) -> Result<MessageAndThreadToSave, SlackError> {
        let message_messages = message_and_thread
            .message
            .messages
            .as_ref()
            .unwrap()
            .iter()
            .map(|message| MessageToSave::from_message_and_user_map(message, users))
            .collect::<Result<Vec<MessageToSave>, SlackError>>();

        message_messages.and_then(|message_messages| {
            let thread_messages = message_and_thread
                .thread
                .messages
                .as_ref()
                .unwrap()
                .iter()
                .map(|message| MessageToSave::from_message_and_user_map(message, users))
                .collect::<Result<Vec<MessageToSave>, SlackError>>();
            thread_messages.map(|thread_messages| {
                MessageAndThreadToSaveBuilder::default()
                    .thread(thread_messages)
                    .message(message_messages)
                    .build()
                    .unwrap()
            })
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Builder)]
struct MessageToSave {
    r#type: Option<String>,
    user: Option<User>,
    text: Option<String>,
    thread_ts: Option<String>,
    reply_count: Option<u16>,
    team: Option<String>,
    ts: Option<String>,
}

impl MessageToSave {
    fn from_message_and_user_map(
        message: &Message,
        users: &HashMap<String, User>,
    ) -> Result<MessageToSave, SlackError> {
        match users.get(message.user.as_ref().unwrap()) {
            Some(user) => Ok(MessageToSaveBuilder::default()
                .r#type(message.r#type.clone())
                .user(Some(user.to_owned()))
                .text(message.text.clone())
                .thread_ts(message.thread_ts.clone())
                .reply_count(message.reply_count)
                .team(message.team.clone())
                .ts(message.ts.clone())
                .build()
                .unwrap()),
            None => Err(SlackError::MissingUsers),
        }
    }
}

#[wasm_bindgen(module = "index")]
extern "C" {
    #[wasm_bindgen(js_name=obsidian_request)]
    fn request(request: JsValue) -> Promise;
}

#[wasm_bindgen()]
extern "C" {
    fn alert(message: &str);
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
/// with the error. If there is a panic, then there is a programming bug that needs
/// to be addressed
#[wasm_bindgen]
pub async fn get_slack_message(
    api_token: String,
    cookie: String,
    url: String,
    feature_flags: JsValue,
) -> JsValue {
    #[derive(Clone, Serialize)]
    struct Buffer {
        message_and_thread: MessageAndThreadToSave,
        file_name: String,
    }

    // separate calls for intermediate results due to `and_then` closures not allowing await
    let results_from_api = get_results_from_api(api_token, cookie, url, feature_flags).await;

    m! {
        results_from_api <- results_from_api;
        let (message_and_thread, slack_url) = results_from_api;
        let file_name = create_file_name(&slack_url);
        return Buffer {
            message_and_thread,
            file_name
        };
    }
    .map_or_else(
        |err| {
            let message = format!("There was a problem getting slack messages. Error: {}", err);
            log::error!("{}", &message);
            alert(&message);
            JsValue::NULL
        },
        |buffer| serde_wasm_bindgen::to_value(&buffer).unwrap(),
    )
}

async fn get_results_from_api(
    api_token: String,
    cookie: String,
    url: String,
    feature_flags: JsValue,
) -> Result<(MessageAndThreadToSave, SlackUrl), errors::SlackError> {
    // separate calls for intermediate results due to `and_then` closures not allowing await
    #[derive(Clone, Builder, Default)]
    #[builder(default)]
    struct Buffer {
        message_and_thread: Option<MessageAndThread>,
        slack_url: Option<SlackUrl>,
        users: Option<HashMap<String, User>>,
        client: Option<SlackHttpClient<Promise>>,
    }

    let buffer = m! {
        feature_flags <- serde_wasm_bindgen::from_value(feature_flags).map_err(errors::SlackError::SerdeWasmBindgen);
        let _ = log::info!("{:#?}", feature_flags);
        config <- SlackHttpClientConfig::new(
                get_api_base(),
                api_token.to_string(),
                cookie.to_string(),
                feature_flags,
            );
        slack_url <- SlackUrl::new(&url);
        let client = SlackHttpClient::<Promise>::new(config, make_request);
        return BufferBuilder::create_empty()
                .client(Some(client))
                .slack_url(Some(slack_url))
                .build()
                .unwrap();
    };

    let buffer = match buffer {
        Ok(mut buffer) => messages::get_messages_from_api(
            buffer.client.as_ref().unwrap(),
            buffer.slack_url.as_ref().unwrap(),
        )
        .await
        .map(|message_and_thread| {
            buffer.message_and_thread = Some(message_and_thread);
            buffer
        }),
        Err(err) => Err(err),
    };

    let buffer = match buffer {
        Ok(mut buffer) => match buffer.message_and_thread.as_ref().unwrap().collect_users() {
            Ok(user_ids) => users::get_users_from_api(&user_ids, buffer.client.as_ref().unwrap())
                .await
                .map(|users| {
                    buffer.users = Some(users);
                    buffer
                }),
            Err(err) => Err(err),
        },
        Err(err) => Err(err),
    };

    buffer.and_then(|buffer| {
        MessageAndThreadToSave::from_message_and_thread_and_users(
            buffer.message_and_thread.as_ref().unwrap(),
            buffer.users.as_ref().unwrap(),
        )
        .map(|message_and_thread| (message_and_thread, buffer.slack_url.unwrap()))
    })
}
