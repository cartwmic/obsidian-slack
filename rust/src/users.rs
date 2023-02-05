use amplify_derive::Display;
use do_notation::m;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use shrinkwraprs::Shrinkwrap;
use snafu::{ResultExt, Snafu};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    iter::FromIterator,
    ops::DerefMut,
};
use wasm_bindgen_futures::JsFuture;

use crate::{
    response::{self, convert_result_string_to_object, SlackResponseValidator},
    slack_http_client::SlackHttpClient,
    team::{CollectTeams, Team, TeamIds, Teams},
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Awaiting a JsFuture returned an error: {error}"))]
    WasmErrorFromJsFuture { error: String },

    #[snafu(display("{source}"))]
    SerdeWasmBindgenCouldNotParseUserResponse { source: response::Error },

    #[snafu(display("The user response was not ok. - source: {source}"))]
    InvalidUserResponse { source: response::Error },

    #[snafu(display("Could not parse json from user response string - source: {source}"))]
    CouldNotParseJsonFromUserResponse { source: response::Error },

    #[snafu(display(
        "Attempted to retrieve the team id for the user, but found none: {container}"
    ))]
    TeamIdWasNoneInUser { container: User },

    #[snafu(display("When mapping team ids from response to retrieved team info, team id was not in team map. team_id: {team_id} - team_map: {team_map}"))]
    TeamIdNotFoundInTeamMap { team_id: String, team_map: String },
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub async fn get_users_from_api<T>(
    user_ids: &Vec<String>,
    client: &SlackHttpClient<T>,
) -> Result<Users>
where
    wasm_bindgen_futures::JsFuture: std::convert::From<T>,
{
    let users = user_ids
        .iter()
        .map(|user_id| JsFuture::from(client.get_users_info(user_id)))
        .collect::<Vec<JsFuture>>();

    let user_responses = join_all(users)
        .await
        .into_iter()
        .map(|result| {
            m! {
                // mapping error instead of using snafu context because jsvalue is not an Error from parse method
                val <- result.map_err(|err| Error::WasmErrorFromJsFuture {
                    error: format!("{:#?}", err),
                });
                js_obj <- convert_result_string_to_object(val).context(CouldNotParseJsonFromUserResponseSnafu);
                user_response <- response::defined_from_js_object(js_obj).context(SerdeWasmBindgenCouldNotParseUserResponseSnafu);
                valid_response <- UserResponse::validate_response(user_response).context(InvalidUserResponseSnafu);
                return valid_response;
            }
        })
        .collect::<Result<Vec<UserResponse>>>()?;

    Ok(Users(
        user_ids
            .iter()
            .map(String::to_string)
            .zip(user_responses.into_iter().map(|user_response| {
                user_response
                    .user
                    .expect("Expected a user in the user response, but got None. This is a bug")
            }))
            .collect::<HashMap<String, User>>(),
    ))
}

pub trait CollectUsers<T>: Debug + Display
where
    T: snafu::Error,
{
    fn collect_users(&self) -> std::result::Result<UserIds, T>;
}

#[derive(Debug, Serialize, Deserialize, Clone, Display, Shrinkwrap, PartialEq, Eq)]
#[display(Debug)]
pub struct UserIds(pub Vec<String>);

impl From<Vec<String>> for UserIds {
    fn from(value: Vec<String>) -> Self {
        UserIds(value)
    }
}

impl FromIterator<String> for UserIds {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        UserIds(iter.into_iter().collect())
    }
}

impl DerefMut for UserIds {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Display)]
#[display(Debug)]
pub struct User {
    pub id: String,
    pub team_id: Option<String>,
    pub team_info: Option<Team>,
    pub name: Option<String>,
    pub real_name: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Display, Shrinkwrap)]
#[display(Debug)]
pub struct Users(pub HashMap<String, User>);

impl CollectTeams<Error> for Users {
    fn collect_teams(&self) -> Result<TeamIds> {
        self.iter()
            .map(|(_, user)| {
                user.team_id.as_ref().map_or(
                    TeamIdWasNoneInUserSnafu {
                        container: user.clone(),
                    }
                    .fail(),
                    |team_id| Ok(team_id.to_owned()),
                )
            })
            .collect()
    }
}

impl FromIterator<(String, User)> for Users {
    fn from_iter<T: IntoIterator<Item = (String, User)>>(iter: T) -> Self {
        Users(iter.into_iter().collect())
    }
}

impl Users {
    pub fn finalize_users(users: Users, teams: &Teams) -> Result<Users> {
        users
            .0
            .into_iter()
            .map(|(user_id, mut user)| {
                if let Some(team_id) = user.team_id.as_ref() {
                    if let Some(team) = teams.get(team_id) {
                        user.team_info = Some(team.to_owned());
                        Ok((user_id, user))
                    } else {
                        TeamIdNotFoundInTeamMapSnafu {
                            team_id,
                            team_map: format!("{:#?}", teams),
                        }
                        .fail()
                    }
                } else {
                    TeamIdWasNoneInUserSnafu {
                        container: user.clone(),
                    }
                    .fail()
                }
            })
            .collect()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserResponse {
    pub ok: Option<bool>,
    pub error: Option<String>,
    pub user: Option<User>,
}

impl SlackResponseValidator for UserResponse {
    fn ok(&self) -> Option<bool> {
        self.ok
    }
}
