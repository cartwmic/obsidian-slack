use std::str::FromStr;

use tuple_conv::RepeatedTuple;

const THREAD_TS_KEY: &str = "thread_ts";

#[derive(Debug)]
pub struct SlackUrl {
    pub channel_id: String,
    pub ts: String,
    pub thread_ts: Option<String>,
    url: url::Url,
    path_segments: Vec<String>,
}

impl SlackUrl {
    pub fn new(url_string: &str) -> Self {
        let log_prefix = "rust|SlackUrl|new";

        let url = url::Url::from_str(url_string).unwrap();

        let path_segments = SlackUrl::parse_path_segments(&url);
        let channel_id = SlackUrl::parse_channel_id(&path_segments);
        let ts = SlackUrl::parse_ts(&url, &path_segments);
        let thread_ts = SlackUrl::parse_thread_ts(&url);

        let res = SlackUrl {
            channel_id,
            ts,
            thread_ts,
            url,
            path_segments,
        };

        log::info!("{}|slack url={:#?}", log_prefix, res);
        res
    }

    fn parse_path_segments(url: &url::Url) -> Vec<String> {
        url.path_segments()
            .unwrap()
            .collect::<Vec<&str>>()
            .into_iter()
            .map(String::from)
            .collect::<Vec<String>>()
    }

    fn parse_channel_id(path_segments: &Vec<String>) -> String {
        // channel id can be prefixed with 'C', 'D', or 'G'. See https://api.slack.com/docs/conversations-api#shared_channels
        path_segments
            .iter()
            .find(|segment| {
                segment.starts_with('C') || segment.starts_with('D') || segment.starts_with('G')
            })
            .unwrap()
            .to_string()
    }

    fn parse_ts(url: &url::Url, path_segments: &Vec<String>) -> String {
        path_segments
            .iter()
            .find(|segment| segment.starts_with('p'))
            .unwrap()
            .split_terminator('p')
            .last()
            .unwrap()
            .split_at(10)
            .to_vec()
            .join(".")
    }

    fn parse_thread_ts(url: &url::Url) -> Option<String> {
        url.query_pairs()
            .find(|(key, _)| *key == THREAD_TS_KEY)
            .map(|(_, value)| value.to_string())
    }
}
