use std::collections::HashMap;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};

use crate::{
    messages::{self, MessageAndThread},
    users::Users,
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not get users from messages - source: {source}"))]
    CouldNotGetUsersFromMessages { source: messages::Error },
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq, Eq)]
#[builder(field(public))]
pub struct ObsidianSlackComponents {
    pub message_and_thread: MessageAndThread,
    pub file_name: String,
    pub users: Option<Users>,
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
