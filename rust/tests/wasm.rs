use std::{ops::Deref};

use js_sys::JSON;
use obsidian_slack::{
    get_slack_message,
    ObsidianSlackReturnData,
    messages::{Message, MessageResponse},
    slack_http_client::{RequestUrlParam, SlackHttpClientConfigFeatureFlags},
    users::{User, UserResponse}, MessageAndThreadToSave, MessageToSave,
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
        messages: Some(vec![
            Message {
                r#type: Some("mock_type".to_string()),
                user: Some("mock_user".to_string()),
                text: Some("mock_text".to_string()),
                thread_ts: None,
                reply_count: None,
                team: Some("mock_team".to_string()),
                ts: Some("0000000000.000000".to_string()),
            }
        ]),
        has_more: Some(false),
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
        })
    },
    "https://mock.slack.com/archives/C0000000000/p0000000000000000".to_string(),
    SlackHttpClientConfigFeatureFlags {
        get_users: false,
        get_reactions: false,
        get_channel_info: false,
        get_attachments: false,
        get_team_info: false,
    },
    ObsidianSlackReturnData {
        message_and_thread: MessageAndThreadToSave {
            message: vec![
                MessageToSave {
                    r#type: Some("mock_type".to_string()),
                    user_id: Some("mock_user".to_string()),
                    user: None,
                    text: Some("mock_text".to_string()),
                    thread_ts: None,
                    reply_count: None,
                    team: Some("mock_team".to_string()),
                    ts: Some("0000000000.000000".to_string()),
                }
            ],
            thread: vec![
                MessageToSave {
                    r#type: Some("mock_type".to_string()),
                    user_id: Some("mock_user".to_string()),
                    user: None,
                    text: Some("mock_text".to_string()),
                    thread_ts: None,
                    reply_count: None,
                    team: Some("mock_team".to_string()),
                    ts: Some("0000000000.000000".to_string()),
                }
            ],
            file_name: "C0000000000-0000000000.000000.json".to_string()
        }
    }
    ; "no thread_ts - no flags")]
#[test_case(
    MessageResponse {
        is_null: Some(false),
        messages: Some(vec![
            Message {
                r#type: Some("mock_type".to_string()),
                user: Some("mock_user".to_string()),
                text: Some("mock_text".to_string()),
                thread_ts: None,
                reply_count: None,
                team: Some("mock_team".to_string()),
                ts: Some("0000000000.000000".to_string()),
            }
        ]),
        has_more: Some(false),
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
        })
    },
    "https://mock.slack.com/archives/C0000000000/p0000000000000000".to_string(),
    SlackHttpClientConfigFeatureFlags {
        get_users: true,
        get_reactions: false,
        get_channel_info: false,
        get_attachments: false,
        get_team_info: false,
    },
    ObsidianSlackReturnData {
        message_and_thread: MessageAndThreadToSave {
            message: vec![
                MessageToSave {
                    r#type: Some("mock_type".to_string()),
                    user_id: Some("mock_user".to_string()),
                    user: Some(User {
                        id: "mock_user".to_string(),
                        team_id: Some("mock_team".to_string()),
                        name: Some("mock_name".to_string()),
                        real_name: Some("mock_real_name".to_string()),
                    }),
                    text: Some("mock_text".to_string()),
                    thread_ts: None,
                    reply_count: None,
                    team: Some("mock_team".to_string()),
                    ts: Some("0000000000.000000".to_string()),
                }
            ],
            thread: vec![
                MessageToSave {
                    r#type: Some("mock_type".to_string()),
                    user_id: Some("mock_user".to_string()),
                    user: Some(User {
                        id: "mock_user".to_string(),
                        team_id: Some("mock_team".to_string()),
                        name: Some("mock_name".to_string()),
                        real_name: Some("mock_real_name".to_string()),
                    }),
                    text: Some("mock_text".to_string()),
                    thread_ts: None,
                    reply_count: None,
                    team: Some("mock_team".to_string()),
                    ts: Some("0000000000.000000".to_string()),
                }
            ],
            file_name: "C0000000000-0000000000.000000.json".to_string()
        }
    }
    ; "no thread_ts - user flag")]
