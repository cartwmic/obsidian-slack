use amplify_derive::Display;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use shrinkwraprs::Shrinkwrap;
use snafu::{ResultExt, Snafu};

use crate::{
    channels::{Channel, ChannelId},
    messages::{self, MessageAndThread},
    users::{UserIds, Users},
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not get users from messages - source: {source}"))]
    CouldNotGetUsersFromMessages { source: messages::Error },
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Serialize, Deserialize, Clone, Display, Shrinkwrap, PartialEq, Eq)]
#[display(Debug)]
pub struct FileName(pub String);

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
#[builder(field(public))]
pub struct ObsidianSlackComponents {
    pub message_and_thread: MessageAndThread,
    pub file_name: FileName,
    pub channel_id: ChannelId,
    pub user_ids: UserIds,
    pub users: Option<Users>,
    pub channel: Option<Channel>,
}

impl ObsidianSlackComponents {
    // finalize for saving, replace user ids with object, team id with object, channel id with object, reactions, etc.
    pub fn finalize(mut components: ObsidianSlackComponents) -> Result<ObsidianSlackComponents> {
        // evenetually, self.users.iter.finalize() to add team info, etc.
        components.message_and_thread = MessageAndThread::finalize_message_and_thread(
            components.message_and_thread,
            components.users.as_ref(),
        )
        .context(CouldNotGetUsersFromMessagesSnafu)?;
        Ok(components)
    }
}
