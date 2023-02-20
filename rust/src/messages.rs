use amplify_derive::Display;
use do_notation::m;
use serde::{Deserialize, Serialize};
use shrinkwraprs::Shrinkwrap;
use snafu::{ResultExt, Snafu};
use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
};

use crate::{
    response::{self, convert_result_string_to_object, SlackResponseValidator},
    slack_http_client::SlackHttpClient,
    slack_url::SlackUrl,
    users::{CollectUsers, User, UserIds, Users},
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
        "Attempted to retrieve user ids for messages in message response, but found none: message_and_thread: {message_and_thread} - previous_err: {previous_err}"
    ))]
    NoUsersFoundInMessageResponseOrThreadResponse {
        message_and_thread: MessageAndThread,
        previous_err: Box<Error>,
    },

    #[snafu(display("Attempted to access messages in message response, but no messages found: {message_response}"))]
    MessagesNotFoundInMessageResponse { message_response: String },

    #[snafu(display("{source}"))]
    SerdeWasmBindgenCouldNotParseMessageResponse { source: response::Error },

    #[snafu(display("{file_url}"))]
    FileDataWasNotString { file_url: String },
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub async fn get_file_data_from_slack<T>(
    client: &SlackHttpClient<T>,
    file_url: String,
) -> Result<String>
where
    wasm_bindgen_futures::JsFuture: std::convert::From<T>,
{
    let awaited_val = wasm_bindgen_futures::JsFuture::from(client.get_file_data(&file_url))
        .await
        // mapping error instead of using snafu context because jsvalue is not an Error from parse method
        .map_err(|err| Error::WasmErrorFromJsFuture {
            error: format!("{:#?}", err),
        })?;

    let result = if awaited_val.is_string() {
        Ok(awaited_val
            .as_string()
            .expect("Expected string, found None. This is a bug"))
    } else {
        FileDataWasNotStringSnafu { file_url }.fail()
    };

    log::info!("attachment value: {:#?}", awaited_val);

    result
}

pub async fn get_messages_from_api<T>(
    client: &SlackHttpClient<T>,
    slack_url: &SlackUrl,
) -> Result<MessageAndThread>
where
    wasm_bindgen_futures::JsFuture: std::convert::From<T>,
{
    let thread_ts = slack_url.thread_ts.as_ref().unwrap_or(&slack_url.ts);

    let awaited_val = wasm_bindgen_futures::JsFuture::from(
        client.get_conversations_replies(&slack_url.channel_id, thread_ts),
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
        message: Messages(
            copy.messages
                .expect("Expected messsages but found None, this is a bug"),
        ),
        thread: Messages(
            response
                .messages
                .expect("Expected messsages but found None, this is a bug"),
        ),
    })
}

#[derive(Serialize, Deserialize, Clone, Debug, Display, PartialEq, Eq)]
#[display(Debug)]
pub struct MessageAndThread {
    pub message: Messages,
    pub thread: Messages,
}

