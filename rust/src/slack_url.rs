use std::str::FromStr;

use do_notation::m;
use tuple_conv::RepeatedTuple;

use crate::{
    errors::{SlackError, SlackUrlError},
    slack_http_client::SlackApiQueryParams,
};

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct SlackUrl {
    pub channel_id: String,
    pub ts: String,
    pub thread_ts: Option<String>,
    url: url::Url,
    path_segments: Vec<String>,
}

impl SlackUrl {
    pub fn new(url_string: &str) -> Result<SlackUrl, SlackError> {
        m! {
            url <- url::Url::from_str(url_string).map_err(|parse_err| SlackError::SlackUrl(SlackUrlError::UrlParse(
                format!(
                    "There was an issue parsing the following slack url: {}",
                    url_string
                ),
                parse_err,
            )));
            path_segments <- SlackUrl::parse_path_segments(&url);
            channel_id <- SlackUrl::parse_channel_id(&path_segments);
            ts <- SlackUrl::parse_ts(&url, &path_segments);
            thread_ts <- Ok(SlackUrl::parse_thread_ts(&url));
            return SlackUrl {
                channel_id,
                ts,
                thread_ts,
                url,
                path_segments
            };
        }
    }

    fn parse_path_segments(url: &url::Url) -> Result<Vec<String>, SlackError> {
        match url.path_segments() {
            Some(segments) => Ok(segments
                .collect::<Vec<&str>>()
                .into_iter()
                .map(String::from)
                .collect::<Vec<String>>()),
            None => Err(SlackError::SlackUrl(SlackUrlError::PathSegmentsNotFound(
                format!("No path segments found for url: {}", url),
            ))),
        }
    }

    fn parse_channel_id(path_segments: &Vec<String>) -> Result<String, SlackError> {
        // channel id can be prefixed with 'C', 'D', or 'G'. See https://api.slack.com/docs/conversations-api#shared_channels
        match path_segments
            .iter()
            .find(|segment| {
                segment.starts_with('C') || segment.starts_with('D') || segment.starts_with('G')
            }) {
                Some(found) => Ok(found.to_string()),
                None => Err(SlackError::SlackUrl(SlackUrlError::ChannelIdNotFound(format!("No channel ID found. Channel id must strat with 'C', 'D', or 'G'. path segments: {:#?}", path_segments))))
            }
    }

    fn parse_ts(url: &url::Url, path_segments: &Vec<String>) -> Result<String, SlackError> {
        match path_segments
            .iter()
            .find(|segment| segment.starts_with('p'))
        {
            Some(segment) => match segment.split_terminator('p').last() {
                Some(item) => Ok(item.split_at(10).to_vec().join(".")),
                None => Err(SlackError::SlackUrl(SlackUrlError::ParseTimestamp(
                    format!("url= {}. path segments= {:#?}", url, path_segments),
                ))),
            },
            None => Err(SlackError::SlackUrl(SlackUrlError::TimestampNotFound(
                format!("url= {}. path segments= {:#?}", url, path_segments),
            ))),
        }
    }

    fn parse_thread_ts(url: &url::Url) -> Option<String> {
        url.query_pairs()
            .find(|(key, _)| *key == SlackApiQueryParams::thread_ts.to_string())
            .map(|(_, value)| value.to_string())
    }
}
