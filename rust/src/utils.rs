use crate::{slack_http_client::RequestUrlParam, slack_url::SlackUrl};
use js_sys::Promise;
use serde::Serialize;
use serde_wasm_bindgen::Serializer;
use std::collections::HashSet;
use wasm_bindgen::JsValue;

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub fn create_file_name(slack_url: &SlackUrl) -> String {
    let mut items = vec![slack_url.channel_id.to_string()];
    let mut other_items = vec![
        slack_url
            .thread_ts
            .as_ref()
            .unwrap_or(&slack_url.ts)
            .to_string(),
        slack_url.ts.to_string(),
    ]
    .into_iter()
    .collect::<HashSet<String>>()
    .into_iter()
    .collect::<Vec<String>>();
    other_items.sort();

    items.extend(other_items);
    // .collect::<Vec<String>>()
    items.join("-") + ".json"
}

pub fn curry_request_func(
    request_func: js_sys::Function,
) -> Box<dyn Fn(RequestUrlParam) -> Promise> {
    Box::new(move |params: RequestUrlParam| -> Promise {
        let serializer = Serializer::json_compatible();
        js_sys::Promise::from(
            request_func
                .call1(
                    &JsValue::NULL,
                    &params
                        .serialize(&serializer)
                        .expect("Expected to serialize params, but was unable to. This is a bug"),
                )
                .expect(
                    "Expected to create a js promise in rust, but was unable too. This is a bug",
                ),
        )
    })
}

pub fn top_level_fail(err: &dyn snafu::Error) -> JsValue {
    let message = format!(
        "There was a problem getting slack messages. Error message: {} - Error struct: {:#?}",
        &err, &err
    );
    log::error!("{}", &message);
    JsValue::from_str(&message)
}
