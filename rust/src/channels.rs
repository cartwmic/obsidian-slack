use amplify_derive::Display;
use do_notation::m;
use futures::stream::Collect;
use serde::{Deserialize, Serialize};
use shrinkwraprs::Shrinkwrap;
use snafu::{ResultExt, Snafu};

use crate::{
    messages::Message,
    response::{self, convert_result_string_to_object, SlackResponseValidator},
    slack_http_client::SlackHttpClient,
    users::{CollectUser, User, Users},
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("When mapping user ids from response to retrieved user info, user id was not in user map. user_id: {user_id} - user_map: {user_map}"))]
    UserIdNotFoundInUserMap { user_id: String, user_map: String },

    #[snafu(display("User was none in channel response, indicating this channel is not a direct message: {channel}"))]
    UserInChannelWasNone { channel: Channel },

    #[snafu(display("Awaiting a JsFuture returned an error: {error}"))]
    WasmErrorFromJsFuture { error: String },

    #[snafu(display("The channel response was not ok. - source: {source}"))]
    InvalidChannelResponse { source: response::Error },

    #[snafu(display("{source}"))]
    CouldNotParseJsonFromChannelResponse { source: response::Error },

    #[snafu(display("{source}"))]
    SerdeWasmBindgenCouldNotParseChannelResponse { source: response::Error },
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub async fn get_channel_from_api<T>(
    client: &SlackHttpClient<T>,
    channel_id: &str,
) -> Result<Channel>
where
    wasm_bindgen_futures::JsFuture: std::convert::From<T>,
{
    let awaited_val =
        wasm_bindgen_futures::JsFuture::from(client.get_conversations_info(channel_id))
            .await
            // mapping error instead of using snafu context because jsvalue is not an Error from parse method
            .map_err(|err| Error::WasmErrorFromJsFuture {
                error: format!("{:#?}", err),
            })?;

    let response = m! {
        js_obj <- convert_result_string_to_object(awaited_val).context(CouldNotParseJsonFromChannelResponseSnafu);
        message_response <- response::defined_from_js_object(js_obj).context(SerdeWasmBindgenCouldNotParseChannelResponseSnafu);
        valid_response <- ChannelResponse::validate_response(message_response).context(InvalidChannelResponseSnafu);
        return valid_response;
    }?;

    Ok(response
        .channel
        .expect("Expected Channel but got None, this is a bug"))
}

#[derive(Debug, Serialize, Deserialize, Clone, Display, Shrinkwrap, PartialEq, Eq)]
#[display(Debug)]
pub struct ChannelId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Display)]
#[display(Debug)]
pub struct Channel {
    pub id: Option<String>,
    pub name: Option<String>,
    pub is_channel: Option<bool>,
    pub is_group: Option<bool>,
    pub is_im: Option<bool>,
    pub created: Option<i64>,
    pub creator: Option<String>,
    pub is_archived: Option<bool>,
    pub is_general: Option<bool>,
    pub unlinked: Option<i64>,
    pub name_normalized: Option<String>,
    pub is_read_only: Option<bool>,
    pub is_shared: Option<bool>,
    pub is_member: Option<bool>,
    pub is_private: Option<bool>,
    pub is_mpim: Option<bool>,
    pub last_read: Option<String>,
    pub topic: Option<ChannelAuxData>,
    pub purpose: Option<ChannelAuxData>,
    pub previous_names: Option<Vec<String>>,
    pub locale: Option<String>,
    pub is_org_shared: Option<bool>,
    pub user: Option<String>,
    pub user_info: Option<User>,
    pub latest: Option<Message>,
    pub unread_count: Option<i64>,
    pub unread_count_display: Option<i64>,
    pub is_open: Option<bool>,
    pub priority: Option<f64>,
}

impl CollectUser<Error> for Channel {
    fn collect_users(&self) -> Result<Vec<String>> {
        self.user
            .as_ref()
            .map_or(Ok(vec![]), |user_id| Ok(vec![user_id.to_owned()]))
    }
}

impl Channel {
    pub fn finalize_channel(mut channel: Channel, users: Option<&Users>) -> Result<Channel> {
        match (&channel.user, users) {
            (Some(user_id), Some(users)) => users.get(user_id).map_or(
                UserIdNotFoundInUserMapSnafu {
                    user_id,
                    user_map: format!("{:#?}", users),
                }
                .fail(),
                |user| {
                    Ok({
                        channel.user_info = Some(user.to_owned());
                        channel
                    })
                },
            ),
            _ => Ok(channel),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Display)]
#[display(Debug)]
pub struct ChannelAuxData {
    value: Option<String>,
    creator: Option<String>,
    last_set: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Display)]
#[display(Debug)]
pub struct ChannelResponse {
    pub ok: Option<bool>,
    pub error: Option<String>,
    pub channel: Option<Channel>,
}

impl SlackResponseValidator for ChannelResponse {
    fn ok(&self) -> Option<bool> {
        self.ok
    }
}
