//! This crate provides an Obsidian friendly interface to retrieve
//! Slack messages from the Slack web api and save them to your Obsidian
//! vault without needing to create apps in Slack itself
//!
//! This is possible by using Slack's web interface's 'xoxc' token and
//! corresponding 'xoxd' cookie.

mod errors;
pub mod messages;
pub mod slack_http_client;
mod slack_url;
pub mod users;
mod utils;

use derive_builder::Builder;
use do_notation::m;
use errors::SlackError;
use js_sys::{JsString, Promise, JSON};
use messages::Message;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::Serializer;
use slack_http_client::SlackHttpClientConfigFeatureFlags;
use slack_url::SlackUrl;
use std::{collections::HashMap, path::Path, str::FromStr};
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

use crate::{
    messages::MessageAndThread,
    slack_http_client::{get_api_base, RequestUrlParam, SlackHttpClient, SlackHttpClientConfig},
    users::User,
    utils::create_file_name,
};

#[derive(Debug, Serialize, Deserialize, Clone, Builder, PartialEq)]
pub struct ObsidianSlackReturnData {
    pub message_and_thread: MessageAndThreadToSave,
}

#[derive(Debug, Serialize, Deserialize, Clone, Builder, PartialEq, Default)]
#[builder(default)]
pub struct MessageAndThreadToSave {
    pub message: Vec<MessageToSave>,
    pub thread: Vec<MessageToSave>,
    pub file_name: String,
}

impl MessageAndThreadToSave {
    fn from_components(
        message_and_thread: &MessageAndThread,
        users: Option<&HashMap<String, User>>,
    ) -> Result<MessageAndThreadToSave, SlackError> {
        let message_messages = message_and_thread
            .message
            .messages
            .as_ref()
            .expect("Expected messages to unwrap, no messages found for main message")
            .iter()
            .map(|message| MessageToSave::from_components(message, users))
            .collect::<Result<Vec<MessageToSave>, SlackError>>()?;

        let thread_messages = message_and_thread
            .thread
            .messages
            .as_ref()
            .expect("Expected messages to unwrap, no messages found for thread")
            .iter()
            .map(|message| MessageToSave::from_components(message, users))
            .collect::<Result<Vec<MessageToSave>, SlackError>>()?;

        Ok(MessageAndThreadToSaveBuilder::default()
            .thread(thread_messages)
            .message(message_messages)
            .build()
            .unwrap())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Builder, PartialEq, Default)]
#[builder(default)]
pub struct MessageToSave {
    pub r#type: Option<String>,
    pub user_id: Option<String>,
    pub user: Option<User>,
    pub text: Option<String>,
    pub thread_ts: Option<String>,
    pub reply_count: Option<u16>,
    pub team: Option<String>,
    pub ts: Option<String>,
}

impl MessageToSave {
    fn from_components(
        message: &Message,
        users: Option<&HashMap<String, User>>,
    ) -> Result<MessageToSave, SlackError> {
        match users {
            Some(users) => match users.get(message.user.as_ref().unwrap()) {
                Some(user) => Ok(MessageToSaveBuilder::default()
                    .r#type(message.r#type.clone())
                    .user_id(Some(message.user.as_ref().unwrap().to_string()))
                    .user(Some(user.to_owned()))
                    .text(message.text.clone())
                    .thread_ts(message.thread_ts.clone())
                    .reply_count(message.reply_count)
                    .team(message.team.clone())
                    .ts(message.ts.clone())
                    .build()
                    .unwrap()),
                None => Err(SlackError::MissingUsers),
            },
            None => Ok(MessageToSaveBuilder::default()
                .r#type(message.r#type.clone())
                .user_id(Some(message.user.as_ref().unwrap().to_string()))
                .text(message.text.clone())
                .thread_ts(message.thread_ts.clone())
                .reply_count(message.reply_count)
                .team(message.team.clone())
                .ts(message.ts.clone())
                .build()
                .unwrap()),
        }
    }
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
    request_func: JsValue,
) -> JsValue {
    // separate calls for intermediate results due to `and_then` closures not allowing await
    let results_from_api =
        get_results_from_api(api_token, cookie, url, feature_flags, request_func).await;

    m! {
        results_from_api <- results_from_api;
        let (mut message_and_thread, slack_url) = results_from_api;
        let file_name = create_file_name(&slack_url);
        return {
            message_and_thread.file_name = file_name;
            ObsidianSlackReturnData {message_and_thread}
        };
    }
    .map_or_else(
        |err| {
            let message = format!("There was a problem getting slack messages. Error: {}", err);
            log::error!("{}", &message);
            JsValue::from_str(&message)
        },
        |buffer| serde_wasm_bindgen::to_value(&buffer).unwrap(),
    )
}

fn curry_request_func(
    request_func: js_sys::Function,
) -> Box<dyn Fn(slack_http_client::RequestUrlParam) -> js_sys::Promise> {
    Box::new(move |params: RequestUrlParam| -> Promise {
        let serializer = Serializer::json_compatible();
        js_sys::Promise::from(
            request_func
                .call1(&JsValue::NULL, &params.serialize(&serializer).unwrap())
                .unwrap(),
        )
    })
}

async fn get_results_from_api(
    api_token: String,
    cookie: String,
    url: String,
    feature_flags: JsValue,
    request_func: JsValue,
) -> Result<(MessageAndThreadToSave, SlackUrl), errors::SlackError> {
    // separate calls for intermediate results due to `and_then` closures not allowing await
    let make_request = curry_request_func(js_sys::Function::from(request_func));

    let (client, slack_url) = m! {
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
        return (client, slack_url);
    }?;

    let messages = messages::get_messages_from_api(&client, &slack_url).await?;

    let users = if client.config.feature_flags.get_users {
        let user_ids = messages.collect_users()?;
        let users = users::get_users_from_api(&user_ids, &client).await?;
        Some(users)
    } else {
        None
    };

    MessageAndThreadToSave::from_components(&messages, users.as_ref())
        .map(|message_and_thread| (message_and_thread, slack_url))
}
