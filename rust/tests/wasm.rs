use core::panic;
use std::ops::Index;

use js_sys::JSON;
use obsidian_slack::{
    channels::{Channel, ChannelResponse},
    components::{FileName, ObsidianSlackComponents},
    get_slack_message,
    messages::{
        File, FileLinks, Files, Message, MessageAndThread, MessageResponse, Messages, Reaction,
        Reactions,
    },
    slack_http_client::SlackHttpClientConfigFeatureFlags,
    team::{Team, TeamResponse, Teams},
    users::{User, UserResponse, Users},
};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

const DEFAULT_CHANNEL_ID: &str = "C0000000000";
const DEFAULT_TS: &str = "p0000000000000000";
const DEFAULT_TS_PARSED: &str = "0000000000.000000";
const DEFAULT_THREAD_TS: &str = "0000000000.000000";
const DEFAULT_USER_ID: &str = "mock_user";
const DEFAULT_TEAM_ID: &str = "mock_team";

fn get_mock_request_function(
    message_response: MessageResponse,
    user_response: Option<UserResponse>,
    channel_response: Option<ChannelResponse>,
    team_response: Option<TeamResponse>,
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
            else if (params.url.includes("team.info")) {{
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
            JSON::stringify(
                &serde_wasm_bindgen::to_value(&user_response.unwrap_or(UserResponse {
                    error: None,
                    ok: Some(true),
                    user: None
                }))
                .unwrap()
            )
            .unwrap()
        ),
        Into::<String>::into(
            JSON::stringify(
                &serde_wasm_bindgen::to_value(&channel_response.unwrap_or({
                    ChannelResponse {
                        error: None,
                        ok: Some(true),
                        channel: None,
                    }
                }))
                .unwrap()
            )
            .unwrap()
        ),
        Into::<String>::into(
            JSON::stringify(
                &serde_wasm_bindgen::to_value(&team_response.unwrap_or(TeamResponse {
                    error: None,
                    ok: Some(true),
                    team: None
                }))
                .unwrap()
            )
            .unwrap()
        ),
    );

    let func = js_sys::Function::new_with_args("params", &func_body);
    assert_eq!(func.js_typeof(), JsValue::from_str("function"));
    JsValue::from(func)
}

fn channel(user: Option<User>, user_id: Option<String>) -> Channel {
    Channel {
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
        user: user_id,
        user_info: user,
        latest: None,
        unread_count: None,
        unread_count_display: None,
        is_open: None,
        priority: None,
    }
}

fn team() -> Team {
    Team {
        id: DEFAULT_TEAM_ID.to_string(),
        name: "mock_team_name".to_string(),
        domain: None,
        email_domain: None,
        enterprise_id: None,
        enterprise_name: None,
    }
}

fn user(team: Option<Team>) -> User {
    User {
        id: DEFAULT_USER_ID.to_string(),
        team_id: Some(DEFAULT_TEAM_ID.to_string()),
        team_info: team,
        name: Some("mock_name".to_string()),
        real_name: Some("mock_real_name".to_string()),
    }
}

fn reaction(user: Option<User>) -> Reaction {
    Reaction {
        name: "mock reaction".to_string(),
        users: vec![DEFAULT_USER_ID.to_string()],
        users_info: user.map(|user| vec![user]),
        count: 1,
    }
}

fn message(
    timestamp: String,
    thread_timestamp: String,
    user: Option<User>,
    reactions: Option<Reactions>,
    files: Option<Files>,
) -> Message {
    Message {
        r#type: Some("mock_type".to_string()),
        user: Some(DEFAULT_USER_ID.to_string()),
        user_info: user,
        text: Some("mock_text".to_string()),
        thread_ts: Some(thread_timestamp),
        reply_count: None,
        ts: Some(timestamp),
        reactions,
        files,
    }
}

fn messages(
    timestamps: Vec<(String, String)>,
    user: Option<User>,
    reactions: Option<Reactions>,
    files: Option<Files>,
) -> Messages {
    Messages(
        timestamps
            .into_iter()
            .map(|(ts, thread_ts)| {
                message(
                    ts,
                    thread_ts,
                    user.clone(),
                    reactions.clone(),
                    files.clone(),
                )
            })
            .collect(),
    )
}

fn files() -> Files {
    Files(vec![File {
        id: "my-file-id".to_string(),
        name: "my-file-name".to_string(),
        user_team: "my-file-user-team".to_string(),
        title: "my-file-title".to_string(),
        mimetype: "my-file-mime-type".to_string(),
        filetype: "my-file-file-type".to_string(),
        size: 42,
        url_private: "https://files.slack.com/my-file-url-private".to_string(),
        url_private_download: "my-file-url-private-download".to_string(),
        permalink: "my-file-permalink".to_string(),
        permalink_public: "my-file-permalink-public".to_string(),
    }])
}

fn file_links() -> FileLinks {
    files().collect_file_links()
}

fn team_response(ok: Option<bool>, error: Option<String>, team: Option<Team>) -> TeamResponse {
    TeamResponse { ok, error, team }
}

