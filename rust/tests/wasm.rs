use std::collections::HashMap;

use js_sys::JSON;
use obsidian_slack::{
    channels::{Channel, ChannelResponse},
    components::{FileName, ObsidianSlackComponents},
    get_slack_message,
    messages::{Message, MessageAndThread, MessageResponse, Messages},
    slack_http_client::SlackHttpClientConfigFeatureFlags,
    users::{User, UserResponse, Users},
};
use test_case::test_case;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::{console_log, *};

fn get_mock_request_function(
    message_response: MessageResponse,
    user_response: UserResponse,
    channel_response: ChannelResponse
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
            else if (params.url.includes("conversations.info")) {{
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
        ),
        Into::<String>::into(
            JSON::stringify(&serde_wasm_bindgen::to_value(&channel_response).unwrap()).unwrap()
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
                user_info: None,
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
    ChannelResponse {
        error: None,
        ok: Some(true),
        channel: None 
    },
    "https://mock.slack.com/archives/C0000000000/p0000000000000000".to_string(),
    SlackHttpClientConfigFeatureFlags {
        get_users: false,
        get_reactions: false,
        get_channel_info: false,
        get_attachments: false,
        get_team_info: false,
    },
    ObsidianSlackComponents {
        message_and_thread: MessageAndThread {
            message: Messages (
                vec![
                    Message {
                        r#type: Some("mock_type".to_string()),
                        user: Some("mock_user".to_string()),
                        user_info: None,
                        text: Some("mock_text".to_string()),
                        thread_ts: None,
                        reply_count: None,
                        team: Some("mock_team".to_string()),
                        ts: Some("0000000000.000000".to_string()),
                    }
                ]
            ),
            thread: Messages (
                vec![
                    Message {
                        r#type: Some("mock_type".to_string()),
                        user: Some("mock_user".to_string()),
                        user_info: None,
                        text: Some("mock_text".to_string()),
                        thread_ts: None,
                        reply_count: None,
                        team: Some("mock_team".to_string()),
                        ts: Some("0000000000.000000".to_string()),
                    }
                ]
            ),
        },
        file_name: FileName("C0000000000-0000000000.000000.json".to_string()),
        users: None,
        channel: None
    }
    ; "no thread_ts - no flags")]
#[test_case(
    MessageResponse {
        is_null: Some(false),
        messages: Some(vec![
            Message {
                r#type: Some("mock_type".to_string()),
                user: Some("mock_user".to_string()),
                user_info: None,
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
    ChannelResponse {
        error: None,
        ok: Some(true),
        channel: None 
    },
    "https://mock.slack.com/archives/C0000000000/p0000000000000000".to_string(),
    SlackHttpClientConfigFeatureFlags {
        get_users: true,
        get_reactions: false,
        get_channel_info: false,
        get_attachments: false,
        get_team_info: false,
    },
    ObsidianSlackComponents {
        message_and_thread: MessageAndThread {
            message: Messages(
                vec![
                    Message {
                        r#type: Some("mock_type".to_string()),
                        user: Some("mock_user".to_string()),
                        user_info: Some(User {
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
            ),
            thread: Messages(
                vec![
                    Message {
                        r#type: Some("mock_type".to_string()),
                        user: Some("mock_user".to_string()),
                        user_info: Some(User {
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
            ),
        },
        file_name: FileName("C0000000000-0000000000.000000.json".to_string()),
        users: Some(Users(
            [(
                "mock_user".to_owned(),
                User {
                    id: "mock_user".to_string(),
                    team_id: Some("mock_team".to_string()),
                    name: Some("mock_name".to_string()),
                    real_name: Some("mock_real_name".to_string()),
                }
            )].iter().cloned().collect()
        )),
        channel: None
    }
    ; "no thread_ts - user flag")]
#[test_case(
    MessageResponse {
        is_null: Some(false),
        messages: Some(vec![
            Message {
                r#type: Some("mock_type".to_string()),
                user: Some("mock_user".to_string()),
                user_info: None,
                text: Some("mock_text".to_string()),
                thread_ts: Some("0000000000.000000".to_string()),
                reply_count: None,
                team: Some("mock_team".to_string()),
                ts: Some("0000000000.000000".to_string()),
            },
            Message {
                r#type: Some("mock_type".to_string()),
                user: Some("mock_user".to_string()),
                user_info: None,
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
    ChannelResponse {
        error: None,
        ok: Some(true),
        channel: None 
    },
    "https://mock.slack.com/archives/C0000000000/p0000000000000000?thread_ts=0000000000.000000".to_string(),
    SlackHttpClientConfigFeatureFlags {
        get_users: false,
        get_reactions: false,
        get_channel_info: false,
        get_attachments: false,
        get_team_info: false,
    },
    ObsidianSlackComponents {
        message_and_thread: MessageAndThread {
            message: Messages(
                vec![
                    Message {
                        r#type: Some("mock_type".to_string()),
                        user: Some("mock_user".to_string()),
                        user_info: None,
                        text: Some("mock_text".to_string()),
                        thread_ts: Some("0000000000.000000".to_string()),
                        reply_count: None,
                        team: Some("mock_team".to_string()),
                        ts: Some("0000000000.000000".to_string()),
                    }
                ]
            ),
            thread: Messages(
                vec![
                    Message {
                        r#type: Some("mock_type".to_string()),
                        user: Some("mock_user".to_string()),
                        user_info: None,
                        text: Some("mock_text".to_string()),
                        thread_ts: Some("0000000000.000000".to_string()),
                        reply_count: None,
                        team: Some("mock_team".to_string()),
                        ts: Some("0000000000.000000".to_string()),
                    },
                    Message {
                        r#type: Some("mock_type".to_string()),
                        user: Some("mock_user".to_string()),
                        user_info: None,
                        text: Some("mock_text2".to_string()),
                        thread_ts: Some("0000000000.000000".to_string()),
                        reply_count: None,
                        team: Some("mock_team".to_string()),
                        ts: Some("0000000000.000001".to_string()),
                    }
                ]
            ),
        },
        file_name: FileName("C0000000000-0000000000.000000.json".to_string()),
        users: None,
        channel: None
    }
    ; "thread_ts - thread_ts and ts the same - no flags")]
#[test_case(
    MessageResponse {
        is_null: Some(false),
        messages: Some(vec![
            Message {
                r#type: Some("mock_type".to_string()),
                user: Some("mock_user".to_string()),
                user_info: None,
                text: Some("mock_text".to_string()),
                thread_ts: Some("0000000000.000000".to_string()),
                reply_count: None,
                team: Some("mock_team".to_string()),
                ts: Some("0000000000.000000".to_string()),
            },
            Message {
                r#type: Some("mock_type".to_string()),
                user: Some("mock_user".to_string()),
                user_info: None,
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
    ChannelResponse {
        error: None,
        ok: Some(true),
        channel: None 
    },
    "https://mock.slack.com/archives/C0000000000/p0000000000000000?thread_ts=0000000000.000000".to_string(),
    SlackHttpClientConfigFeatureFlags {
        get_users: true,
        get_reactions: false,
        get_channel_info: false,
        get_attachments: false,
        get_team_info: false,
    },
    ObsidianSlackComponents {
        message_and_thread: MessageAndThread {
            message: Messages(
                vec![
                    Message {
                        r#type: Some("mock_type".to_string()),
                        user: Some("mock_user".to_string()),
                        user_info: Some(User {
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
                ]
            ),
            thread: Messages(
                vec![
                    Message {
                        r#type: Some("mock_type".to_string()),
                        user: Some("mock_user".to_string()),
                        user_info: Some(User {
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
                    Message {
                        r#type: Some("mock_type".to_string()),
                        user: Some("mock_user".to_string()),
                        user_info: Some(User {
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
                ]
            ),
        },
        file_name: FileName("C0000000000-0000000000.000000.json".to_string()),
        channel: None,
        users: Some(Users(
            [(
                "mock_user".to_owned(),
                User {
                    id: "mock_user".to_string(),
                    team_id: Some("mock_team".to_string()),
                    name: Some("mock_name".to_string()),
                    real_name: Some("mock_real_name".to_string()),
                }
            )].iter().cloned().collect()
        ))
    }
    ; "thread_ts - thread_ts and ts the same - user flag")]
#[test_case(
    MessageResponse {
        is_null: Some(false),
        messages: Some(vec![
            Message {
                r#type: Some("mock_type".to_string()),
                user: Some("mock_user".to_string()),
                user_info: None,
                text: Some("mock_text".to_string()),
                thread_ts: Some("0000000000.000000".to_string()),
                reply_count: None,
                team: Some("mock_team".to_string()),
                ts: Some("0000000000.000000".to_string()),
            },
            Message {
                r#type: Some("mock_type".to_string()),
                user: Some("mock_user".to_string()),
                user_info: None,
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
    ChannelResponse {
        error: None,
        ok: Some(true),
        channel: None 
    },
    "https://mock.slack.com/archives/C0000000000/p0000000000000001?thread_ts=0000000000.000000".to_string(),
    SlackHttpClientConfigFeatureFlags {
        get_users: false,
        get_reactions: false,
        get_channel_info: false,
        get_attachments: false,
        get_team_info: false,
    },
    ObsidianSlackComponents {
        message_and_thread: MessageAndThread {
            message: Messages(
                vec![
                    Message {
                        r#type: Some("mock_type".to_string()),
                        user: Some("mock_user".to_string()),
                        user_info: None,
                        text: Some("mock_text2".to_string()),
                        thread_ts: Some("0000000000.000000".to_string()),
                        reply_count: None,
                        team: Some("mock_team".to_string()),
                        ts: Some("0000000000.000001".to_string()),
                    }
                ]
            ),
            thread: Messages(
                vec![
                    Message {
                        r#type: Some("mock_type".to_string()),
                        user: Some("mock_user".to_string()),
                        user_info: None,
                        text: Some("mock_text".to_string()),
                        thread_ts: Some("0000000000.000000".to_string()),
                        reply_count: None,
                        team: Some("mock_team".to_string()),
                        ts: Some("0000000000.000000".to_string()),
                    },
                    Message {
                        r#type: Some("mock_type".to_string()),
                        user: Some("mock_user".to_string()),
                        user_info: None,
                        text: Some("mock_text2".to_string()),
                        thread_ts: Some("0000000000.000000".to_string()),
                        reply_count: None,
                        team: Some("mock_team".to_string()),
                        ts: Some("0000000000.000001".to_string()),
                    }
                ]
            ),
        },
        file_name: FileName("C0000000000-0000000000.000000-0000000000.000001.json".to_string()),
        channel: None,
        users: None
    }
    ; "thread_ts - thread_ts and ts not the same - no flags")]
#[test_case(
    MessageResponse {
        is_null: Some(false),
        messages: Some(vec![
            Message {
                r#type: Some("mock_type".to_string()),
                user: Some("mock_user".to_string()),
                user_info: None,
                text: Some("mock_text".to_string()),
                thread_ts: Some("0000000000.000000".to_string()),
                reply_count: None,
                team: Some("mock_team".to_string()),
                ts: Some("0000000000.000000".to_string()),
            },
            Message {
                r#type: Some("mock_type".to_string()),
                user: Some("mock_user".to_string()),
                user_info: None,
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
    ChannelResponse {
        error: None,
        ok: Some(true),
        channel: None 
    },
    "https://mock.slack.com/archives/C0000000000/p0000000000000001?thread_ts=0000000000.000000".to_string(),
    SlackHttpClientConfigFeatureFlags {
        get_users: true,
        get_reactions: false,
        get_channel_info: false,
        get_attachments: false,
        get_team_info: false,
    },
    ObsidianSlackComponents {
        message_and_thread: MessageAndThread {
            message: Messages(
                vec![
                    Message {
                        r#type: Some("mock_type".to_string()),
                        user: Some("mock_user".to_string()),
                        user_info: Some(User {
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
                ]
            ),
            thread: Messages(
                vec![
                    Message {
                        r#type: Some("mock_type".to_string()),
                        user: Some("mock_user".to_string()),
                        user_info: Some(User {
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
                    Message {
                        r#type: Some("mock_type".to_string()),
                        user: Some("mock_user".to_string()),
                        user_info: Some(User {
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
                ]
            ),
        },
        file_name: FileName("C0000000000-0000000000.000000-0000000000.000001.json".to_string()),
        channel: None,
        users: Some(Users(
            [(
                "mock_user".to_owned(),
                User {
                    id: "mock_user".to_string(),
                    team_id: Some("mock_team".to_string()),
                    name: Some("mock_name".to_string()),
                    real_name: Some("mock_real_name".to_string()),
                }
            )].iter().cloned().collect()
        ))
    }
    ; "thread_ts - thread_ts and ts not the same - user flag")]
#[test_case(
    MessageResponse {
        is_null: Some(false),
        messages: Some(vec![
            Message {
                r#type: Some("mock_type".to_string()),
                user: Some("mock_user".to_string()),
                user_info: None,
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
        user: None,
    },
    ChannelResponse {
        error: None,
        ok: Some(true),
        channel: Some(Channel {
            id: None,
            name: None,
            is_channel: None,
            is_group: None,
            is_im: None,
            created: None,
            creator: None,
            is_archived: None,
            is_general: None,
            unlinked: None,
            name_normalized: None,
            is_read_only: None,
            is_shared: None,
            is_member: None,
            is_private: None,
            is_mpim: None,
            last_read: None,
            topic: None,
            purpose: None,
            previous_names: None,
            locale: None,
            is_org_shared: None,
            user: None,
            user_info: None,
            latest: None,
            unread_count: None,
            unread_count_display: None,
            is_open: None,
            priority: None,
        }) 
    },
    "https://mock.slack.com/archives/C0000000000/p0000000000000000".to_string(),
    SlackHttpClientConfigFeatureFlags {
        get_users: false,
        get_reactions: false,
        get_channel_info: true,
        get_attachments: false,
        get_team_info: false,
    },
    ObsidianSlackComponents {
        message_and_thread: MessageAndThread {
            message: Messages (
                vec![
                    Message {
                        r#type: Some("mock_type".to_string()),
                        user: Some("mock_user".to_string()),
                        user_info: None,
                        text: Some("mock_text".to_string()),
                        thread_ts: None,
                        reply_count: None,
                        team: Some("mock_team".to_string()),
                        ts: Some("0000000000.000000".to_string()),
                    }
                ]
            ),
            thread: Messages (
                vec![
                    Message {
                        r#type: Some("mock_type".to_string()),
                        user: Some("mock_user".to_string()),
                        user_info: None,
                        text: Some("mock_text".to_string()),
                        thread_ts: None,
                        reply_count: None,
                        team: Some("mock_team".to_string()),
                        ts: Some("0000000000.000000".to_string()),
                    }
                ]
            ),
        },
        file_name: FileName("C0000000000-0000000000.000000.json".to_string()),
        users: None,
        channel: Some(Channel {
            id: None,
            name: None,
            is_channel: None,
            is_group: None,
            is_im: None,
            created: None,
            creator: None,
            is_archived: None,
            is_general: None,
            unlinked: None,
            name_normalized: None,
            is_read_only: None,
            is_shared: None,
            is_member: None,
            is_private: None,
            is_mpim: None,
            last_read: None,
            topic: None,
            purpose: None,
            previous_names: None,
            locale: None,
            is_org_shared: None,
            user: None,
            user_info: None,
            latest: None,
            unread_count: None,
            unread_count_display: None,
            is_open: None,
            priority: None,
        })
    }
    ; "only channel flag set")]
#[test_case(
    MessageResponse {
        is_null: Some(false),
        messages: Some(vec![
            Message {
                r#type: Some("mock_type".to_string()),
                user: Some("mock_user".to_string()),
                user_info: None,
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
    ChannelResponse {
        error: None,
        ok: Some(true),
        channel: Some(Channel {
            id: None,
            name: None,
            is_channel: None,
            is_group: None,
            is_im: None,
            created: None,
            creator: None,
            is_archived: None,
            is_general: None,
            unlinked: None,
            name_normalized: None,
            is_read_only: None,
            is_shared: None,
            is_member: None,
            is_private: None,
            is_mpim: None,
            last_read: None,
            topic: None,
            purpose: None,
            previous_names: None,
            locale: None,
            is_org_shared: None,
            user: None,
            user_info: None,
            latest: None,
            unread_count: None,
            unread_count_display: None,
            is_open: None,
            priority: None,
        }) 
    },
    "https://mock.slack.com/archives/C0000000000/p0000000000000000".to_string(),
    SlackHttpClientConfigFeatureFlags {
        get_users: true,
        get_reactions: false,
        get_channel_info: true,
        get_attachments: false,
        get_team_info: false,
    },
    ObsidianSlackComponents {
        message_and_thread: MessageAndThread {
            message: Messages (
                vec![
                    Message {
                        r#type: Some("mock_type".to_string()),
                        user: Some("mock_user".to_string()),
                        user_info: Some(User {
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
                ]
            ),
            thread: Messages (
                vec![
                    Message {
                        r#type: Some("mock_type".to_string()),
                        user: Some("mock_user".to_string()),
                        user_info: Some(User {
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
                ]
            ),
        },
        file_name: FileName("C0000000000-0000000000.000000.json".to_string()),
        users: Some(Users(
            [(
                "mock_user".to_owned(),
                User {
                    id: "mock_user".to_string(),
                    team_id: Some("mock_team".to_string()),
                    name: Some("mock_name".to_string()),
                    real_name: Some("mock_real_name".to_string()),
                }
            )].iter().cloned().collect()
        )),
        channel: Some(Channel {
            id: None,
            name: None,
            is_channel: None,
            is_group: None,
            is_im: None,
            created: None,
            creator: None,
            is_archived: None,
            is_general: None,
            unlinked: None,
            name_normalized: None,
            is_read_only: None,
            is_shared: None,
            is_member: None,
            is_private: None,
            is_mpim: None,
            last_read: None,
            topic: None,
            purpose: None,
            previous_names: None,
            locale: None,
            is_org_shared: None,
            user: None,
            user_info: None,
            latest: None,
            unread_count: None,
            unread_count_display: None,
            is_open: None,
            priority: None,
        })
    }
    ; "channel and user flag set, channel has no user")]
#[test_case(
    MessageResponse {
        is_null: Some(false),
        messages: Some(vec![
            Message {
                r#type: Some("mock_type".to_string()),
                user: Some("mock_user".to_string()),
                user_info: None,
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
    ChannelResponse {
        error: None,
        ok: Some(true),
        channel: Some(Channel {
            id: None,
            name: None,
            is_channel: None,
            is_group: None,
            is_im: None,
            created: None,
            creator: None,
            is_archived: None,
            is_general: None,
            unlinked: None,
            name_normalized: None,
            is_read_only: None,
            is_shared: None,
            is_member: None,
            is_private: None,
            is_mpim: None,
            last_read: None,
            topic: None,
            purpose: None,
            previous_names: None,
            locale: None,
            is_org_shared: None,
            user: Some("mock_user".to_string()),
            user_info: None,
            latest: None,
            unread_count: None,
            unread_count_display: None,
            is_open: None,
            priority: None,
        }) 
    },
    "https://mock.slack.com/archives/C0000000000/p0000000000000000".to_string(),
    SlackHttpClientConfigFeatureFlags {
        get_users: true,
        get_reactions: false,
        get_channel_info: true,
        get_attachments: false,
        get_team_info: false,
    },
    ObsidianSlackComponents {
        message_and_thread: MessageAndThread {
            message: Messages (
                vec![
                    Message {
                        r#type: Some("mock_type".to_string()),
                        user: Some("mock_user".to_string()),
                        user_info: Some(User {
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
                ]
            ),
            thread: Messages (
                vec![
                    Message {
                        r#type: Some("mock_type".to_string()),
                        user: Some("mock_user".to_string()),
                        user_info: Some(User {
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
                ]
            ),
        },
        file_name: FileName("C0000000000-0000000000.000000.json".to_string()),
        users: Some(Users(
            [(
                "mock_user".to_owned(),
                User {
                    id: "mock_user".to_string(),
                    team_id: Some("mock_team".to_string()),
                    name: Some("mock_name".to_string()),
                    real_name: Some("mock_real_name".to_string()),
                }
            )].iter().cloned().collect()
        )),
        channel: Some(Channel {
            id: None,
            name: None,
            is_channel: None,
            is_group: None,
            is_im: None,
            created: None,
            creator: None,
            is_archived: None,
            is_general: None,
            unlinked: None,
            name_normalized: None,
            is_read_only: None,
            is_shared: None,
            is_member: None,
            is_private: None,
            is_mpim: None,
            last_read: None,
            topic: None,
            purpose: None,
            previous_names: None,
            locale: None,
            is_org_shared: None,
            user: Some("mock_user".to_string()),
            user_info: Some(User {
                id: "mock_user".to_string(),
                team_id: Some("mock_team".to_string()),
                name: Some("mock_name".to_string()),
                real_name: Some("mock_real_name".to_string()),
            }),
            latest: None,
            unread_count: None,
            unread_count_display: None,
            is_open: None,
            priority: None,
        })
    }
    ; "channel and user flag set, channel has user")]
#[wasm_bindgen_test]
async fn get_slack_message_returns_data_correctly(
    message_response: MessageResponse,
    user_response: UserResponse,
    channel_response: ChannelResponse,
    url: String,
    feature_flags: SlackHttpClientConfigFeatureFlags,
    expected_return_data: ObsidianSlackComponents,
) {
    let feature_flags = serde_wasm_bindgen::to_value(&feature_flags).unwrap();

    let request_func = get_mock_request_function(message_response, user_response, channel_response);

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

    console_log!("Result: {:#?}", result);
    assert!(!result.is_string(), "Result was a string: {:#?}", result);

    let result: ObsidianSlackComponents =
        serde_wasm_bindgen::from_value(result).expect("Should parse return object");

    assert_eq!(expected_return_data, result);
}

#[test_case(
    None,
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
    },
    "InvalidSlackApiToken"
    ; "bad api token"
)]
#[test_case(
    None,
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
    },
    "InvalidSlackApiCookie"
    ; "bad cookie"
)]
#[test_case(
    None,
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
    },
    "ChannelIdNotFoundInPathSegments"
    ; "bad channel_id"
)]
#[test_case(
    None,
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
    },
    "TimestampNotFound"
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
    None,
    "xoxc...".to_string(),
    "xoxd...".to_string(),
    "https://mock.slack.com/archives/C0000000000/p0000000000000000?thread_ts=0000000000.000000".to_string(),
    SlackHttpClientConfigFeatureFlags {
        get_users: true,
        get_reactions: false,
        get_channel_info: false,
        get_attachments: false,
        get_team_info: false,
    },
    "InvalidMessageResponse"
    ; "message response not ok - no flags"
)]
#[test_case(
    Some(MessageResponse {
        is_null: Some(false),
        messages: Some(vec![
            Message {
                r#type: Some("mock_type".to_string()),
                user: Some("mock_user".to_string()),
                user_info: None,
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
    None,
    "xoxc...".to_string(),
    "xoxd...".to_string(),
    "https://mock.slack.com/archives/C0000000000/p0000000000000000?thread_ts=0000000000.000000".to_string(),
    SlackHttpClientConfigFeatureFlags {
        get_users: true,
        get_reactions: false,
        get_channel_info: false,
        get_attachments: false,
        get_team_info: false,
    },
    "InvalidUserResponse"
    ; "user response not ok - user flag"
)]
#[test_case(
    Some(MessageResponse {
        is_null: Some(false),
        messages: Some(vec![
            Message {
                r#type: Some("mock_type".to_string()),
                user: Some("mock_user".to_string()),
                user_info: None,
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
    None,
    Some(ChannelResponse {
        ok: Some(false),
        error: None,
        channel: None
    }),
    "xoxc...".to_string(),
    "xoxd...".to_string(),
    "https://mock.slack.com/archives/C0000000000/p0000000000000000?thread_ts=0000000000.000000".to_string(),
    SlackHttpClientConfigFeatureFlags {
        get_users: false,
        get_reactions: false,
        get_channel_info: true,
        get_attachments: false,
        get_team_info: false,
    },
    "InvalidChannelResponse"
    ; "channel response not ok - channel flag"
)]
#[wasm_bindgen_test]
async fn get_slack_message_returns_error_messages_correctly(
    message_response: Option<MessageResponse>,
    user_response: Option<UserResponse>,
    channel_response: Option<ChannelResponse>,
    api_token: String,
    cookie: String,
    url: String,
    feature_flags: SlackHttpClientConfigFeatureFlags,
    expected_error: &str,
) {
    let feature_flags = serde_wasm_bindgen::to_value(&feature_flags).unwrap();

    let request_func = get_mock_request_function(
        message_response.unwrap_or(MessageResponse {
            is_null: Some(false),
            messages: Some(vec![Message {
                r#type: Some("mock_type".to_string()),
                user: Some("mock_user".to_string()),
                user_info: None,
                text: Some("mock_text".to_string()),
                thread_ts: Some("0000000000.000000".to_string()),
                reply_count: None,
                team: Some("mock_team".to_string()),
                ts: Some("0000000000.000000".to_string()),
            }]),
            has_more: Some(false),
            ok: Some(true),
            error: None,
            response_metadata: None,
        }),
        user_response.unwrap_or(UserResponse {
            ok: Some(true),
            error: None,
            user: Some(User {
                id: "mock_user".to_string(),
                team_id: Some("mock_team".to_string()),
                name: Some("mock_name".to_string()),
                real_name: Some("mock_real_name".to_string()),
            }),
        }),
        channel_response.unwrap_or(ChannelResponse {
            error: None,
            ok: Some(true),
            channel: None, 
        })
    );

    let result = get_slack_message(
        api_token.to_string(),
        cookie.to_string(),
        url,
        feature_flags,
        request_func,
    )
    .await;

    console_log!("Result: {:#?}", result);
    assert!(result.is_string(), "Result was not a string: {:#?}", result);
    assert!(result.as_string().unwrap().contains(expected_error))
}