#[test_case(
    MessageResponse {
        is_null: Some(false),
        messages: Some(vec![
            Message {
                r#type: Some("mock_type".to_string()),
                user: Some("mock_user".to_string()),
                text: Some("mock_text".to_string()),
                thread_ts: Some("0000000000.000000".to_string()),
                reply_count: None,
                team: Some("mock_team".to_string()),
                ts: Some("0000000000.000000".to_string()),
            },
            Message {
                r#type: Some("mock_type".to_string()),
                user: Some("mock_user".to_string()),
                text: Some("mock_text2".to_string()),
                thread_ts: Some("0000000000.000000".to_string()),
                reply_count: None,
                team: Some("mock_team".to_string()),
                ts: Some("0000000000.000001".to_string()),
            }
        ]),
        has_more: Some(false),
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
        })
    },
    "https://mock.slack.com/archives/C0000000000/p0000000000000000?thread_ts=0000000000.000000".to_string(),
    SlackHttpClientConfigFeatureFlags {
        get_users: false,
        get_reactions: false,
        get_channel_info: false,
        get_attachments: false,
        get_team_info: false,
    },
    ObsidianSlackReturnData {
        message_and_thread: MessageAndThreadToSave {
            message: vec![
                MessageToSave {
                    r#type: Some("mock_type".to_string()),
                    user_id: Some("mock_user".to_string()),
                    user: None,
                    text: Some("mock_text".to_string()),
                    thread_ts: Some("0000000000.000000".to_string()),
                    reply_count: None,
                    team: Some("mock_team".to_string()),
                    ts: Some("0000000000.000000".to_string()),
                }
            ],
            thread: vec![
                MessageToSave {
                    r#type: Some("mock_type".to_string()),
                    user_id: Some("mock_user".to_string()),
                    user: None,
                    text: Some("mock_text".to_string()),
                    thread_ts: Some("0000000000.000000".to_string()),
                    reply_count: None,
                    team: Some("mock_team".to_string()),
                    ts: Some("0000000000.000000".to_string()),
                },
                MessageToSave {
                    r#type: Some("mock_type".to_string()),
                    user_id: Some("mock_user".to_string()),
                    user: None,
                    text: Some("mock_text2".to_string()),
                    thread_ts: Some("0000000000.000000".to_string()),
                    reply_count: None,
                    team: Some("mock_team".to_string()),
                    ts: Some("0000000000.000001".to_string()),
                }
            ],
            file_name: "C0000000000-0000000000.000000.json".to_string()
        }
    }
    ; "thread_ts - thread_ts and ts the same - no flags")]
#[test_case(
    MessageResponse {
        is_null: Some(false),
        messages: Some(vec![
            Message {
                r#type: Some("mock_type".to_string()),
                user: Some("mock_user".to_string()),
                text: Some("mock_text".to_string()),
                thread_ts: Some("0000000000.000000".to_string()),
                reply_count: None,
                team: Some("mock_team".to_string()),
                ts: Some("0000000000.000000".to_string()),
            },
            Message {
                r#type: Some("mock_type".to_string()),
                user: Some("mock_user".to_string()),
                text: Some("mock_text2".to_string()),
                thread_ts: Some("0000000000.000000".to_string()),
                reply_count: None,
                team: Some("mock_team".to_string()),
                ts: Some("0000000000.000001".to_string()),
            }
        ]),
        has_more: Some(false),
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
        })
    },
    "https://mock.slack.com/archives/C0000000000/p0000000000000000?thread_ts=0000000000.000000".to_string(),
    SlackHttpClientConfigFeatureFlags {
        get_users: true,
        get_reactions: false,
        get_channel_info: false,
        get_attachments: false,
        get_team_info: false,
    },
    ObsidianSlackReturnData {
        message_and_thread: MessageAndThreadToSave {
            message: vec![
                MessageToSave {
                    r#type: Some("mock_type".to_string()),
                    user_id: Some("mock_user".to_string()),
                    user: Some(User {
                        id: "mock_user".to_string(),
                        team_id: Some("mock_team".to_string()),
                        name: Some("mock_name".to_string()),
                        real_name: Some("mock_real_name".to_string()),
                    }),
                    text: Some("mock_text".to_string()),
                    thread_ts: Some("0000000000.000000".to_string()),
                    reply_count: None,
                    team: Some("mock_team".to_string()),
                    ts: Some("0000000000.000000".to_string()),
                }
            ],
            thread: vec![
                MessageToSave {
                    r#type: Some("mock_type".to_string()),
                    user_id: Some("mock_user".to_string()),
                    user: Some(User {
                        id: "mock_user".to_string(),
                        team_id: Some("mock_team".to_string()),
                        name: Some("mock_name".to_string()),
                        real_name: Some("mock_real_name".to_string()),
                    }),
                    text: Some("mock_text".to_string()),
                    thread_ts: Some("0000000000.000000".to_string()),
                    reply_count: None,
                    team: Some("mock_team".to_string()),
                    ts: Some("0000000000.000000".to_string()),
                },
                MessageToSave {
                    r#type: Some("mock_type".to_string()),
                    user_id: Some("mock_user".to_string()),
                    user: Some(User {
                        id: "mock_user".to_string(),
                        team_id: Some("mock_team".to_string()),
                        name: Some("mock_name".to_string()),
                        real_name: Some("mock_real_name".to_string()),
                    }),
                    text: Some("mock_text2".to_string()),
                    thread_ts: Some("0000000000.000000".to_string()),
                    reply_count: None,
                    team: Some("mock_team".to_string()),
                    ts: Some("0000000000.000001".to_string()),
                }
            ],
            file_name: "C0000000000-0000000000.000000.json".to_string()
        }
    }
    ; "thread_ts - thread_ts and ts the same - user flag")]
