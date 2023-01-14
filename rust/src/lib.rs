//! This crate provides an Obsidian friendly interface to retrieve
//! Slack messages from the Slack web api and save them to your Obsidian
//! vault without needing to create apps in Slack itself
//!
//! This is possible by using Slack's web interface's 'xoxc' token and
//! corresponding 'xoxd' cookie.

pub mod messages;
mod response;
pub mod slack_http_client;
mod slack_url;
pub mod users;
mod utils;

use crate::{
    messages::MessageAndThread,
    slack_http_client::{get_api_base, SlackHttpClient, SlackHttpClientConfig},
    users::User,
    utils::create_file_name,
};
use derive_builder::Builder;
use do_notation::m;
use js_sys::Promise;
use messages::Message;
use serde::{Deserialize, Serialize};
use slack_url::SlackUrl;
use snafu::{ResultExt, Snafu};
use std::{collections::HashMap, str::FromStr};
use utils::{curry_request_func, set_panic_hook};
use wasm_bindgen::prelude::*;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("When mapping user ids from response to retrieved user info, user id was not in user map. user_id: {user_id} - user_map: {user_map}"))]
    UserIdNotFoundInUserMap { user_id: String, user_map: String },

    #[snafu(display(
        "Could not parse feature flags js value to a feature flags rust object: {feature_flags} - source: {source}"
    ))]
    CouldNotParseFeatureFlags {
        feature_flags: String,
        source: serde_wasm_bindgen::Error,
    },

    #[snafu(display("Could not create slack http client config - source: {source}"))]
    ErrorCreatingSlackHttpClientConfig { source: slack_http_client::Error },

    #[snafu(display("Could not create slack url - source: {source}"))]
    ErrorCreatingSlackUrl { source: slack_url::Error },

    #[snafu(display("Could not get messages from api - source: {source}"))]
    CouldNotGetMessagesFromApi { source: messages::Error },

    #[snafu(display("Could not get users from api - source: {source}"))]
    CouldNotGetUsersFromApi { source: users::Error },

    #[snafu(display("Could not get users from messages - source: {source}"))]
    CouldNotGetUsersFromMessages { source: messages::Error },
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Serialize, Deserialize, Clone, Builder, PartialEq, Eq)]
pub struct ObsidianSlackReturnData {
    pub message_and_thread: MessageAndThreadToSave,
}

#[derive(Debug, Serialize, Deserialize, Clone, Builder, PartialEq, Eq, Default)]
#[builder(default)]
pub struct MessageAndThreadToSave {
    pub message: Vec<MessageToSave>,
    pub thread: Vec<MessageToSave>,
    pub file_name: String,
}

impl MessageAndThreadToSave {
    fn from_components<'a>(
        message_and_thread: &'a MessageAndThread,
        users: Option<&'a HashMap<String, User>>,
    ) -> Result<MessageAndThreadToSave> {
        let message_messages = message_and_thread
            .message
            .messages
            .as_ref()
            .expect("Expected messages to unwrap, no messages found for main message")
            .iter()
            .map(|message| MessageToSave::from_components(message, users))
            .collect::<Result<Vec<MessageToSave>>>()?;

        let thread_messages = message_and_thread
            .thread
            .messages
            .as_ref()
            .expect("Expected messages to unwrap, no messages found for thread")
            .iter()
            .map(|message| MessageToSave::from_components(message, users))
            .collect::<Result<Vec<MessageToSave>>>()?;

        Ok(MessageAndThreadToSaveBuilder::default()
            .thread(thread_messages)
            .message(message_messages)
            .build()
            .unwrap())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Builder, PartialEq, Eq, Default)]
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
    fn from_components<'a>(
        message: &'a Message,
        users: Option<&'a HashMap<String, User>>,
    ) -> Result<MessageToSave> {
        match users {
            Some(users) => {
                let user_id = message
                    .user
                    .as_ref()
                    .expect("expected a user id, got None. This should never happen");
                users.get(user_id).map_or(
                    UserIdNotFoundInUserMapSnafu {
                        user_id,
                        user_map: format!("{:#?}", users),
                    }
                    .fail(),
                    |user| {
                        Ok(MessageToSaveBuilder::default()
                            .r#type(message.r#type.clone())
                            .user_id(Some(
                                message
                                    .user
                                    .as_ref()
                                    .expect(
                                        "expected a user id, got None. This should never happen",
                                    )
                                    .to_string(),
                            ))
                            .user(Some(user.to_owned()))
                            .text(message.text.clone())
                            .thread_ts(message.thread_ts.clone())
                            .reply_count(message.reply_count)
                            .team(message.team.clone())
                            .ts(message.ts.clone())
                            .build()
                            .unwrap())
                    },
                )
            }
            None => Ok(MessageToSaveBuilder::default()
                .r#type(message.r#type.clone())
                .user_id(Some(
                    message
                        .user
                        .as_ref()
                        .expect("expected a user id, got None. This should never happen")
                        .to_string(),
                ))
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
            let message = format!(
                "There was a problem getting slack messages. Error message: {} - Error struct: {:#?}",
                &err,
                &err
            );
            log::error!("{}", &message);
            JsValue::from_str(&message)
        },
        |buffer| serde_wasm_bindgen::to_value(&buffer).expect("Expected to serialize object with serde, but was unable to. This is a bug"),
    )
}

async fn get_results_from_api(
    api_token: String,
    cookie: String,
    url: String,
    feature_flags: JsValue,
    request_func: JsValue,
) -> Result<(MessageAndThreadToSave, SlackUrl)> {
    // separate calls for intermediate results due to `and_then` closures not allowing await
    let make_request = curry_request_func(js_sys::Function::from(request_func));
    let feature_flags_string = format!("{:#?}", feature_flags);

    let (client, slack_url) = m! {
        feature_flags <- serde_wasm_bindgen::from_value(feature_flags).context(CouldNotParseFeatureFlagsSnafu {feature_flags: feature_flags_string});
        config <- SlackHttpClientConfig::new(
                get_api_base(),
                api_token.to_string(),
                cookie.to_string(),
                feature_flags,
            ).context(ErrorCreatingSlackHttpClientConfigSnafu);
        slack_url <- SlackUrl::new(&url).context(ErrorCreatingSlackUrlSnafu);
        let client = SlackHttpClient::<Promise>::new(config, make_request);
        return (client, slack_url);
    }?;

    let messages = messages::get_messages_from_api(&client, &slack_url)
        .await
        .context(CouldNotGetMessagesFromApiSnafu)?;

    let users = if client.config.feature_flags.get_users {
        let user_ids = messages
            .collect_users()
            .context(CouldNotGetUsersFromMessagesSnafu)?;
        let users = users::get_users_from_api(&user_ids, &client)
            .await
            .context(CouldNotGetUsersFromApiSnafu)?;
        Some(users)
    } else {
        None
    };

    MessageAndThreadToSave::from_components(&messages, users.as_ref())
        .map(|message_and_thread| (message_and_thread, slack_url))
}