fn channel_response(
    ok: Option<bool>,
    error: Option<String>,
    channel: Option<Channel>,
) -> ChannelResponse {
    ChannelResponse { ok, error, channel }
}

fn user_response(ok: Option<bool>, error: Option<String>, user: Option<User>) -> UserResponse {
    UserResponse { ok, error, user }
}

fn message_response(
    ok: Option<bool>,
    error: Option<String>,
    messages: Option<Messages>,
) -> MessageResponse {
    MessageResponse {
        messages: messages.map(|messages| messages.0),
        ok,
        error,
    }
}

fn url(channel_id: Option<String>, ts: Option<String>, thread_ts: Option<String>) -> String {
    match (channel_id, ts, thread_ts) {
        (Some(cid), None, None) => format!("https://mock.slack.com/archives/{cid}"),
        (Some(cid), Some(ts), None) => {
            format!("https://mock.slack.com/archives/{cid}/{ts}")
        }
        (Some(cid), None, Some(thread_ts)) => {
            format!("https://mock.slack.com/archives/{cid}?thread_ts={thread_ts}")
        }
        (Some(cid), Some(ts), Some(thread_ts)) => {
            format!("https://mock.slack.com/archives/{cid}/{ts}?thread_ts={thread_ts}")
        }
        (None, Some(ts), None) => format!("https://mock.slack.com/archives/{ts}"),
        (None, Some(ts), Some(thread_ts)) => {
            format!("https://mock.slack.com/archives/{ts}?thread_ts={thread_ts}")
        }
        (None, None, Some(thread_ts)) => {
            format!("https://mock.slack.com/archives?thread_ts={thread_ts}")
        }
        (None, None, None) => "https://mock.slack.com/archives".to_string(),
    }
}

fn feature_flags(
    get_users: bool,
    get_channel_info: bool,
    get_team_info: bool,
    get_file_data: bool,
) -> SlackHttpClientConfigFeatureFlags {
    SlackHttpClientConfigFeatureFlags {
        get_users,
        get_channel_info,
        get_team_info,
        get_file_data,
    }
}

fn obsidian_slack_components(
    message_and_thread: MessageAndThread,
    file_name: FileName,
    users: Option<Users>,
    channel: Option<Channel>,
    teams: Option<Teams>,
    file_links: Option<FileLinks>,
) -> ObsidianSlackComponents {
    ObsidianSlackComponents {
        message_and_thread,
        file_name,
        users,
        channel,
        teams,
        file_links,
    }
}

fn message_and_thread(message: Messages, thread: Messages) -> MessageAndThread {
    MessageAndThread { message, thread }
}

fn file_name(
    channel_id: Option<String>,
    ts: Option<String>,
    thread_ts: Option<String>,
) -> FileName {
    match (channel_id, ts, thread_ts) {
        (Some(cid), Some(ts), None) => FileName(format!("{cid}-{ts}.json")),
        (Some(cid), Some(ts), Some(thread_ts)) => {
            if ts == thread_ts {
                FileName(format!("{cid}-{ts}.json"))
            } else {
                FileName(format!("{cid}-{thread_ts}-{ts}.json"))
            }
        }
        _ => panic!("unsupported filename in test suite"),
    }
}

async fn get_slack_message_returns_data_correctly_common(
    message_response: MessageResponse,
    user_response: Option<UserResponse>,
    channel_response: Option<ChannelResponse>,
    team_response: Option<TeamResponse>,
    url: String,
    feature_flags: SlackHttpClientConfigFeatureFlags,
    expected_return_data: ObsidianSlackComponents,
) {
    let feature_flags = serde_wasm_bindgen::to_value(&feature_flags).unwrap();

    let request_func = get_mock_request_function(
        message_response,
        user_response,
        channel_response,
        team_response,
    );

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

    let result: ObsidianSlackComponents =
        serde_wasm_bindgen::from_value(result).expect("Should parse return object");

    assert_eq!(expected_return_data, result);
}

#[wasm_bindgen_test]
async fn get_slack_message_returns_data_correctly_no_feature_flags_no_reactions_just_ts_in_url_and_one_message(
) {
    let message_response = message_response(
        Some(true),
        None,
        Some(messages(
            vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
            None,
            None,
            None,
        )),
    );
    let user_response = None;
    let channel_response = None;
    let team_response = None;
    let url = url(
        Some(DEFAULT_CHANNEL_ID.to_string()),
        Some(DEFAULT_TS.to_string()),
        None,
    );
    let feature_flags = feature_flags(false, false, false, false);
    let expected_return_data = obsidian_slack_components(
        message_and_thread(
            messages(
                vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
                None,
                None,
                None,
            ),
            messages(
                vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
                None,
                None,
                None,
            ),
        ),
        file_name(
            Some(DEFAULT_CHANNEL_ID.to_string()),
            Some(DEFAULT_TS_PARSED.to_string()),
            None,
        ),
        None,
        None,
        None,
        None,
    );

    get_slack_message_returns_data_correctly_common(
        message_response,
        user_response,
        channel_response,
        team_response,
        url,
        feature_flags,
        expected_return_data,
    )
    .await;
}

