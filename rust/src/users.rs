use amplify_derive::Display;
use do_notation::m;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use shrinkwraprs::Shrinkwrap;
use snafu::{ResultExt, Snafu};
use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
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
    SerdeWasmBindgenCouldNotParseUserResponse { source: response::Error },

    #[snafu(display("The user response was not ok. - source: {source}"))]
    InvalidUserResponse { source: response::Error },

    #[snafu(display("Could not parse json from user response string - source: {source}"))]
    CouldNotParseJsonFromUserResponse { source: response::Error },
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
        .map(|user_id| JsFuture::from(client.get_user_info(user_id)))
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

pub trait CollectUser<T>: Debug + Display
where
    T: snafu::Error,
{
    fn collect_users(&self) -> std::result::Result<Vec<String>, T>;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Display)]
#[display(Debug)]
pub struct User {
    pub id: String,
    pub team_id: Option<String>,
    pub name: Option<String>,
    pub real_name: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Display, Shrinkwrap)]
#[display(Debug)]
pub struct Users(pub HashMap<String, User>);

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
