use std::{error::Error, fmt::Display};

use url::ParseError;
use wasm_bindgen::JsValue;

#[derive(Debug)]
pub enum SlackHttpClientError {
    ThreadTsWasEmpty,
    InvalidApiToken(String),
    InvalidApiCookie(String),
}

impl Display for SlackHttpClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SlackHttpClientError::ThreadTsWasEmpty => {
                write!(f, "Thread timestamp was empty")
            }
            SlackHttpClientError::InvalidApiToken(error_msg) => {
                write!(f, "Provided api token was invalid: {}", error_msg)
            }
            SlackHttpClientError::InvalidApiCookie(error_msg) => {
                write!(f, "Provided api cookie was invalid: {}", error_msg)
            }
        }
    }
}

impl Error for SlackHttpClientError {}

#[derive(Debug)]
pub enum SlackUrlError {
    UrlParseError(String, ParseError),
    PathSegmentsNotFoundError(String),
    ChannelIdNotFoundError(String),
    TimestampNotFoundError(String),
    ParseTimestampError(String),
}

impl Display for SlackUrlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SlackUrlError::UrlParseError(err_msg, err) => write!(
                    f,
                    "An error occurred while parsing the slack url. Error message: {}. Error: {}",
                    err_msg, err
                ),
            SlackUrlError::PathSegmentsNotFoundError(err_msg) => write!(
                    f,
                    "There was an issue parsing path segments for the url. Error msg: {}",
                    err_msg
                ),
            SlackUrlError::ChannelIdNotFoundError(err_msg) => write!(
                    f,
                    "There was an issue parsing channel ID for the url. Error msg: {}",
                    err_msg
                ),
            SlackUrlError::TimestampNotFoundError(err_msg) => write!(
                    f,
                    "There was an issue parsing the timestamp for the url, timestamp was not found. Error msg: {}",
                    err_msg
                ),
            SlackUrlError::ParseTimestampError(err_msg) => write!(
                    f,
                    "There was an issue parsing the timestamp for the url Error msg: {}",
                    err_msg
                ),
        }
    }
}

impl Error for SlackUrlError {}

#[derive(Debug)]
pub enum SlackError {
    SlackUrlError(SlackUrlError),
    SlackHttpClientError(SlackHttpClientError),
    JsError(JsValue),
}
