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
    users::CollectUser,
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("User was none in channel response, indicating this channel is not a direct message: {channel}"))]
    UserInChannelWasNone { channel: Channel },

    #[snafu(display("Awaiting a JsFuture returned an error: {error}"))]
    WasmErrorFromJsFuture { error: String },

    #[snafu(display("The message response was not ok. - source: {source}"))]
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
    id: Option<String>,
    name: Option<String>,
    is_channel: Option<bool>,
    is_group: Option<bool>,
    is_im: Option<bool>,
    created: Option<i64>,
    creator: Option<String>,
    is_archived: Option<bool>,
    is_general: Option<bool>,
    unlinked: Option<i64>,
    name_normalized: Option<String>,
    is_read_only: Option<bool>,
    is_shared: Option<bool>,
    is_member: Option<bool>,
    is_private: Option<bool>,
    is_mpim: Option<bool>,
    last_read: Option<String>,
    topic: Option<ChannelAuxData>,
    purpose: Option<ChannelAuxData>,
    previous_names: Option<Vec<String>>,
    locale: Option<String>,
    is_org_shared: Option<bool>,
    user: Option<String>,
    latest: Option<Message>,
    unread_count: Option<i64>,
    unread_count_display: Option<i64>,
    is_open: Option<bool>,
    priority: Option<f64>,
}

impl CollectUser<Error> for Channel {
    fn collect_users(&self) -> Result<Vec<String>> {
        self.user
            .as_ref()
            .map_or(Ok(vec![]), |user_id| Ok(vec![user_id.to_owned()]))
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
