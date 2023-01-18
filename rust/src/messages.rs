use amplify_derive::Display;
use do_notation::m;
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use std::collections::{HashMap, HashSet};

use crate::{
    response::{self, convert_result_string_to_object, SlackResponseValidator},
    slack_http_client::SlackHttpClient,
    slack_url::SlackUrl,
    users::{self, CollectUser, User},
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("When mapping user ids from response to retrieved user info, user id was not in user map. user_id: {user_id} - user_map: {user_map}"))]
    UserIdNotFoundInUserMap { user_id: String, user_map: String },

    #[snafu(display("Awaiting a JsFuture returned an error: {error}"))]
    WasmErrorFromJsFuture { error: String },

    #[snafu(display("The message response was not ok. - source: {source}"))]
    InvalidMessageResponse { source: response::Error },

    #[snafu(display("{source}"))]
    CouldNotParseJsonFromMessageResponse { source: response::Error },

    #[snafu(display(
        "Attempted to retrieve the user id for the message, but found none: {container}"
    ))]
    UserIdWasNoneInMessage { container: Message },

    #[snafu(display(
        "Attempted to retrieve user ids for messages in message response, but found none: message_response: {message_response} - thread_response: {thread_response}"
    ))]
    NoUsersFoundInMessageResponseOrThreadResponse {
        message_response: MessageResponse,
        thread_response: MessageResponse,
    },

    #[snafu(display("Attempted to access messages in message response, but no messages found: {message_response}"))]
    MessagesNotFoundInMessageResponse { message_response: String },

    #[snafu(display("{source}"))]
    SerdeWasmBindgenCouldNotParseMessageResponse { source: response::Error },
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub async fn get_messages_from_api<T>(
    client: &SlackHttpClient<T>,
    slack_url: &SlackUrl,
) -> Result<MessageAndThread>
where
    wasm_bindgen_futures::JsFuture: std::convert::From<T>,
{
    let thread_ts = slack_url.thread_ts.as_ref().unwrap_or(&slack_url.ts);

    let awaited_val = wasm_bindgen_futures::JsFuture::from(
        client.get_conversation_replies(&slack_url.channel_id, thread_ts),
    )
    .await
    // mapping error instead of using snafu context because jsvalue is not an Error from parse method
    .map_err(|err| Error::WasmErrorFromJsFuture {
        error: format!("{:#?}", err),
    })?;

    let response = m! {
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageResponseMetadata {
    next_cursor: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Display)]
#[display(Debug)]
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
        copy.messages =
            Some(
                copy.messages
                    .expect("Expected messages to work on, got None. This is a bug")
                    .into_iter()
                    .filter(|message| {
                        message.ts.as_ref().expect(
                            "Expected message to have a timestamp, but got None. This is a bug",
                        ) == seed_ts
                    })
                    .collect(),
            );
        copy
    }

    fn finalize_message_response(
        mut message_response: MessageResponse,
        user_map: Option<&HashMap<String, User>>,
    ) -> Result<MessageResponse> {
        message_response.messages = Some(
            message_response
                .messages
                .expect("expected messages to finalize, None was available. This is a bug")
                .into_iter()
                .map(|mut message| {
                    message = Message::finalize_message(message, user_map)?;
                    Ok(message)
                })
                .collect::<Result<Vec<Message>>>()?,
        );
        Ok(message_response)
    }

}

impl CollectUser<Error> for MessageResponse {
    fn collect_users(&self) -> Result<Vec<String>> {
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
                                container: message.to_owned(),
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

#[derive(Serialize, Deserialize, Clone, Debug, Display)]
#[display(Debug)]
pub struct MessageAndThread {
    pub message: MessageResponse,
    pub thread: MessageResponse,
}

impl CollectUser<Error> for MessageAndThread {
    fn collect_users(&self) -> Result<Vec<String>> {
        self.message
            .collect_users()
            .and_then(|mut message_users| {
                self.thread.collect_users().map(|thread_users| {
                    message_users.extend(thread_users);
                    message_users
                })
            })
            .map_or(
                NoUsersFoundInMessageResponseOrThreadResponseSnafu {
                    message_response: self.message.clone(),
                    thread_response: self.thread.clone(),
                }
                .fail(),
                |user_ids| {
                    Ok(user_ids
                        .into_iter()
                        .collect::<HashSet<String>>()
                        .into_iter()
                        .collect())
                },
            )
    }
}

impl MessageAndThread {
    pub fn finalize_message_and_thread(
        mut message_and_thread: MessageAndThread,
        user_map: Option<&HashMap<String, User>>,
    ) -> Result<MessageAndThread> {
        message_and_thread.message = MessageResponse::finalize_message_response(
            message_and_thread.message,
            user_map,
        )?;
        message_and_thread.thread = MessageResponse::finalize_message_response(
            message_and_thread.thread,
            user_map,
        )?;
        Ok(message_and_thread)
    }

}

#[derive(Debug, Serialize, Deserialize, Clone, Display)]
#[display(Debug)]
pub struct Message {
    pub r#type: Option<String>,
    pub user: Option<String>,
    pub user_info: Option<User>,
    pub text: Option<String>,
    pub thread_ts: Option<String>,
    pub reply_count: Option<u16>,
    pub team: Option<String>,
    pub ts: Option<String>,
}

impl Message {
    fn finalize_message(
        mut message: Message,
        user_map: Option<&HashMap<String, User>>,
    ) -> Result<Message> {
        match user_map {
            Some(user_map) => {
                let user_id = message
                    .user
                    .as_ref()
                    .expect("expected a user id, got None. This should never happen");
                user_map.get(user_id).map_or(
                    UserIdNotFoundInUserMapSnafu {
                        user_id,
                        user_map: format!("{:#?}", user_map),
                    }
                    .fail(),
                    |user| {
                        Ok({
                            message.user_info = Some(user.to_owned());
                            message
                        })
                    },
                )
            }
            None => Ok(message),
        }
    }
}