#[wasm_bindgen_test]
async fn get_slack_message_returns_data_correctly_no_feature_flags_no_reactions_just_ts_and_multiple_messages(
) {
    let message_response = message_response(
        Some(true),
        None,
        Some(messages(
            vec![
                (DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string()),
                (
                    DEFAULT_TS_PARSED.to_string() + "1",
                    DEFAULT_THREAD_TS.to_string(),
                ),
            ],
            None,
            None,
            None,
        )),
    );
    let user_response = None;
    let channel_response = None;
    let team_response = None;
    let url = url(
        Some(DEFAULT_CHANNEL_ID.to_string()),
        Some(DEFAULT_TS.to_string()),
        None,
    );
    let feature_flags = feature_flags(false, false, false, false);
    let expected_return_data = obsidian_slack_components(
        message_and_thread(
            messages(
                vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
                None,
                None,
                None,
            ),
            messages(
                vec![
                    (DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string()),
                    (
                        DEFAULT_TS_PARSED.to_string() + "1",
                        DEFAULT_THREAD_TS.to_string(),
                    ),
                ],
                None,
                None,
                None,
            ),
        ),
        file_name(
            Some(DEFAULT_CHANNEL_ID.to_string()),
            Some(DEFAULT_TS_PARSED.to_string()),
            None,
        ),
        None,
        None,
        None,
        None,
    );

    get_slack_message_returns_data_correctly_common(
        message_response,
        user_response,
        channel_response,
        team_response,
        url,
        feature_flags,
        expected_return_data,
    )
    .await;
}

#[wasm_bindgen_test]
async fn get_slack_message_returns_data_correctly_no_feature_flags_no_reactions_ts_and_thread_ts_that_are_the_same_and_multiple_messages(
) {
    let message_response = message_response(
        Some(true),
        None,
        Some(messages(
            vec![
                (DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string()),
                (
                    DEFAULT_TS_PARSED.to_string() + "1",
                    DEFAULT_THREAD_TS.to_string(),
                ),
            ],
            None,
            None,
            None,
        )),
    );
    let user_response = None;
    let channel_response = None;
    let team_response = None;
    let url = url(
        Some(DEFAULT_CHANNEL_ID.to_string()),
        Some(DEFAULT_TS.to_string()),
        Some(DEFAULT_THREAD_TS.to_string()),
    );
    let feature_flags = feature_flags(false, false, false, false);
    let expected_return_data = obsidian_slack_components(
        message_and_thread(
            messages(
                vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
                None,
                None,
                None,
            ),
            messages(
                vec![
                    (DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string()),
                    (
                        DEFAULT_TS_PARSED.to_string() + "1",
                        DEFAULT_THREAD_TS.to_string(),
                    ),
                ],
                None,
                None,
                None,
            ),
        ),
        file_name(
            Some(DEFAULT_CHANNEL_ID.to_string()),
            Some(DEFAULT_TS_PARSED.to_string()),
            Some(DEFAULT_THREAD_TS.to_string()),
        ),
        None,
        None,
        None,
        None,
    );

    get_slack_message_returns_data_correctly_common(
        message_response,
        user_response,
        channel_response,
        team_response,
        url,
        feature_flags,
        expected_return_data,
    )
    .await;
}

#[wasm_bindgen_test]
async fn get_slack_message_returns_data_correctly_no_feature_flags_no_reactions_ts_and_thread_ts_that_are_different_and_multiple_messages(
) {
    let message_response = message_response(
        Some(true),
        None,
        Some(messages(
            vec![
                (DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string()),
                (
                    DEFAULT_TS_PARSED.to_string() + "1",
                    DEFAULT_THREAD_TS.to_string(),
                ),
            ],
            None,
            None,
            None,
        )),
    );
    let user_response = None;
    let channel_response = None;
    let team_response = None;
    let url = url(
        Some(DEFAULT_CHANNEL_ID.to_string()),
        Some(DEFAULT_TS.to_string() + "1"),
        Some(DEFAULT_THREAD_TS.to_string()),
    );
    let feature_flags = feature_flags(false, false, false, false);
    let expected_return_data = obsidian_slack_components(
        message_and_thread(
            messages(
                vec![(
                    DEFAULT_TS_PARSED.to_string() + "1",
                    DEFAULT_THREAD_TS.to_string(),
                )],
                None,
                None,
                None,
            ),
            messages(
                vec![
                    (DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string()),
                    (
                        DEFAULT_TS_PARSED.to_string() + "1",
                        DEFAULT_THREAD_TS.to_string(),
                    ),
                ],
                None,
                None,
                None,
            ),
        ),
        file_name(
            Some(DEFAULT_CHANNEL_ID.to_string()),
            Some(DEFAULT_TS_PARSED.to_string() + "1"),
            Some(DEFAULT_THREAD_TS.to_string()),
        ),
        None,
        None,
        None,
        None,
    );

    get_slack_message_returns_data_correctly_common(
        message_response,
        user_response,
        channel_response,
        team_response,
        url,
        feature_flags,
        expected_return_data,
    )
    .await;
}

