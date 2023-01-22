use amplify_derive::Display;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use shrinkwraprs::Shrinkwrap;
use snafu::{ResultExt, Snafu};

use crate::{
    channels::{Channel, ChannelId},
    messages::{self, MessageAndThread},
    users::{CollectUser, UserIds, Users},
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

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq, Display)]
#[display(Debug)]
#[builder(field(public))]
#[builder(derive(Debug))]
pub struct ObsidianSlackComponents {
    pub message_and_thread: MessageAndThread,

    pub file_name: FileName,

    #[builder(default)]
    pub users: Option<Users>,

    #[builder(default)]
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

impl std::fmt::Display for ObsidianSlackComponentsBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl CollectUser<Error> for ObsidianSlackComponentsBuilder {
    fn collect_users(&self) -> Result<Vec<String>> {
        let mut user_ids = self
            .message_and_thread
            .as_ref()
            .expect("expected a message and thread, found None. This is a bug")
            .collect_users()
            .context(CouldNotGetUsersFromMessagesSnafu)?;

        // not getting users from channel
        self.channel
            .as_ref()
            .unwrap_or(&None)
            .as_ref()
            .map_or(Ok::<(), Error>(()), |channel| {
                Ok(user_ids.extend(channel.collect_users().expect("Should always have a vec of users from channel (can be empty), if err than this is a bug")))
            })?;
        Ok(user_ids)
    }
}