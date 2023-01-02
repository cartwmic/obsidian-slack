use std::collections::HashSet;

use do_notation::m;
use js_sys::{Promise, JSON};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use crate::{
    errors::{self, SlackError},
    make_request,
    slack_http_client::{
        get_api_base, SlackHttpClient, SlackHttpClientConfig, SlackResponseValidator,
    },
    slack_url::SlackUrl,
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
    pub r#type: String,
    pub user: String,
    pub text: String,
    pub thread_ts: String,
    pub reply_count: u16,
    pub team: String,
    pub ts: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageResponseMetadata {
    next_cursor: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageResponse {
    pub is_null: Option<bool>,
    pub messages: Option<Vec<Message>>,
    has_more: Option<bool>,
    is_thread: Option<bool>,
    pub ok: Option<bool>,
    pub error: Option<String>,
    response_metadata: Option<MessageResponseMetadata>,
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
        let mut copy = self.to_owned().clone();
        copy.is_thread = Some(false);
        copy.messages = Some(
            copy.messages
                .unwrap()
                .into_iter()
                .filter(|message| message.ts == seed_ts)
                .collect(),
        );
        copy
    }

    pub fn collect_users(&self) -> Option<Vec<String>> {
        self.messages.as_ref().map(|messages| {
            messages
                .into_iter()
                .map(|message| message.user.to_string())
                .collect()
        })
    }
}

impl SlackResponseValidator for MessageResponse {
    fn ok(&self) -> Option<bool> {
        self.ok
    }
}

// pub fn validate_result(val: JsValue) -> Result<JsValue, SlackError> {
//     m! {
//         let key = "ok";
//         _ <- js_sys::Reflect::has(&val, &JsValue::from_str(key))
//              .map_err(|err| errors::SlackError::ResponseMissingOkField(format!("{:#?} | {:#?}", err, val)))
//              .and_then(|has_ok| {
//                 if has_ok {Ok(has_ok)} else {Err(errors::SlackError::ResponseMissingOkField("".to_string()))}
//              });
//         is_ok <- js_sys::Reflect::get(&val, &JsValue::from_str(key))
//                  .map_err(|err| errors::SlackError::ResponseMissingOkField(format!("{:#?} | {:#?}", err, val)));
//         is_ok <- is_ok
//                  .as_bool()
//                  .map_or(Err(errors::SlackError::ResponseOkNotABoolean(format!("{:#?}", val))), Ok);
//         return (is_ok, val);
//     }.and_then(|(is_ok, val)| {
//         if is_ok {
//             Ok(val)
//         } else {
//             Err(SlackError::ResponseNotOk(format!("{:#?}", val)))
//         }
//     })
// }

pub fn convert_result_string_to_object(val: JsValue) -> Result<JsValue, SlackError> {
    // results from the `request` function of obsidian return strings
    m! {
        str_val <- val
                   .as_string()
                   .map_or(Err(errors::SlackError::EmptyResult(format!("{:#?}", val))), Ok);
        obj_val <- JSON::parse(&str_val)
                   .map_err(|err| errors::SlackError::ResponseNotAnObject(format!("{:#?} | {:#?}", err, val)));
        return obj_val;
    }
}

pub async fn get_messages_from_api(
    api_token: &str,
    cookie: &str,
    url: &str,
) -> Result<(MessageAndThread, SlackUrl), SlackError> {
    let client =
        SlackHttpClientConfig::new(get_api_base(), api_token.to_string(), cookie.to_string())
            .map(|config| SlackHttpClient::<Promise>::new(config, make_request));
    let client_and_url = client.and_then(|client| SlackUrl::new(&url).map(|url| (client, url)));
    match client_and_url {
        Ok((client, slack_url)) => {
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
                let copy =
                    MessageResponse::copy_from_existing_given_seed_ts(&response, &slack_url.ts);

                MessageAndThread {
                    message: response,
                    thread: copy,
                }
            })
            .map(|message_and_thread| (message_and_thread, slack_url))
        }
        Err(err) => Err(err),
    }
}
