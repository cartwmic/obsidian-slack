use std::collections::{HashMap, HashSet};

use do_notation::m;
use futures::future::join_all;
use js_sys::Promise;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::JsFuture;

use crate::{
    errors::SlackError,
    slack_http_client::{
        get_api_base, SlackHttpClient, SlackHttpClientConfig, SlackResponseValidator,
    },
    utils::convert_result_string_to_object,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct User {
    pub id: String,
    pub team_id: Option<String>,
    pub name: Option<String>,
    pub real_name: Option<String>,
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

pub async fn get_users_from_api<T>(
    user_ids: &HashSet<String>,
    client: &SlackHttpClient<T>,
) -> Result<HashMap<String, User>, SlackError>
where
    wasm_bindgen_futures::JsFuture: std::convert::From<T>,
{
    let users: Vec<JsFuture> = user_ids
        .iter()
        .map(|user_id| JsFuture::from(client.get_user_info(user_id)))
        .collect();

    let users: Result<Vec<UserResponse>, SlackError> = join_all(users)
        .await
        .into_iter()
        .map(|result| {
            result
                .map_err(SlackError::Js)
                .and_then(convert_result_string_to_object)
                .and_then(|response| {
                    serde_wasm_bindgen::from_value(response)
                        .map_err(SlackError::SerdeWasmBindgen)
                        .and_then(UserResponse::validate_response)
                })
        })
        .collect();

    users.map(|user_responses| {
        user_ids
            .iter()
            .map(String::to_string)
            .zip(
                user_responses
                    .into_iter()
                    .map(|user_response| user_response.user.unwrap()),
            )
            .collect::<HashMap<String, User>>()
    })
}
