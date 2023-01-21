//! This crate provides an Obsidian friendly interface to retrieve
//! Slack messages from the Slack web api and save them to your Obsidian
//! vault without needing to create apps in Slack itself
//!
//! This is possible by using Slack's web interface's 'xoxc' token and
//! corresponding 'xoxd' cookie.

pub mod channels;
pub mod components;
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
use channels::{Channel, ChannelId};
use components::{FileName, ObsidianSlackComponents, ObsidianSlackComponentsBuilder};
use derive_builder::Builder;
use do_notation::m;
use js_sys::Promise;
use messages::{Message, MessageResponse};
use serde::{Deserialize, Serialize};
use slack_url::SlackUrl;
use snafu::{ResultExt, Snafu};
use std::{collections::HashMap, hash::Hash, str::FromStr};
use users::CollectUser;
use utils::{curry_request_func, set_panic_hook};
use wasm_bindgen::prelude::*;

#[derive(Debug, Snafu)]
pub enum Error {
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

    #[snafu(display("Could not get channel from api - source: {source}"))]
    CouldNotGetChannelFromApi { source: channels::Error },

    #[snafu(display("Could not get users from messages - source: {source}"))]
    CouldNotGetUsersFromMessages { source: messages::Error },

    #[snafu(display(
        "There was a problem gathering components of message request - source: {source}"
    ))]
    CouldNotBuildComponentsTogether {
        source: components::ObsidianSlackComponentsBuilderError,
    },

    #[snafu(display("There was a problem finalizing components to save - source {source}"))]
    CouldNotFinalizeComponents { source: components::Error },

    #[snafu(display("There was a problem finalizing messages for saving - source: {source}"))]
    CouldNotFinalizeMessages { source: messages::Error },
}

type Result<T, E = Error> = std::result::Result<T, E>;

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
        let (mut components_builder, slack_url) = results_from_api;
        let file_name = create_file_name(&slack_url);
        components <- components_builder.file_name(FileName(file_name)).build().context(CouldNotBuildComponentsTogetherSnafu);
        components <- ObsidianSlackComponents::finalize(components).context(CouldNotFinalizeComponentsSnafu);
        return components;
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
) -> Result<(ObsidianSlackComponentsBuilder, SlackUrl)> {
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

    let message_and_thread = messages::get_messages_from_api(&client, &slack_url)
        .await
        .context(CouldNotGetMessagesFromApiSnafu)?;

    let components_builder = &mut ObsidianSlackComponentsBuilder::default();
    components_builder.message_and_thread(message_and_thread);
    components_builder.channel_id(ChannelId(slack_url.channel_id.clone()));

    if client.config.feature_flags.get_channel_info {
        components_builder.channel(Some(
            channels::get_channel_from_api(
                &client,
                components_builder
                    .channel_id
                    .as_ref()
                    .expect("Expect channel id, found None. This is a bug"),
            )
            .await
            .context(CouldNotGetChannelFromApiSnafu)?,
        ));
    } else {
        components_builder.channel(None);
    }

    if client.config.feature_flags.get_users {
        let mut user_ids = components_builder
            .message_and_thread
            .as_ref()
            .expect("expected a message and thread, found None. This is a bug")
            .collect_users()
            .context(CouldNotGetUsersFromMessagesSnafu)?;

        components_builder
            .channel
            .as_ref()
            .expect("expected a channel option, but found None, this is a bug")
            .as_ref()
            .map_or(Ok::<(), Error>(()), |channel| {
                Ok(user_ids.extend(channel.collect_users().unwrap_or_else(|err| vec![])))
            });

        let users = users::get_users_from_api(&user_ids, &client)
            .await
            .context(CouldNotGetUsersFromApiSnafu)?;

        components_builder.users(Some(users));
    } else {
        components_builder.users(None);
    };

    Ok((components_builder.to_owned(), slack_url))
}
