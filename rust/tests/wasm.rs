use std::ops::Deref;

use js_sys::JSON;
use obsidian_slack::{
    get_slack_message,
    messages::{Message, MessageResponse},
    slack_http_client::{RequestUrlParam, SlackHttpClientConfigFeatureFlags},
    users::{User, UserResponse},
};
use test_case::test_case;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use wasm_bindgen_test::*;

fn get_mock_request_function(
    message_response: MessageResponse,
    user_response: UserResponse,
) -> JsValue {
    let func_body = format!(
        r#"
        {{
            if (params.url.includes("conversations.replies")) {{
                return Promise.resolve(JSON.stringify({}))
            }}
            else if (params.url.includes("users.info")) {{
                return Promise.resolve(JSON.stringify({}))
            }}
            else {{
                return JSON.stringify({{
                    "ok": false,
                    "error": "endpoint not supported in test for obsidian-slack"
                }})
            }}
        }}
    "#,
        Into::<String>::into(
            JSON::stringify(&serde_wasm_bindgen::to_value(&message_response).unwrap()).unwrap()
        ),
        Into::<String>::into(
            JSON::stringify(&serde_wasm_bindgen::to_value(&user_response).unwrap()).unwrap()
        )
    );

    let func = js_sys::Function::new_with_args("params", &func_body);
    assert_eq!(func.js_typeof(), JsValue::from_str("function"));
    JsValue::from(func)
}

#[test_case(
    MessageResponse {
        is_null: Some(false),
        messages: Some(vec![Message {
            r#type: Some("mock_type".to_string()),
            user: Some("mock_user".to_string()),
            text: Some("mock_text".to_string()),
            thread_ts: Some("0000000000.000000".to_string()),
            reply_count: None,
            team: Some("mock_team".to_string()),
            ts: Some("0000000000.000000".to_string()),
        }]),
        has_more: Some(false),
        is_thread: None,
        ok: Some(true),
        error: None,
        response_metadata: None,
    }, 
    UserResponse {
        ok: Some(true),
        error: None,
        user: Some(User {
            id: "mock_user".to_string(),
            team_id: Some("mock_team".to_string()),
            name: Some("mock_name".to_string()),
            real_name: Some("mock_real_name".to_string()),
        }),
    } ; "when both operands are negative")]
#[wasm_bindgen_test]
async fn get_slack_message_successfully_returns_successfully(
    message_response: MessageResponse,
    user_response: UserResponse,
) {
    let feature_flags = serde_wasm_bindgen::to_value(&SlackHttpClientConfigFeatureFlags {
        get_users: true,
        get_reactions: false,
        get_channel_info: false,
        get_attachments: false,
        get_team_info: false,
    })
    .unwrap();

    let request_func = get_mock_request_function(message_response, user_response);

    let api_token = "xoxc...";
    let cookie = "xoxd...";
    let url = "https://mock.slack.com/archives/C0000000000/p0000000000000000";

    let result = get_slack_message(
        api_token.to_string(),
        cookie.to_string(),
        url.to_string(),
        feature_flags,
        request_func,
    )
    .await;

    assert!(!result.is_string(), "Result was a string: {:#?}", result);
}
