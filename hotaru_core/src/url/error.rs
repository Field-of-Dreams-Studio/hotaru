use std::fmt;

use super::PathPattern;

/// Errors produced by the URL tree during registration or traversal setup.
#[derive(Debug, Clone, PartialEq)]
pub enum UrlError {
    ChildAlreadyExists(PathPattern),
    ChildNotFound(PathPattern),
    InvalidPath(String),
    ParseError(String),
    DepthLimitExceeded {
        max: u32,
        actual: usize,
    },
    NotImplemented(&'static str),
    /// No protocol of the requested type is registered in this registry.
    ProtocolNotFound,
}

impl fmt::Display for UrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UrlError::ChildAlreadyExists(pattern) => {
                write!(f, "child already exists: {}", pattern)
            }
            UrlError::ChildNotFound(pattern) => write!(f, "child not found: {}", pattern),
            UrlError::InvalidPath(path) => write!(f, "invalid path: {}", path),
            UrlError::ParseError(error) => write!(f, "parse error: {}", error),
            UrlError::DepthLimitExceeded { max, actual } => {
                write!(f, "depth limit exceeded: max={}, actual={}", max, actual)
            }
            UrlError::NotImplemented(feature) => write!(f, "not implemented: {}", feature),
            UrlError::ProtocolNotFound => write!(f, "protocol not found in registry"),
        }
    }
}

impl std::error::Error for UrlError {}
