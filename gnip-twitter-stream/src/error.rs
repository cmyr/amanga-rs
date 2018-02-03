use std::io;
use std::string::FromUtf8Error;
use reqwest::{StatusCode, Error as ReqError};
use serde_json::Error as JsonError;

use tweet::Tweet;

#[derive(Debug)]
/// Error that occurs when connecting to a url.
pub enum ConnectionError {
    Http(ReqError),
    UnexpectedStatus(StatusCode),
}

#[derive(Debug)]
/// Error that occurs during the course of a connection.
pub enum StreamError {
    Io(io::Error),
    Utf8(FromUtf8Error),
    Disconnect,
    Timeout,
    Json(JsonError),
}

pub type StreamResult = Result<Tweet, StreamError>;

impl From<io::Error> for StreamError {
    fn from(error: io::Error) -> StreamError {
        StreamError::Io(error)
    }
}

impl From<FromUtf8Error> for StreamError {
    fn from(error: FromUtf8Error) -> StreamError {
        StreamError::Utf8(error)
    }
}

impl From<ReqError> for ConnectionError {
    fn from(error: ReqError) -> ConnectionError {
        ConnectionError::Http(error)
    }
}

impl From<JsonError> for StreamError {
    fn from(error: JsonError) -> StreamError {
        StreamError::Json(error)
    }
}

impl StreamError {
    pub fn is_disconnect(&self) -> bool {
        match *self {
            StreamError::Disconnect | StreamError::Io(_) => true,
            _ => false,
        }
    }
}