impl CollectUsers<Error> for MessageAndThread {
    fn collect_users(&self) -> Result<UserIds> {
        self.message
            .collect_users()
            .and_then(|mut message_users| {
                self.thread.collect_users().map(|thread_users| {
                    message_users.extend(thread_users.0);
                    message_users
                })
            })
            .map_or_else(
                |err| {
                    NoUsersFoundInMessageResponseOrThreadResponseSnafu {
                        message_and_thread: self.clone(),
                        previous_err: Box::new(err),
                    }
                    .fail()
                },
                |user_ids| {
                    Ok(user_ids
                        .0
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
        users: Option<&Users>,
    ) -> Result<MessageAndThread> {
        message_and_thread.message =
            Messages::finalize_messages(message_and_thread.message, users)?;
        message_and_thread.thread = Messages::finalize_messages(message_and_thread.thread, users)?;
        Ok(message_and_thread)
    }

    pub fn collect_file_links(&self) -> FileLinks {
        self.thread
            .iter()
            .filter_map(|message| {
                message.files.as_ref().map(|files| {
                    files
                        .iter()
                        .map(|file| {
                            let user_team = file.user_team.clone();
                            let file_id = file.id.clone();
                            (format!("{user_team}-{file_id}"), file.url_private.clone())
                        })
                        .collect::<Vec<(String, String)>>()
                })
            })
            .flatten()
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Display)]
#[display(Debug)]
pub struct MessageResponse {
    pub messages: Option<Vec<Message>>,
    pub ok: Option<bool>,
    pub error: Option<String>,
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
}

impl SlackResponseValidator for MessageResponse {
    fn ok(&self) -> Option<bool> {
        self.ok
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Display, Shrinkwrap, PartialEq, Eq)]
#[display(Debug)]
#[shrinkwrap(mutable)]
pub struct Messages(pub Vec<Message>);

impl CollectUsers<Error> for Messages {
    fn collect_users(&self) -> Result<UserIds> {
        Ok(self
            .iter()
            .map(|message| -> Result<Vec<String>> {
                let message_user = message.user.as_ref().map_or(
                    UserIdWasNoneInMessageSnafu {
                        container: message.to_owned(),
                    }
                    .fail(),
                    |user| Ok(user.to_string()),
                )?;
                let mut reactions_users = message.reactions.as_ref().map_or(vec![], |reactions| {
                    reactions
                        .iter()
                        .flat_map(|reaction| reaction.users.clone())
                        .collect::<Vec<String>>()
                });
                reactions_users.push(message_user);
                Ok(reactions_users)
            })
            .collect::<Result<Vec<Vec<String>>>>()?
            .into_iter()
            .flatten()
            .collect())
    }
}

impl Messages {
    fn finalize_messages(mut messages: Messages, users: Option<&Users>) -> Result<Messages> {
        messages = Messages(
            messages
                .0
                .into_iter()
                .map(|mut message| {
                    message = Message::finalize_message(message, users)?;
                    Ok(message)
                })
                .collect::<Result<Vec<Message>>>()?,
        );
        Ok(messages)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Display, PartialEq, Eq)]
#[display(Debug)]
pub struct Message {
    pub r#type: Option<String>,
    pub user: Option<String>,
    pub user_info: Option<User>,
    pub text: Option<String>,
    pub thread_ts: Option<String>,
    pub reply_count: Option<u16>,
    pub ts: Option<String>,
    pub reactions: Option<Reactions>,
    pub files: Option<Files>,
}

impl Message {
    fn finalize_message(mut message: Message, users: Option<&Users>) -> Result<Message> {
        if let Some(users) = users {
            let user_id = message
                .user
                .as_ref()
                .expect("expected a user id, got None. This should never happen");
            message = if let Some(user) = users.get(user_id) {
                Ok({
                    message.user_info = Some(user.to_owned());
                    message
                })
            } else {
                UserIdNotFoundInUserMapSnafu {
                    user_id,
                    user_map: format!("{:#?}", users),
                }
                .fail()
            }?;

            message.reactions = if let Some(reactions) = message.reactions {
                Some(
                    reactions
                        .0
                        .into_iter()
                        .map(|reaction| Reaction::finalize_reaction(reaction, users))
                        .collect::<Result<Reactions>>()?,
                )
            } else {
                message.reactions
            };
            Ok(message)
        } else {
            Ok(message)
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Display, Shrinkwrap)]
#[display(Debug)]
pub struct Reactions(pub Vec<Reaction>);

impl FromIterator<Reaction> for Reactions {
    fn from_iter<T: IntoIterator<Item = Reaction>>(iter: T) -> Self {
        Reactions(iter.into_iter().collect())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Display)]
#[display(Debug)]
pub struct Reaction {
    pub name: String,
    pub users: Vec<String>,
    pub users_info: Option<Vec<User>>,
    pub count: u16,
}

impl Reaction {
    fn finalize_reaction(mut reaction: Reaction, users: &Users) -> Result<Reaction> {
        reaction.users_info = Some({
            reaction
                .users
                .iter()
                .map(|user_id| {
                    users.get(user_id).map_or(
                        UserIdNotFoundInUserMapSnafu {
                            user_id,
                            user_map: format!("{:#?}", users),
                        }
                        .fail(),
                        |user| Ok(user.clone()),
                    )
                })
                .collect::<Result<Vec<User>>>()?
        });
        Ok(reaction)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Display, Shrinkwrap)]
#[display(Debug)]
pub struct Files(pub Vec<File>);

impl FromIterator<File> for Files {
    fn from_iter<T: IntoIterator<Item = File>>(iter: T) -> Self {
        Files(iter.into_iter().collect())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Display, Shrinkwrap)]
#[display(Debug)]
pub struct FileLinks(pub HashMap<String, String>);

impl FromIterator<(String, String)> for FileLinks {
    fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
        FileLinks(iter.into_iter().collect())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Display, Shrinkwrap)]
#[display(Debug)]
pub struct FilesData(pub HashMap<String, String>);

impl FromIterator<(String, String)> for FilesData {
    fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
        FilesData(iter.into_iter().collect())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Display)]
#[display(Debug)]
pub struct File {
    pub id: String, // use this as filename?
    pub name: String,
    pub user_team: String,
    pub title: String,
    pub mimetype: String,
    pub filetype: String,
    pub size: i64,
    pub url_private: String, // use this to download
    pub url_private_download: String,
    pub permalink: String,
    pub permalink_public: String,
}
