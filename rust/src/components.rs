use amplify_derive::Display;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use shrinkwraprs::Shrinkwrap;
use snafu::{ResultExt, Snafu};

use crate::{
    channels::{self, Channel},
    messages::{self, FilesData, MessageAndThread},
    team::{CollectTeams, TeamIds, Teams},
    users::{self, CollectUsers, UserIds, Users},
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not get users from messages - source: {source}"))]
    CouldNotGetUsersFromMessages { source: messages::Error },

    #[snafu(display("Could not get users from channel - source: {source}"))]
    CouldNotGetUsersFromChannel { source: channels::Error },

    #[snafu(display("Could not get teams from users - source: {source}"))]
    CouldNotGetTeamsFromUsers { source: users::Error },

    #[snafu(display("Could not finalize messages - source: {source}"))]
    CouldNotFinalizeMesages { source: messages::Error },

    #[snafu(display("Could not finalize channel - source: {source}"))]
    CouldNotFinalizeChannel { source: channels::Error },

    #[snafu(display("Could not finalize users - source: {source}"))]
    CouldNotFinalizeUsers { source: users::Error },
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

    #[builder(default)]
    pub teams: Option<Teams>,

    #[builder(default)]
    pub file_data: Option<FilesData>,
}

impl ObsidianSlackComponents {
    pub fn finalize(mut components: ObsidianSlackComponents) -> Result<ObsidianSlackComponents> {
        components.users = if let Some(users) = components.users {
            if let Some(ref teams) = components.teams {
                Some(Users::finalize_users(users, teams).context(CouldNotFinalizeUsersSnafu)?)
            } else {
                Some(users)
            }
        } else {
            components.users
        };

        components.message_and_thread = MessageAndThread::finalize_message_and_thread(
            components.message_and_thread,
            components.users.as_ref(),
        )
        .context(CouldNotFinalizeMesagesSnafu)?;

        components.channel = if let Some(channel) = components.channel {
            Some(
                Channel::finalize_channel(channel, components.users.as_ref())
                    .context(CouldNotFinalizeChannelSnafu)?,
            )
        } else {
            components.channel
        };
        Ok(components)
    }
}

impl std::fmt::Display for ObsidianSlackComponentsBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl CollectUsers<Error> for ObsidianSlackComponentsBuilder {
    fn collect_users(&self) -> Result<UserIds> {
        let mut user_ids = self
            .message_and_thread
            .as_ref()
            .expect("expected a message and thread, found None. This is a bug")
            .collect_users()
            .context(CouldNotGetUsersFromMessagesSnafu)?;

        self.channel
            .as_ref()
            .unwrap_or(&None)
            .as_ref()
            .map_or(Ok::<(), Error>(()), |channel| {
                user_ids.extend(
                    channel
                        .collect_users()
                        .expect(
                            "Should always have a vec of users from channel \
                            (can be empty), if err than this is a bug",
                        )
                        .0,
                );
                Ok(())
            })?;

        Ok(user_ids)
    }
}

impl CollectTeams<Error> for ObsidianSlackComponentsBuilder {
    fn collect_teams(&self) -> Result<TeamIds> {
        self.users
            .as_ref()
            .unwrap_or(&None)
            .as_ref()
            .expect(
                "If collecting teams, should always have users \
                to collect teams from, if err than this is a bug",
            )
            .collect_teams()
            .context(CouldNotGetTeamsFromUsersSnafu)
    }
}
