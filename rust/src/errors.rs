// use std::{error::Error, fmt::Display};

use thiserror::Error;
use url::ParseError;
use wasm_bindgen::JsValue;

#[derive(Debug, Error)]
pub enum SlackHttpClientError {
    #[error("Provided api token was invalid: {0}")]
    InvalidApiToken(String),
    #[error("Provided api cookie was invalid: {0}")]
    InvalidApiCookie(String),
}

#[derive(Debug, Error)]
pub enum SlackUrlError {
    #[error("An error occurred while parsing the slack url. Error message: {0}. Error: {1}")]
    UrlParse(String, ParseError),
    #[error("There was an issue parsing path segments for the url. Error msg: {0}")]
    PathSegmentsNotFound(String),
    #[error("There was an issue parsing channel ID for the url. Error msg: {0}")]
    ChannelIdNotFound(String),
    #[error("There was an issue parsing the timestamp for the url, timestamp was not found. Error msg: {0}")]
    TimestampNotFound(String),
    #[error("There was an issue parsing the timestamp for the url Error msg: {0}")]
    ParseTimestamp(String),
}

#[derive(Debug, Error)]
pub enum SlackError {
    #[error("{0}")]
    SlackUrl(SlackUrlError),
    #[error("{0}")]
    SlackHttpClient(SlackHttpClientError),
    #[error("{0:#?}")]
    Js(JsValue),
    #[error("{0}")]
    JsValueNotObject(String),
    #[error("{0}")]
    ResponseNotOk(String),
    #[error("{0}")]
    ResponseNotAnObject(String),
    #[error("{0}")]
    EmptyResult(String),
    #[error("{0}")]
    SerdeWasmBindgen(serde_wasm_bindgen::Error),
    #[error("")]
    MissingUsers,
}