#[wasm_bindgen_test]
async fn get_slack_message_returns_data_correctly_no_feature_flags_no_reactions_just_ts_in_url_and_one_message_with_reactions(
) {
    let message_response = message_response(
        Some(true),
        None,
        Some(messages(
            vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
            None,
            Some(Reactions(vec![reaction(None)])),
            None,
        )),
    );
    let user_response = None;
    let channel_response = None;
    let team_response = None;
    let url = url(
        Some(DEFAULT_CHANNEL_ID.to_string()),
        Some(DEFAULT_TS.to_string()),
        None,
    );
    let feature_flags = feature_flags(false, false, false, false);
    let expected_return_data = obsidian_slack_components(
        message_and_thread(
            messages(
                vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
                None,
                Some(Reactions(vec![reaction(None)])),
                None,
            ),
            messages(
                vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
                None,
                Some(Reactions(vec![reaction(None)])),
                None,
            ),
        ),
        file_name(
            Some(DEFAULT_CHANNEL_ID.to_string()),
            Some(DEFAULT_TS_PARSED.to_string()),
            None,
        ),
        None,
        None,
        None,
        None,
    );

    get_slack_message_returns_data_correctly_common(
        message_response,
        user_response,
        channel_response,
        team_response,
        url,
        feature_flags,
        expected_return_data,
    )
    .await;
}

#[wasm_bindgen_test]
async fn get_slack_message_returns_data_correctly_with_at_least_channel_info_flag_set() {
    let message_response = message_response(
        Some(true),
        None,
        Some(messages(
            vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
            None,
            Some(Reactions(vec![reaction(None)])),
            None,
        )),
    );
    let user_response = None;
    let channel_response = Some(channel_response(
        Some(true),
        None,
        Some(channel(None, None)),
    ));
    let team_response = None;
    let url = url(
        Some(DEFAULT_CHANNEL_ID.to_string()),
        Some(DEFAULT_TS.to_string()),
        None,
    );
    let feature_flags = feature_flags(false, true, false, false);
    let expected_return_data = obsidian_slack_components(
        message_and_thread(
            messages(
                vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
                None,
                Some(Reactions(vec![reaction(None)])),
                None,
            ),
            messages(
                vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
                None,
                Some(Reactions(vec![reaction(None)])),
                None,
            ),
        ),
        file_name(
            Some(DEFAULT_CHANNEL_ID.to_string()),
            Some(DEFAULT_TS_PARSED.to_string()),
            None,
        ),
        None,
        Some(channel(None, None)),
        None,
        None,
    );

    get_slack_message_returns_data_correctly_common(
        message_response,
        user_response,
        channel_response,
        team_response,
        url,
        feature_flags,
        expected_return_data,
    )
    .await;
}

#[wasm_bindgen_test]
async fn get_slack_message_returns_data_correctly_with_user_info_flag_set_and_channel_info_flag_not_set(
) {
    let message_response = message_response(
        Some(true),
        None,
        Some(messages(
            vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
            None,
            Some(Reactions(vec![reaction(None)])),
            None,
        )),
    );
    let user_response = Some(user_response(Some(true), None, Some(user(None))));
    let channel_response = None;
    let team_response = None;
    let url = url(
        Some(DEFAULT_CHANNEL_ID.to_string()),
        Some(DEFAULT_TS.to_string()),
        None,
    );
    let feature_flags = feature_flags(true, false, false, false);
    let expected_return_data = obsidian_slack_components(
        message_and_thread(
            messages(
                vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
                Some(user(None)),
                Some(Reactions(vec![reaction(Some(user(None)))])),
                None,
            ),
            messages(
                vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
                Some(user(None)),
                Some(Reactions(vec![reaction(Some(user(None)))])),
                None,
            ),
        ),
        file_name(
            Some(DEFAULT_CHANNEL_ID.to_string()),
            Some(DEFAULT_TS_PARSED.to_string()),
            None,
        ),
        Some(Users(
            vec![(DEFAULT_USER_ID.to_string(), user(None))]
                .into_iter()
                .collect(),
        )),
        None,
        None,
        None,
    );

    get_slack_message_returns_data_correctly_common(
        message_response,
        user_response,
        channel_response,
        team_response,
        url,
        feature_flags,
        expected_return_data,
    )
    .await;
}

