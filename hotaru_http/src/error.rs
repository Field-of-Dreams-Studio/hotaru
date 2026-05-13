use std::fmt;

use hotaru_core::connection::error::ConnectionError;
use hotaru_core::protocol::ProtocolError;

use crate::http::http_value::StatusCode;

#[derive(Debug)]
pub enum HttpError {
    Io(std::io::Error),
    Connection(ConnectionError),
    Status(StatusCode),
    NoRoute(String),
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpError::Io(err) => write!(f, "I/O error: {}", err),
            HttpError::Connection(err) => write!(f, "Connection error: {}", err),
            HttpError::Status(code) => write!(f, "HTTP status error: {:?}", code),
            HttpError::NoRoute(path) => write!(f, "No route matched path: {}", path),
        }
    }
}

impl std::error::Error for HttpError {}

impl ProtocolError for HttpError {
    fn can_continue(&self) -> bool {
        matches!(self, HttpError::Status(_) | HttpError::NoRoute(_))
    }
}

impl From<std::io::Error> for HttpError {
    fn from(err: std::io::Error) -> Self {
        HttpError::Io(err)
    }
}

impl From<ConnectionError> for HttpError {
    fn from(err: ConnectionError) -> Self {
        HttpError::Connection(err)
    }
}

impl From<StatusCode> for HttpError {
    fn from(code: StatusCode) -> Self {
        HttpError::Status(code)
    }
}