#[test_case(
    MessageResponse {
        is_null: Some(false),
        messages: Some(vec![
            Message {
                r#type: Some("mock_type".to_string()),
                user: Some("mock_user".to_string()),
                text: Some("mock_text".to_string()),
                thread_ts: Some("0000000000.000000".to_string()),
                reply_count: None,
                team: Some("mock_team".to_string()),
                ts: Some("0000000000.000000".to_string()),
            },
            Message {
                r#type: Some("mock_type".to_string()),
                user: Some("mock_user".to_string()),
                text: Some("mock_text2".to_string()),
                thread_ts: Some("0000000000.000000".to_string()),
                reply_count: None,
                team: Some("mock_team".to_string()),
                ts: Some("0000000000.000001".to_string()),
            }
        ]),
        has_more: Some(false),
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
        })
    },
    "https://mock.slack.com/archives/C0000000000/p0000000000000001?thread_ts=0000000000.000000".to_string(),
    SlackHttpClientConfigFeatureFlags {
        get_users: false,
        get_reactions: false,
        get_channel_info: false,
        get_attachments: false,
        get_team_info: false,
    },
    ObsidianSlackReturnData {
        message_and_thread: MessageAndThreadToSave {
            message: vec![
                MessageToSave {
                    r#type: Some("mock_type".to_string()),
                    user_id: Some("mock_user".to_string()),
                    user: None,
                    text: Some("mock_text2".to_string()),
                    thread_ts: Some("0000000000.000000".to_string()),
                    reply_count: None,
                    team: Some("mock_team".to_string()),
                    ts: Some("0000000000.000001".to_string()),
                }
            ],
            thread: vec![
                MessageToSave {
                    r#type: Some("mock_type".to_string()),
                    user_id: Some("mock_user".to_string()),
                    user: None,
                    text: Some("mock_text".to_string()),
                    thread_ts: Some("0000000000.000000".to_string()),
                    reply_count: None,
                    team: Some("mock_team".to_string()),
                    ts: Some("0000000000.000000".to_string()),
                },
                MessageToSave {
                    r#type: Some("mock_type".to_string()),
                    user_id: Some("mock_user".to_string()),
                    user: None,
                    text: Some("mock_text2".to_string()),
                    thread_ts: Some("0000000000.000000".to_string()),
                    reply_count: None,
                    team: Some("mock_team".to_string()),
                    ts: Some("0000000000.000001".to_string()),
                }
            ],
            file_name: "C0000000000-0000000000.000000-0000000000.000001.json".to_string()
        }
    }
    ; "thread_ts - thread_ts and ts not the same - no flags")]