#[wasm_bindgen_test]
async fn get_slack_message_returns_data_correctly_with_user_info_flag_set_and_channel_info_flag_set_and_channel_does_not_have_user(
) {
    let message_response = message_response(
        Some(true),
        None,
        Some(messages(
            vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
            None,
            Some(Reactions(vec![reaction(None)])),
            None,
        )),
    );
    let user_response = Some(user_response(Some(true), None, Some(user(None))));
    let channel_response = Some(channel_response(
        Some(true),
        None,
        Some(channel(None, None)),
    ));
    let team_response = None;
    let url = url(
        Some(DEFAULT_CHANNEL_ID.to_string()),
        Some(DEFAULT_TS.to_string()),
        None,
    );
    let feature_flags = feature_flags(true, true, false, false);
    let expected_return_data = obsidian_slack_components(
        message_and_thread(
            messages(
                vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
                Some(user(None)),
                Some(Reactions(vec![reaction(Some(user(None)))])),
                None,
            ),
            messages(
                vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
                Some(user(None)),
                Some(Reactions(vec![reaction(Some(user(None)))])),
                None,
            ),
        ),
        file_name(
            Some(DEFAULT_CHANNEL_ID.to_string()),
            Some(DEFAULT_TS_PARSED.to_string()),
            None,
        ),
        Some(Users(
            vec![(DEFAULT_USER_ID.to_string(), user(None))]
                .into_iter()
                .collect(),
        )),
        Some(channel(None, None)),
        None,
        None,
    );

    get_slack_message_returns_data_correctly_common(
        message_response,
        user_response,
        channel_response,
        team_response,
        url,
        feature_flags,
        expected_return_data,
    )
    .await;
}

#[wasm_bindgen_test]
async fn get_slack_message_returns_data_correctly_with_user_info_flag_set_and_channel_info_flag_set_and_channel_does_have_user(
) {
    let message_response = message_response(
        Some(true),
        None,
        Some(messages(
            vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
            None,
            Some(Reactions(vec![reaction(None)])),
            None,
        )),
    );
    let user_response = Some(user_response(Some(true), None, Some(user(None))));
    let channel_response = Some(channel_response(
        Some(true),
        None,
        Some(channel(None, Some(DEFAULT_USER_ID.to_string()))),
    ));
    let team_response = None;
    let url = url(
        Some(DEFAULT_CHANNEL_ID.to_string()),
        Some(DEFAULT_TS.to_string()),
        None,
    );
    let feature_flags = feature_flags(true, true, false, false);
    let expected_return_data = obsidian_slack_components(
        message_and_thread(
            messages(
                vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
                Some(user(None)),
                Some(Reactions(vec![reaction(Some(user(None)))])),
                None,
            ),
            messages(
                vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
                Some(user(None)),
                Some(Reactions(vec![reaction(Some(user(None)))])),
                None,
            ),
        ),
        file_name(
            Some(DEFAULT_CHANNEL_ID.to_string()),
            Some(DEFAULT_TS_PARSED.to_string()),
            None,
        ),
        Some(Users(
            vec![(DEFAULT_USER_ID.to_string(), user(None))]
                .into_iter()
                .collect(),
        )),
        Some(channel(Some(user(None)), Some(DEFAULT_USER_ID.to_string()))),
        None,
        None,
    );

    get_slack_message_returns_data_correctly_common(
        message_response,
        user_response,
        channel_response,
        team_response,
        url,
        feature_flags,
        expected_return_data,
    )
    .await;
}

#[wasm_bindgen_test]
async fn get_slack_message_returns_data_correctly_with_user_info_flag_set_and_team_info_flag_set() {
    let message_response = message_response(
        Some(true),
        None,
        Some(messages(
            vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
            None,
            Some(Reactions(vec![reaction(None)])),
            None,
        )),
    );
    let user_response = Some(user_response(Some(true), None, Some(user(None))));
    let channel_response = None;
    let team_response = Some(team_response(Some(true), None, Some(team())));
    let url = url(
        Some(DEFAULT_CHANNEL_ID.to_string()),
        Some(DEFAULT_TS.to_string()),
        None,
    );
    let feature_flags = feature_flags(true, false, true, false);
    let expected_return_data = obsidian_slack_components(
        message_and_thread(
            messages(
                vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
                Some(user(Some(team()))),
                Some(Reactions(vec![reaction(Some(user(Some(team()))))])),
                None,
            ),
            messages(
                vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
                Some(user(Some(team()))),
                Some(Reactions(vec![reaction(Some(user(Some(team()))))])),
                None,
            ),
        ),
        file_name(
            Some(DEFAULT_CHANNEL_ID.to_string()),
            Some(DEFAULT_TS_PARSED.to_string()),
            None,
        ),
        Some(Users(
            vec![(DEFAULT_USER_ID.to_string(), user(Some(team())))]
                .into_iter()
                .collect(),
        )),
        None,
        Some(Teams(
            vec![(DEFAULT_TEAM_ID.to_string(), team())]
                .into_iter()
                .collect(),
        )),
        None,
    );

    get_slack_message_returns_data_correctly_common(
        message_response,
        user_response,
        channel_response,
        team_response,
        url,
        feature_flags,
        expected_return_data,
    )
    .await;
}

