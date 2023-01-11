use crate::slack_url::SlackUrl;
use std::collections::HashSet;

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
