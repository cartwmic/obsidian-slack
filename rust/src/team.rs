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
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Awaiting a JsFuture returned an error: {error}"))]
    WasmErrorFromJsFuture { error: String },

    #[snafu(display("{source}"))]
    SerdeWasmBindgenCouldNotParseTeamResponse { source: response::Error },

    #[snafu(display("The team response was not ok. - source: {source}"))]
    InvalidTeamResponse { source: response::Error },

    #[snafu(display("Could not parse json from team response string - source: {source}"))]
    CouldNotParseJsonFromTeamResponse { source: response::Error },
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub async fn get_teams_from_api<T>(
    team_ids: &Vec<String>,
    client: &SlackHttpClient<T>,
) -> Result<Teams>
where
    wasm_bindgen_futures::JsFuture: std::convert::From<T>,
{
    let teams = team_ids
        .iter()
        .map(|team_id| JsFuture::from(client.get_team_info(team_id)))
        .collect::<Vec<JsFuture>>();

    let team_responses = join_all(teams)
        .await
        .into_iter()
        .map(|result| {
            m! {
                // mapping error instead of using snafu context because jsvalue is not an Error from parse method
                val <- result.map_err(|err| Error::WasmErrorFromJsFuture {
                    error: format!("{:#?}", err),
                });
                js_obj <- convert_result_string_to_object(val).context(CouldNotParseJsonFromTeamResponseSnafu);
                team_response <- response::defined_from_js_object(js_obj).context(SerdeWasmBindgenCouldNotParseTeamResponseSnafu);
                valid_response <- TeamResponse::validate_response(team_response).context(InvalidTeamResponseSnafu);
                return valid_response;
            }
        })
        .collect::<Result<Vec<TeamResponse>>>()?;

    Ok(Teams(
        team_ids
            .iter()
            .map(String::to_string)
            .zip(team_responses.into_iter().map(|team_response| {
                team_response
                    .team
                    .expect("Expected a team in the team response, but got None. This is a bug")
            }))
            .collect::<HashMap<String, Team>>(),
    ))
}

pub trait CollectTeams<T>: Debug + Display
where
    T: snafu::Error,
{
    fn collect_teams(&self) -> std::result::Result<TeamIds, T>;
}

#[derive(Debug, Serialize, Deserialize, Clone, Display, Shrinkwrap, PartialEq, Eq)]
#[display(Debug)]
pub struct TeamIds(pub Vec<String>);

impl From<Vec<String>> for TeamIds {
    fn from(value: Vec<String>) -> Self {
        TeamIds(value)
    }
}

impl FromIterator<String> for TeamIds {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        TeamIds(iter.into_iter().collect())
    }
}

impl DerefMut for TeamIds {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Display, Shrinkwrap)]
#[display(Debug)]
pub struct Teams(pub HashMap<String, Team>);

#[derive(Serialize, Deserialize, Debug)]
pub struct TeamResponse {
    pub ok: Option<bool>,
    pub error: Option<String>,
    pub team: Option<Team>,
}

impl SlackResponseValidator for TeamResponse {
    fn ok(&self) -> Option<bool> {
        self.ok
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Display)]
#[display(Debug)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub domain: Option<String>,
    pub email_domain: Option<String>,
    pub enterprise_id: Option<String>,
    pub enterprise_name: Option<String>,
}