#[test_case(
    MessageResponse {
        is_null: Some(false),
        messages: Some(vec![
            Message {
                r#type: Some("mock_type".to_string()),
                user: Some("mock_user".to_string()),
                text: Some("mock_text".to_string()),
                thread_ts: Some("0000000000.000000".to_string()),
                reply_count: None,
                team: Some("mock_team".to_string()),
                ts: Some("0000000000.000000".to_string()),
            },
            Message {
                r#type: Some("mock_type".to_string()),
                user: Some("mock_user".to_string()),
                text: Some("mock_text2".to_string()),
                thread_ts: Some("0000000000.000000".to_string()),
                reply_count: None,
                team: Some("mock_team".to_string()),
                ts: Some("0000000000.000001".to_string()),
            }
        ]),
        has_more: Some(false),
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
        })
    },
    "https://mock.slack.com/archives/C0000000000/p0000000000000001?thread_ts=0000000000.000000".to_string(),
    SlackHttpClientConfigFeatureFlags {
        get_users: true,
        get_reactions: false,
        get_channel_info: false,
        get_attachments: false,
        get_team_info: false,
    },
    ObsidianSlackReturnData {
        message_and_thread: MessageAndThreadToSave {
            message: vec![
                MessageToSave {
                    r#type: Some("mock_type".to_string()),
                    user_id: Some("mock_user".to_string()),
                    user: Some(User {
                        id: "mock_user".to_string(),
                        team_id: Some("mock_team".to_string()),
                        name: Some("mock_name".to_string()),
                        real_name: Some("mock_real_name".to_string()),
                    }),
                    text: Some("mock_text2".to_string()),
                    thread_ts: Some("0000000000.000000".to_string()),
                    reply_count: None,
                    team: Some("mock_team".to_string()),
                    ts: Some("0000000000.000001".to_string()),
                }
            ],
            thread: vec![
                MessageToSave {
                    r#type: Some("mock_type".to_string()),
                    user_id: Some("mock_user".to_string()),
                    user: Some(User {
                        id: "mock_user".to_string(),
                        team_id: Some("mock_team".to_string()),
                        name: Some("mock_name".to_string()),
                        real_name: Some("mock_real_name".to_string()),
                    }),
                    text: Some("mock_text".to_string()),
                    thread_ts: Some("0000000000.000000".to_string()),
                    reply_count: None,
                    team: Some("mock_team".to_string()),
                    ts: Some("0000000000.000000".to_string()),
                },
                MessageToSave {
                    r#type: Some("mock_type".to_string()),
                    user_id: Some("mock_user".to_string()),
                    user: Some(User {
                        id: "mock_user".to_string(),
                        team_id: Some("mock_team".to_string()),
                        name: Some("mock_name".to_string()),
                        real_name: Some("mock_real_name".to_string()),
                    }),
                    text: Some("mock_text2".to_string()),
                    thread_ts: Some("0000000000.000000".to_string()),
                    reply_count: None,
                    team: Some("mock_team".to_string()),
                    ts: Some("0000000000.000001".to_string()),
                }
            ],
            file_name: "C0000000000-0000000000.000000-0000000000.000001.json".to_string()
        }
    }
    ; "thread_ts - thread_ts and ts not the same - user flag")]
#[wasm_bindgen_test]
async fn get_slack_message_returns_data_correctly(
    message_response: MessageResponse,
    user_response: UserResponse,
    url: String,
    feature_flags: SlackHttpClientConfigFeatureFlags,
    expected_return_data: ObsidianSlackReturnData
) {
    let feature_flags = serde_wasm_bindgen::to_value(&feature_flags)
    .unwrap();

    let request_func = get_mock_request_function(message_response, user_response);

    let api_token = "xoxc...";
    let cookie = "xoxd...";

    let result = get_slack_message(
        api_token.to_string(),
        cookie.to_string(),
        url,
        feature_flags,
        request_func,
    )
    .await;

    assert!(!result.is_string(), "Result was a string: {:#?}", result);

    let result: ObsidianSlackReturnData = serde_wasm_bindgen::from_value(result).expect("Should parse return object");

    assert_eq!(expected_return_data, result);
}

