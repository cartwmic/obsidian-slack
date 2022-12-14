use std::collections::HashSet;

use js_sys::Promise;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use crate::{
    errors::{self, SlackError},
    slack_http_client::{
        get_api_base, SlackHttpClient, SlackHttpClientConfig, SlackResponseValidator,
    },
    slack_url::SlackUrl,
    utils::convert_result_string_to_object,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct MessageAndThread {
    pub message: MessageResponse,
    pub thread: MessageResponse,
}

impl MessageAndThread {
    pub fn collect_users(&self) -> Result<HashSet<String>, SlackError> {
        match self.message.collect_users().and_then(|mut message_users| {
            self.thread.collect_users().map(|thread_users| {
                message_users.extend(thread_users);
                message_users
            })
        }) {
            Some(user_ids) => Ok(user_ids.into_iter().collect()),
            None => Err(errors::SlackError::MissingUsers),
        }
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
    fn defined_from_js_object(val: JsValue) -> Result<MessageResponse, SlackError> {
        if val.is_object() {
            serde_wasm_bindgen::from_value(val)
                .map_err(errors::SlackError::SerdeWasmBindgen)
                .map(|mut val: MessageResponse| {
                    val.is_null = Some(false);
                    val
                })
        } else {
            Err(errors::SlackError::JsValueNotObject(format!(
                "value was not a javascript object. got {:#?} instead",
                val.js_typeof()
            )))
        }
    }

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

    pub fn collect_users(&self) -> Option<Vec<String>> {
        self.messages.as_ref().map(|messages| {
            messages
                .iter()
                .map(|message| message.user.as_ref().unwrap().to_string())
                .collect()
        })
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
) -> Result<MessageAndThread, SlackError>
where
    wasm_bindgen_futures::JsFuture: std::convert::From<T>,
{
    let thread_ts = match slack_url.thread_ts.clone() {
        Some(thread_ts) => thread_ts,
        None => slack_url.clone().ts,
    };
    wasm_bindgen_futures::JsFuture::from(
        client.get_conversation_replies(&slack_url.channel_id, &thread_ts),
    )
    .await
    .map_err(errors::SlackError::Js)
    .and_then(convert_result_string_to_object)
    .and_then(MessageResponse::defined_from_js_object)
    .and_then(MessageResponse::validate_response)
    .map(|response| {
        let copy = MessageResponse::copy_from_existing_given_seed_ts(&response, &slack_url.ts);

        MessageAndThread {
            message: copy,
            thread: response,
        }
    })
}
