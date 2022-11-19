use std::fmt::Error;

use slack_morphism::prelude::*;
use wasm_bindgen::{prelude::*, convert::ReturnWasmAbi};

mod obsidian;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn get_slack_messages(
    api_token: String,
    files: JsValue,
    save_directory: String,
) -> Result<JsValue, JsError> {
    let client = SlackClient::new(SlackClientHyperConnector::new());

    let token_value: SlackApiTokenValue = api_token.into();
    let token: SlackApiToken = SlackApiToken::new(token_value);

    let session = client.open_session(&token);

    // async parse file for slack link, download message, within api limit

    
}

async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = SlackClient::new(SlackClientHyperConnector::new());

    // Create a Slack session with this token
    // A session is just a lightweight wrapper around your token
    // not to specify it all the time for series of calls.

    // Make your first API call (which is `api.test` here)
    let test: SlackApiTestResponse = session
        .api_test(&SlackApiTestRequest::new().with_foo("Test".into()))
        .await?;

    // Send a simple text message
    let post_chat_req = SlackApiChatPostMessageRequest::new(
        "#general".into(),
        SlackMessageContent::new().with_text("Hey there!".into()),
    );

    let post_chat_resp = session.chat_post_message(&post_chat_req).await?;

    Ok(())
}
