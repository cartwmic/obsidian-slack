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
    #[error("slack url error {0}")]
    SlackUrl(SlackUrlError),
    #[error("slack http client error {0}")]
    SlackHttpClient(SlackHttpClientError),
    #[error("js parsing error {0:#?}")]
    Js(JsValue),
    #[error("js parsed a value that was not an object {0}")]
    JsValueNotObject(String),
    #[error("the response was not ok {0}")]
    ResponseNotOk(String),
    #[error("the response was not an object {0}")]
    ResponseNotAnObject(String),
    #[error("the result was empty {0}")]
    EmptyResult(String),
    #[error("serded wasm parsing errored {0}")]
    SerdeWasmBindgen(serde_wasm_bindgen::Error),
    #[error("Missing user info")]
    MissingUsers,
}