#[wasm_bindgen_test]
async fn get_slack_message_returns_data_correctly_with_user_info_flag_set_and_channel_info_flag_set_and_channel_does_have_user_and_team_info_flag_set(
) {
    let message_response = message_response(
        Some(true),
        None,
        Some(messages(
            vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
            None,
            Some(Reactions(vec![reaction(None)])),
            None,
        )),
    );
    let user_response = Some(user_response(Some(true), None, Some(user(None))));
    let channel_response = Some(channel_response(
        Some(true),
        None,
        Some(channel(None, Some(DEFAULT_USER_ID.to_string()))),
    ));
    let team_response = Some(team_response(Some(true), None, Some(team())));
    let url = url(
        Some(DEFAULT_CHANNEL_ID.to_string()),
        Some(DEFAULT_TS.to_string()),
        None,
    );
    let feature_flags = feature_flags(true, true, true, false);
    let expected_return_data = obsidian_slack_components(
        message_and_thread(
            messages(
                vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
                Some(user(Some(team()))),
                Some(Reactions(vec![reaction(Some(user(Some(team()))))])),
                None,
            ),
            messages(
                vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
                Some(user(Some(team()))),
                Some(Reactions(vec![reaction(Some(user(Some(team()))))])),
                None,
            ),
        ),
        file_name(
            Some(DEFAULT_CHANNEL_ID.to_string()),
            Some(DEFAULT_TS_PARSED.to_string()),
            None,
        ),
        Some(Users(
            vec![(DEFAULT_USER_ID.to_string(), user(Some(team())))]
                .into_iter()
                .collect(),
        )),
        Some(channel(
            Some(user(Some(team()))),
            Some(DEFAULT_USER_ID.to_string()),
        )),
        Some(Teams(
            vec![(DEFAULT_TEAM_ID.to_string(), team())]
                .into_iter()
                .collect(),
        )),
        None,
    );

    get_slack_message_returns_data_correctly_common(
        message_response,
        user_response,
        channel_response,
        team_response,
        url,
        feature_flags,
        expected_return_data,
    )
    .await;
}

#[wasm_bindgen_test]
async fn get_slack_message_returns_data_correctly_with_file_data_flag_set_only() {
    let message_response = message_response(
        Some(true),
        None,
        Some(messages(
            vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
            None,
            None,
            Some(files()),
        )),
    );
    let user_response = None;
    let channel_response = None;
    let team_response = None;
    let url = url(
        Some(DEFAULT_CHANNEL_ID.to_string()),
        Some(DEFAULT_TS.to_string()),
        None,
    );
    let feature_flags = feature_flags(false, false, false, true);
    let expected_return_data = obsidian_slack_components(
        message_and_thread(
            messages(
                vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
                None,
                None,
                Some(files()),
            ),
            messages(
                vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
                None,
                None,
                Some(files()),
            ),
        ),
        file_name(
            Some(DEFAULT_CHANNEL_ID.to_string()),
            Some(DEFAULT_TS_PARSED.to_string()),
            None,
        ),
        None,
        None,
        None,
        Some(file_links()),
    );

    get_slack_message_returns_data_correctly_common(
        message_response,
        user_response,
        channel_response,
        team_response,
        url,
        feature_flags,
        expected_return_data,
    )
    .await;
}

#[wasm_bindgen_test]
async fn get_slack_message_returns_data_correctly_with_file_data_flag_and_user_flag_set_only() {
    let message_response = message_response(
        Some(true),
        None,
        Some(messages(
            vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
            None,
            None,
            Some(files()),
        )),
    );
    let user_response = Some(user_response(Some(true), None, Some(user(None))));
    let channel_response = None;
    let team_response = None;
    let url = url(
        Some(DEFAULT_CHANNEL_ID.to_string()),
        Some(DEFAULT_TS.to_string()),
        None,
    );
    let feature_flags = feature_flags(true, false, false, true);
    let expected_return_data = obsidian_slack_components(
        message_and_thread(
            messages(
                vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
                Some(user(None)),
                None,
                Some(files()),
            ),
            messages(
                vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
                Some(user(None)),
                None,
                Some(files()),
            ),
        ),
        file_name(
            Some(DEFAULT_CHANNEL_ID.to_string()),
            Some(DEFAULT_TS_PARSED.to_string()),
            None,
        ),
        Some(Users(
            vec![(DEFAULT_USER_ID.to_string(), user(None))]
                .into_iter()
                .collect(),
        )),
        None,
        None,
        Some(file_links()),
    );

    get_slack_message_returns_data_correctly_common(
        message_response,
        user_response,
        channel_response,
        team_response,
        url,
        feature_flags,
        expected_return_data,
    )
    .await;
}

#[wasm_bindgen_test]
async fn get_slack_message_returns_data_correctly_with_file_data_flag_and_channel_flag_set_only() {
    let message_response = message_response(
        Some(true),
        None,
        Some(messages(
            vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
            None,
            None,
            Some(files()),
        )),
    );
    let user_response = None;
    let channel_response = Some(channel_response(
        Some(true),
        None,
        Some(channel(None, None)),
    ));
    let team_response = None;
    let url = url(
        Some(DEFAULT_CHANNEL_ID.to_string()),
        Some(DEFAULT_TS.to_string()),
        None,
    );
    let feature_flags = feature_flags(false, true, false, true);
    let expected_return_data = obsidian_slack_components(
        message_and_thread(
            messages(
                vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
                None,
                None,
                Some(files()),
            ),
            messages(
                vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
                None,
                None,
                Some(files()),
            ),
        ),
        file_name(
            Some(DEFAULT_CHANNEL_ID.to_string()),
            Some(DEFAULT_TS_PARSED.to_string()),
            None,
        ),
        None,
        Some(channel(None, None)),
        None,
        Some(file_links()),
    );

    get_slack_message_returns_data_correctly_common(
        message_response,
        user_response,
        channel_response,
        team_response,
        url,
        feature_flags,
        expected_return_data,
    )
    .await;
}