// user response not ok - user flag
#[test_case(
    None, 
    None, 
    "bad_token".to_string(), 
    "xoxd...".to_string(), 
    "https://mock.slack.com/archives/C0000000000/p0000000000000000?thread_ts=0000000000.000000".to_string(), 
    SlackHttpClientConfigFeatureFlags {
        get_users: true,
        get_reactions: false,
        get_channel_info: false,
        get_attachments: false,
        get_team_info: false,
    }
    ; "bad api token"
)]
#[test_case(
    None, 
    None, 
    "xoxc...".to_string(), 
    "bad cookie".to_string(), 
    "https://mock.slack.com/archives/C0000000000/p0000000000000000?thread_ts=0000000000.000000".to_string(), 
    SlackHttpClientConfigFeatureFlags {
        get_users: true,
        get_reactions: false,
        get_channel_info: false,
        get_attachments: false,
        get_team_info: false,
    }
    ; "bad cookie"
)]
#[test_case(
    None, 
    None, 
    "xoxc...".to_string(), 
    "xoxd...".to_string(), 
    "https://mock.slack.com/archives/bad_channel_id/p0000000000000000?thread_ts=0000000000.000000".to_string(), 
    SlackHttpClientConfigFeatureFlags {
        get_users: true,
        get_reactions: false,
        get_channel_info: false,
        get_attachments: false,
        get_team_info: false,
    }
    ; "bad channel_id"
)]
#[test_case(
    None, 
    None, 
    "xoxc...".to_string(), 
    "xoxd...".to_string(), 
    "https://mock.slack.com/archives/C0000000000/bad_ts?thread_ts=0000000000.000000".to_string(), 
    SlackHttpClientConfigFeatureFlags {
        get_users: true,
        get_reactions: false,
        get_channel_info: false,
        get_attachments: false,
        get_team_info: false,
    }
    ; "bad ts"
)]
#[test_case(
    Some(MessageResponse {
        is_null: Some(false),
        messages: None,
        has_more: Some(false),
        ok: Some(false),
        error: None,
        response_metadata: None,
    }), 
    None, 
    "xoxc...".to_string(), 
    "xoxd...".to_string(), 
    "https://mock.slack.com/archives/C0000000000/p0000000000000000?thread_ts=not_a_good_thread_ts".to_string(), 
    SlackHttpClientConfigFeatureFlags {
        get_users: true,
        get_reactions: false,
        get_channel_info: false,
        get_attachments: false,
        get_team_info: false,
    }
    ; "message response not ok - no flags"
)]
#[test_case(
    Some(MessageResponse {
        is_null: Some(false),
        messages: Some(vec![
            Message {
                r#type: Some("mock_type".to_string()),
                user: Some("mock_user".to_string()),
                text: Some("mock_text".to_string()),
                thread_ts: None,
                reply_count: None,
                team: Some("mock_team".to_string()),
                ts: Some("0000000000.000000".to_string()),
            }
        ]),
        has_more: Some(false),
        ok: Some(true),
        error: None,
        response_metadata: None,
    }), 
    Some(UserResponse {
        ok: Some(false),
        error: None,
        user: None
    }),
    "xoxc...".to_string(), 
    "xoxd...".to_string(), 
    "https://mock.slack.com/archives/C0000000000/p0000000000000000?thread_ts=not_a_good_thread_ts".to_string(), 
    SlackHttpClientConfigFeatureFlags {
        get_users: true,
        get_reactions: false,
        get_channel_info: false,
        get_attachments: false,
        get_team_info: false,
    }
    ; "message response not ok - user flag"
)]
#[wasm_bindgen_test]
async fn get_slack_message_returns_error_messages_correctly(
    message_response: Option<MessageResponse>,
    user_response: Option<UserResponse>,
    api_token: String,
    cookie: String,
    url: String,
    feature_flags: SlackHttpClientConfigFeatureFlags,
) {
    let feature_flags = serde_wasm_bindgen::to_value(&feature_flags)
    .unwrap();

    let request_func = get_mock_request_function(message_response.unwrap_or(
        MessageResponse {
            is_null: Some(false),
            messages: Some(vec![
                Message {
                    r#type: Some("mock_type".to_string()),
                    user: Some("mock_user".to_string()),
                    text: Some("mock_text".to_string()),
                    thread_ts: Some("0000000000.000000".to_string()),
                    reply_count: None,
                    team: Some("mock_team".to_string()),
                    ts: Some("0000000000.000000".to_string()),
                },
            ]),
            has_more: Some(false),
            ok: Some(true),
            error: None,
            response_metadata: None,
        }
    ), 
    user_response.unwrap_or(
        UserResponse {
            ok: Some(true),
            error: None,
            user: Some(User {
                id: "mock_user".to_string(),
                team_id: Some("mock_team".to_string()),
                name: Some("mock_name".to_string()),
                real_name: Some("mock_real_name".to_string()),
            })
        }
    ));

    let result = get_slack_message(
        api_token.to_string(),
        cookie.to_string(),
        url,
        feature_flags,
        request_func,
    )
    .await;

    assert!(result.is_string(), "Result was not a string: {:#?}", result);
}