use std::{collections::HashSet, fmt::format};

use do_notation::m;
use js_sys::Promise;
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use wasm_bindgen::JsValue;

use crate::{
    response::{self, convert_result_string_to_object, SlackResponseValidator},
    slack_http_client::{self, get_api_base, SlackHttpClient, SlackHttpClientConfig},
    slack_url::SlackUrl,
    users::User,
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Awaiting a JsFuture returned an error: {error}"))]
    WasmErrorFromJsFuture { error: String },

    #[snafu(display("The message response was not ok. - source: {source}"))]
    InvalidMessageResponse { source: response::Error },

    #[snafu(display("{source}"))]
    CouldNotParseJsonFromMessageResponse { source: response::Error },

    #[snafu(display(
        "Attempted to retrieve the user id for the message, but found none: {message}"
    ))]
    UserIdWasNoneInMessage { message: String },

    #[snafu(display("Attempted to access messages in message response, but no messages found: {message_response}"))]
    MessagesNotFoundInMessageResponse { message_response: String },

    #[snafu(display("{source}"))]
    SerdeWasmBindgenCouldNotParseMessageResponse { source: response::Error },
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Serialize, Deserialize, Clone)]
pub struct MessageAndThread {
    pub message: MessageResponse,
    pub thread: MessageResponse,
}

impl MessageAndThread {
    pub fn collect_users(&self) -> Result<HashSet<String>> {
        self.message
            .collect_users()
            .and_then(|mut message_users| {
                self.thread.collect_users().map(|thread_users| {
                    message_users.extend(thread_users);
                    message_users
                })
            })
            .map_or(
                UserIdWasNoneInMessageSnafu {
                    message: format!("{:#?}", self.message),
                }
                .fail(),
                |user_ids| Ok(user_ids.into_iter().collect()),
            )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub r#type: Option<String>,
    pub user: Option<String>,
    pub text: Option<String>,
    pub thread_ts: Option<String>,
    pub reply_count: Option<u16>,
    pub team: Option<String>,
    pub ts: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageResponseMetadata {
    next_cursor: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageResponse {
    pub is_null: Option<bool>,
    pub messages: Option<Vec<Message>>,
    pub has_more: Option<bool>,
    pub ok: Option<bool>,
    pub error: Option<String>,
    pub response_metadata: Option<MessageResponseMetadata>,
}

impl MessageResponse {
    fn copy_from_existing_given_seed_ts(&self, seed_ts: &str) -> MessageResponse {
        let mut copy = self.to_owned();
        copy.messages = Some(
            copy.messages
                .unwrap()
                .into_iter()
                .filter(|message| message.ts.as_ref().unwrap() == seed_ts)
                .collect(),
        );
        copy
    }

    pub fn collect_users(&self) -> Result<Vec<String>> {
        self.messages.as_ref().map_or(
            MessagesNotFoundInMessageResponseSnafu {
                message_response: format!("{:#?}", &self),
            }
            .fail(),
            |messages| {
                messages
                    .iter()
                    .map(|message| {
                        message.user.as_ref().map_or(
                            UserIdWasNoneInMessageSnafu {
                                message: format!("{:#?}", message),
                            }
                            .fail(),
                            |user| Ok(user.to_string()),
                        )
                    })
                    .collect::<Result<Vec<String>>>()
            },
        )
    }
}

impl SlackResponseValidator for MessageResponse {
    fn ok(&self) -> Option<bool> {
        self.ok
    }
}

pub async fn get_messages_from_api<T>(
    client: &SlackHttpClient<T>,
    slack_url: &SlackUrl,
) -> Result<MessageAndThread>
where
    wasm_bindgen_futures::JsFuture: std::convert::From<T>,
{
    let thread_ts = match slack_url.thread_ts.clone() {
        Some(thread_ts) => thread_ts,
        None => slack_url.clone().ts,
    };

    let awaited_val = wasm_bindgen_futures::JsFuture::from(
        client.get_conversation_replies(&slack_url.channel_id, &thread_ts),
    )
    .await
    // mapping error instead of using snafu context because jsvalue is not an Error from parse method
    .map_err(|err| Error::WasmErrorFromJsFuture {
        error: format!("{:#?}", err),
    });

    let response = m! {
        awaited_val <- awaited_val;
        js_obj <- convert_result_string_to_object(awaited_val).context(CouldNotParseJsonFromMessageResponseSnafu);
        message_response <- response::defined_from_js_object(js_obj).context(SerdeWasmBindgenCouldNotParseMessageResponseSnafu);
        valid_response <- MessageResponse::validate_response(message_response).context(InvalidMessageResponseSnafu);
        return valid_response;
    }?;
    let copy = MessageResponse::copy_from_existing_given_seed_ts(&response, &slack_url.ts);

    Ok(MessageAndThread {
        message: copy,
        thread: response,
    })
}
