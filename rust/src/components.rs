use std::collections::HashMap;

use derive_builder::Builder;
use serde::{Serialize, Deserialize};
use snafu::{Snafu, ResultExt};

use crate::{messages::{MessageAndThread, self}, users::User};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not get users from messages - source: {source}"))]
    CouldNotGetUsersFromMessages { source: messages::Error },
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Clone, Builder, Serialize, Deserialize)]
#[builder(field(public))]
pub struct ObsidianSlackComponents {
    pub message_and_thread: MessageAndThread,
    pub file_name: String,
    pub users: Option<HashMap<String, User>>,
}

impl ObsidianSlackComponents {
    // finalize for saving, replace user ids with object, team id with object, channel id with object, reactions, etc.
    pub fn finalize(mut components: ObsidianSlackComponents) -> Result<ObsidianSlackComponents> {
        // evenetually, self.users.iter.finalize() to add team info, etc.
        components.message_and_thread = MessageAndThread::finalize_message_and_thread(
            components.message_and_thread,
            components.users.as_ref(),
        ).context(CouldNotGetUsersFromMessagesSnafu)?;
        Ok(components)
    }
}