#[wasm_bindgen_test]
async fn get_slack_message_returns_data_correctly_with_file_data_and_user_and_team_flag_set_only() {
    let message_response = message_response(
        Some(true),
        None,
        Some(messages(
            vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
            None,
            None,
            Some(files()),
        )),
    );
    let user_response = Some(user_response(Some(true), None, Some(user(None))));
    let channel_response = None;
    let team_response = Some(team_response(Some(true), None, Some(team())));
    let url = url(
        Some(DEFAULT_CHANNEL_ID.to_string()),
        Some(DEFAULT_TS.to_string()),
        None,
    );
    let feature_flags = feature_flags(true, false, true, true);
    let expected_return_data = obsidian_slack_components(
        message_and_thread(
            messages(
                vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
                Some(user(Some(team()))),
                None,
                Some(files()),
            ),
            messages(
                vec![(DEFAULT_TS_PARSED.to_string(), DEFAULT_THREAD_TS.to_string())],
                Some(user(Some(team()))),
                None,
                Some(files()),
            ),
        ),
        file_name(
            Some(DEFAULT_CHANNEL_ID.to_string()),
            Some(DEFAULT_TS_PARSED.to_string()),
            None,
        ),
        Some(Users(
            vec![(DEFAULT_USER_ID.to_string(), user(Some(team())))]
                .into_iter()
                .collect(),
        )),
        None,
        Some(Teams(
            vec![(DEFAULT_TEAM_ID.to_string(), team())]
                .into_iter()
                .collect(),
        )),
        Some(file_links()),
    );

    get_slack_message_returns_data_correctly_common(
        message_response,
        user_response,
        channel_response,
        team_response,
        url,
        feature_flags,
        expected_return_data,
    )
    .await;
}

async fn get_slack_message_returns_error_messages_correctly_base(
    the_message_response: Option<MessageResponse>,
    user_response: Option<UserResponse>,
    channel_response: Option<ChannelResponse>,
    team_response: Option<TeamResponse>,
    file_data: Option<String>,
    api_token: String,
    cookie: String,
    url: String,
    feature_flags: SlackHttpClientConfigFeatureFlags,
    expected_error: &str,
) {
    let feature_flags = serde_wasm_bindgen::to_value(&feature_flags).unwrap();

    let request_func = get_mock_request_function(
        the_message_response.unwrap_or_else(|| message_response(Some(true), None, None)),
        user_response,
        channel_response,
        team_response,
    );

    let result = get_slack_message(
        api_token.to_string(),
        cookie.to_string(),
        url,
        feature_flags,
        request_func,
    )
    .await;

    assert!(result.is_string(), "Result was not a string: {:#?}", result);
    assert!(
        result.as_string().unwrap().contains(expected_error),
        "Result did not contain expected error of {:#?} \n instead got {:#?}",
        expected_error,
        result
    )
}

#[wasm_bindgen_test]
async fn get_slack_message_returns_invalid_api_token() {
    let message_response = None;
    let user_response = None;
    let channel_response = None;
    let team_response = None;
    let file_data = None;
    let api_token = "bad_token".to_string();
    let cookie = "xoxd...".to_string();
    let url = url(
        Some(DEFAULT_CHANNEL_ID.to_string()),
        Some(DEFAULT_TS.to_string()),
        None,
    );
    let feature_flags = feature_flags(false, false, false, false);
    let expected_error = "InvalidSlackApiToken";

    get_slack_message_returns_error_messages_correctly_base(
        message_response,
        user_response,
        channel_response,
        team_response,
        file_data,
        api_token,
        cookie,
        url,
        feature_flags,
        expected_error,
    )
    .await;
}

#[wasm_bindgen_test]
async fn get_slack_message_returns_invalid_api_cookie() {
    let message_response = None;
    let user_response = None;
    let channel_response = None;
    let team_response = None;
    let file_data = None;
    let api_token = "xoxc...".to_string();
    let cookie = "...".to_string();
    let url = url(
        Some(DEFAULT_CHANNEL_ID.to_string()),
        Some(DEFAULT_TS.to_string()),
        None,
    );
    let feature_flags = feature_flags(false, false, false, false);
    let expected_error = "InvalidSlackApiCookie";

    get_slack_message_returns_error_messages_correctly_base(
        message_response,
        user_response,
        channel_response,
        team_response,
        file_data,
        api_token,
        cookie,
        url,
        feature_flags,
        expected_error,
    )
    .await;
}

