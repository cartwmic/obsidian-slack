use amplify_derive::Display;
use derive_builder::Builder;
use js_sys::Promise;
use snafu::{ResultExt, Snafu};

use crate::{
    channels::{self, Channel},
    components::{self, ObsidianSlackComponents, ObsidianSlackComponentsBuilder},
    messages::{self, MessageAndThread},
    slack_http_client::{SlackHttpClient, SlackHttpClientConfigFeatureFlags},
    slack_url::SlackUrl,
    users::{self, CollectUser, Users},
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("There was a problem finalizing components to save - source {source}"))]
    CouldNotFinalizeComponents { source: components::Error },

    #[snafu(display("Could not get users from api - source: {source}"))]
    CouldNotGetUsersFromApi { source: users::Error },

    #[snafu(display("Could not get messages from api - source: {source}"))]
    CouldNotGetMessagesFromApi { source: messages::Error },

    #[snafu(display("Could not get channel from api - source: {source}"))]
    CouldNotGetChannelFromApi { source: channels::Error },

    #[snafu(display("Could not get users from components - source: {source}"))]
    CouldNotCollectUsersFromComponents { source: components::Error },

    #[snafu(display("Transition from state: {state} with flags {flags} was invalid"))]
    InvalidStateTransition {
        state: ObsidianSlackStates,
        flags: SlackHttpClientConfigFeatureFlags,
    },
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Display, PartialEq, Eq)]
#[display(Debug)]
pub enum ObsidianSlackStates {
    Start,
    MessageAndThread,
    ChannelInfo,
    UserInfo,
    End,
}
#[derive(Debug)]
pub struct ObsidianSlackStateMachineInput<T> {
    pub components: ObsidianSlackComponentsBuilder,
    pub client: SlackHttpClient<T>,
    pub slack_url: SlackUrl,
}
pub struct ObsidianSlackStateMachine;

impl ObsidianSlackStateMachine {
    pub async fn transition(
        state: ObsidianSlackStates,
        input: &mut ObsidianSlackStateMachineInput<Promise>,
    ) -> Result<ObsidianSlackStates> {
        match (&state, &input.client.config.feature_flags) {
            (
                ObsidianSlackStates::Start,
                SlackHttpClientConfigFeatureFlags {
                    get_users: _,
                    get_reactions: _,
                    get_channel_info: _,
                    get_attachments: _,
                    get_team_info: _,
                },
            ) => {
                let message_and_thread =
                    messages::get_messages_from_api(&input.client, &input.slack_url)
                        .await
                        .context(CouldNotGetMessagesFromApiSnafu)?;
                input.components.message_and_thread(message_and_thread);
                Ok(ObsidianSlackStates::MessageAndThread)
            }
            (
                ObsidianSlackStates::MessageAndThread,
                SlackHttpClientConfigFeatureFlags {
                    get_users: false,
                    get_reactions: false,
                    get_channel_info: false,
                    get_attachments: false,
                    get_team_info: false,
                },
            ) => Ok(ObsidianSlackStates::End),
            (
                ObsidianSlackStates::MessageAndThread,
                SlackHttpClientConfigFeatureFlags {
                    get_users: _,
                    get_reactions: _,
                    get_channel_info: true,
                    get_attachments: _,
                    get_team_info: _,
                },
            ) => ObsidianSlackStateMachine::transition_to_channel_info(input).await,
            (
                ObsidianSlackStates::MessageAndThread,
                SlackHttpClientConfigFeatureFlags {
                    get_users: true,
                    get_reactions: _,
                    get_channel_info: false,
                    get_attachments: _,
                    get_team_info: _,
                },
            ) => ObsidianSlackStateMachine::transition_to_user_info(input).await,
            (
                ObsidianSlackStates::ChannelInfo,
                SlackHttpClientConfigFeatureFlags {
                    get_users: false,
                    get_reactions: _,
                    get_channel_info: true,
                    get_attachments: _,
                    get_team_info: _,
                },
            ) => Ok(ObsidianSlackStates::End),
            (
                ObsidianSlackStates::ChannelInfo,
                SlackHttpClientConfigFeatureFlags {
                    get_users: true,
                    get_reactions: _,
                    get_channel_info: true,
                    get_attachments: _,
                    get_team_info: _,
                },
            ) => ObsidianSlackStateMachine::transition_to_user_info(input).await,
            (
                ObsidianSlackStates::UserInfo,
                SlackHttpClientConfigFeatureFlags {
                    get_users: true,
                    get_reactions: _,
                    get_channel_info: _,
                    get_attachments: _,
                    get_team_info: _,
                },
            ) => Ok(ObsidianSlackStates::End),
            (_, _) => InvalidStateTransitionSnafu {
                state,
                flags: input.client.config.feature_flags.clone(),
            }
            .fail(),
        }
    }

    async fn transition_to_user_info(
        input: &mut ObsidianSlackStateMachineInput<Promise>,
    ) -> Result<ObsidianSlackStates> {
        let user_ids = input
            .components
            .collect_users()
            .context(CouldNotCollectUsersFromComponentsSnafu)?;
        let users = users::get_users_from_api(&user_ids, &input.client)
            .await
            .context(CouldNotGetUsersFromApiSnafu)?;
        input.components.users(Some(users));
        Ok(ObsidianSlackStates::UserInfo)
    }

    async fn transition_to_channel_info(
        input: &mut ObsidianSlackStateMachineInput<Promise>,
    ) -> Result<ObsidianSlackStates> {
        let channel = channels::get_channel_from_api(&input.client, &input.slack_url.channel_id)
            .await
            .context(CouldNotGetChannelFromApiSnafu)?;
        input.components.channel(Some(channel));
        Ok(ObsidianSlackStates::ChannelInfo)
    }
}
