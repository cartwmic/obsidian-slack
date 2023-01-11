use std::{path, str::FromStr};

use do_notation::m;
use snafu::{ResultExt, Snafu};
use tuple_conv::RepeatedTuple;
use url::{ParseError, Url};

use crate::slack_http_client::SlackApiQueryParams;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An error occurred while parsing the slack url: {url}"))]
    UrlCrateCouldNotParse { url: String, source: ParseError },

    #[snafu(display("Path segments not found in url: {url}"))]
    PathSegmentsNotFound { url: String },

    #[snafu(display(
        "Channel id not found in path segments of url. Channel id must start with 'C', 'D', or 'G'. Path segments: {path_segments}"
    ))]
    ChannelIdNotFoundInPathSegments { path_segments: String },

    #[snafu(display("Timestamp was not found in the url: {url}"))]
    TimestampNotFound { url: String },

    #[snafu(display("There was an issue parsing the timestamp for the url: {url}"))]
    TimestampCouldNotBeParsed { url: String },
}

type Result<T, E = Error> = std::result::Result<T, E>;

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
    pub fn new(url_string: &str) -> Result<SlackUrl> {
        m! {
            url <- url::Url::from_str(url_string).context(UrlCrateCouldNotParseSnafu { url: url_string});
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

    fn parse_path_segments(url: &url::Url) -> Result<Vec<String>> {
        url.path_segments().map_or(
            PathSegmentsNotFoundSnafu { url: url.as_str() }.fail(),
            |segments| {
                Ok(segments
                    .collect::<Vec<&str>>()
                    .into_iter()
                    .map(String::from)
                    .collect::<Vec<String>>())
            },
        )
    }

    fn parse_channel_id(path_segments: &Vec<String>) -> Result<String> {
        // channel id can be prefixed with 'C', 'D', or 'G'. See https://api.slack.com/docs/conversations-api#shared_channels
        path_segments
            .iter()
            .find(|segment| {
                segment.starts_with('C') || segment.starts_with('D') || segment.starts_with('G')
            })
            .map_or(
                ChannelIdNotFoundInPathSegmentsSnafu {
                    path_segments: format!("{:#?}", path_segments),
                }
                .fail(),
                |found| Ok(found.to_string()),
            )
    }

    fn parse_ts(url: &url::Url, path_segments: &Vec<String>) -> Result<String> {
        path_segments
            .iter()
            .find(|segment| segment.starts_with('p'))
            .map_or(
                TimestampNotFoundSnafu { url: url.as_str() }.fail(),
                |segment| {
                    segment.split_terminator('p').last().map_or(
                        TimestampCouldNotBeParsedSnafu { url: url.as_str() }.fail(),
                        |item| Ok(item.split_at(10).to_vec().join(".")),
                    )
                },
            )
    }

    fn parse_thread_ts(url: &url::Url) -> Option<String> {
        url.query_pairs()
            .find(|(key, _)| *key == SlackApiQueryParams::thread_ts.to_string())
            .map(|(_, value)| value.to_string())
    }
}
