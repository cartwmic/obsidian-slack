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
    utils::make_request,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    id: String,
    team_id: Option<String>,
    name: Option<String>,
    real_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserResponse {
    ok: Option<bool>,
    error: Option<String>,
    user: Option<User>,
}

impl SlackResponseValidator for UserResponse {
    fn ok(&self) -> Option<bool> {
        self.ok
    }
}

pub async fn get_users_from_api(
    user_ids: &HashSet<String>,
    api_token: &str,
    cookie: &str,
) -> Result<HashMap<String, User>, SlackError> {
    let users: Result<Vec<JsFuture>, SlackError> = m! {
        config <- SlackHttpClientConfig::new(get_api_base(), api_token.to_string(), cookie.to_string());
        let client = SlackHttpClient::<Promise>::new(config, make_request);
        let users = user_ids.iter().map(|user_id| JsFuture::from(client.get_user_info(user_id))).collect();
        return (users);
    };

    let users: Result<Vec<UserResponse>, SlackError> = match users {
        Ok(users) => join_all(users)
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
            .collect(),
        Err(err) => Err(err),
    };

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