#[wasm_bindgen_test]
async fn get_slack_message_returns_missing_channel_id() {
    let message_response = None;
    let user_response = None;
    let channel_response = None;
    let team_response = None;
    let file_data = None;
    let api_token = "xoxc...".to_string();
    let cookie = "xoxd...".to_string();
    let url = url(None, Some(DEFAULT_TS.to_string()), None);
    let feature_flags = feature_flags(false, false, false, false);
    let expected_error = "ChannelIdNotFoundInPathSegments";

    get_slack_message_returns_error_messages_correctly_base(
        message_response,
        user_response,
        channel_response,
        team_response,
        file_data,
        api_token,
        cookie,
        url,
        feature_flags,
        expected_error,
    )
    .await;
}

#[wasm_bindgen_test]
async fn get_slack_message_returns_missing_timestamp() {
    let message_response = None;
    let user_response = None;
    let channel_response = None;
    let team_response = None;
    let file_data = None;
    let api_token = "xoxc...".to_string();
    let cookie = "xoxd...".to_string();
    let url = url(Some(DEFAULT_CHANNEL_ID.to_string()), None, None);
    let feature_flags = feature_flags(false, false, false, false);
    let expected_error = "TimestampNotFound";

    get_slack_message_returns_error_messages_correctly_base(
        message_response,
        user_response,
        channel_response,
        team_response,
        file_data,
        api_token,
        cookie,
        url,
        feature_flags,
        expected_error,
    )
    .await;
}

#[wasm_bindgen_test]
async fn get_slack_message_returns_invalid_message_response() {
    let message_response = Some(message_response(Some(false), None, None));
    let user_response = None;
    let channel_response = None;
    let team_response = None;
    let file_data = None;
    let api_token = "xoxc...".to_string();
    let cookie = "xoxd...".to_string();
    let url = url(
        Some(DEFAULT_CHANNEL_ID.to_string()),
        Some(DEFAULT_TS.to_string()),
        None,
    );
    let feature_flags = feature_flags(false, false, false, false);
    let expected_error = "InvalidMessageResponse";

    get_slack_message_returns_error_messages_correctly_base(
        message_response,
        user_response,
        channel_response,
        team_response,
        file_data,
        api_token,
        cookie,
        url,
        feature_flags,
        expected_error,
    )
    .await;
}

#[wasm_bindgen_test]
async fn get_slack_message_returns_invalid_user_response() {
    let message_response = Some(message_response(
        Some(true),
        None,
        Some(messages(
            vec![(DEFAULT_TS_PARSED.to_string(), "".to_string())],
            None,
            None,
            None,
        )),
    ));
    let user_response = Some(user_response(Some(false), None, None));
    let channel_response = None;
    let team_response = None;
    let file_data = None;
    let api_token = "xoxc...".to_string();
    let cookie = "xoxd...".to_string();
    let url = url(
        Some(DEFAULT_CHANNEL_ID.to_string()),
        Some(DEFAULT_TS.to_string()),
        None,
    );
    let feature_flags = feature_flags(true, false, false, false);
    let expected_error = "InvalidUserResponse";

    get_slack_message_returns_error_messages_correctly_base(
        message_response,
        user_response,
        channel_response,
        team_response,
        file_data,
        api_token,
        cookie,
        url,
        feature_flags,
        expected_error,
    )
    .await;
}

#[wasm_bindgen_test]
async fn get_slack_message_returns_invalid_channel_response() {
    let message_response = Some(message_response(
        Some(true),
        None,
        Some(messages(
            vec![(DEFAULT_TS_PARSED.to_string(), "".to_string())],
            None,
            None,
            None,
        )),
    ));
    let channel_response = Some(channel_response(Some(false), None, None));
    let user_response = None;
    let team_response = None;
    let file_data = None;
    let api_token = "xoxc...".to_string();
    let cookie = "xoxd...".to_string();
    let url = url(
        Some(DEFAULT_CHANNEL_ID.to_string()),
        Some(DEFAULT_TS.to_string()),
        None,
    );
    let feature_flags = feature_flags(false, true, false, false);
    let expected_error = "InvalidChannelResponse";

    get_slack_message_returns_error_messages_correctly_base(
        message_response,
        user_response,
        channel_response,
        team_response,
        file_data,
        api_token,
        cookie,
        url,
        feature_flags,
        expected_error,
    )
    .await;
}

#[wasm_bindgen_test]
async fn get_slack_message_returns_invalid_team_response() {
    let message_response = Some(message_response(
        Some(true),
        None,
        Some(messages(
            vec![(DEFAULT_TS_PARSED.to_string(), "".to_string())],
            None,
            None,
            None,
        )),
    ));
    let user_response = Some(user_response(Some(true), None, Some(user(None))));
    let channel_response = None;
    let team_response = Some(team_response(Some(false), None, None));
    let file_data = None;
    let api_token = "xoxc...".to_string();
    let cookie = "xoxd...".to_string();
    let url = url(
        Some(DEFAULT_CHANNEL_ID.to_string()),
        Some(DEFAULT_TS.to_string()),
        None,
    );
    let feature_flags = feature_flags(true, false, true, false);
    let expected_error = "InvalidTeamResponse";

    get_slack_message_returns_error_messages_correctly_base(
        message_response,
        user_response,
        channel_response,
        team_response,
        file_data,
        api_token,
        cookie,
        url,
        feature_flags,
        expected_error,
    )
    .await;
}
