//! Re-exports of commonly used Hyper types for easy access
//!
//! This module provides direct access to Hyper's types and utilities,
//! allowing users to leverage the full power of the Hyper library.

// HTTP basics
pub use hyper::{
    Method, 
    StatusCode, 
    Version,
    Uri,
    Error as HyperError,
};

// Request and Response
pub use hyper::{
    Request,
    Response,
};

// Headers
pub use hyper::header::{
    HeaderMap,
    HeaderName,
    HeaderValue,
    // Common header names
    ACCEPT,
    ACCEPT_CHARSET,
    ACCEPT_ENCODING,
    ACCEPT_LANGUAGE,
    ACCEPT_RANGES,
    ACCESS_CONTROL_ALLOW_CREDENTIALS,
    ACCESS_CONTROL_ALLOW_HEADERS,
    ACCESS_CONTROL_ALLOW_METHODS,
    ACCESS_CONTROL_ALLOW_ORIGIN,
    ACCESS_CONTROL_EXPOSE_HEADERS,
    ACCESS_CONTROL_MAX_AGE,
    ACCESS_CONTROL_REQUEST_HEADERS,
    ACCESS_CONTROL_REQUEST_METHOD,
    AGE,
    ALLOW,
    ALT_SVC,
    AUTHORIZATION,
    CACHE_CONTROL,
    CONNECTION,
    CONTENT_DISPOSITION,
    CONTENT_ENCODING,
    CONTENT_LANGUAGE,
    CONTENT_LENGTH,
    CONTENT_LOCATION,
    CONTENT_RANGE,
    CONTENT_SECURITY_POLICY,
    CONTENT_SECURITY_POLICY_REPORT_ONLY,
    CONTENT_TYPE,
    COOKIE,
    DATE,
    DNT,
    ETAG,
    EXPECT,
    EXPIRES,
    FORWARDED,
    FROM,
    HOST,
    IF_MATCH,
    IF_MODIFIED_SINCE,
    IF_NONE_MATCH,
    IF_RANGE,
    IF_UNMODIFIED_SINCE,
    LAST_MODIFIED,
    LINK,
    LOCATION,
    MAX_FORWARDS,
    ORIGIN,
    PRAGMA,
    PROXY_AUTHENTICATE,
    PROXY_AUTHORIZATION,
    PUBLIC_KEY_PINS,
    PUBLIC_KEY_PINS_REPORT_ONLY,
    RANGE,
    REFERER,
    REFERRER_POLICY,
    REFRESH,
    RETRY_AFTER,
    SEC_WEBSOCKET_ACCEPT,
    SEC_WEBSOCKET_EXTENSIONS,
    SEC_WEBSOCKET_KEY,
    SEC_WEBSOCKET_PROTOCOL,
    SEC_WEBSOCKET_VERSION,
    SERVER,
    SET_COOKIE,
    STRICT_TRANSPORT_SECURITY,
    TE,
    TRAILER,
    TRANSFER_ENCODING,
    UPGRADE,
    UPGRADE_INSECURE_REQUESTS,
    USER_AGENT,
    VARY,
    VIA,
    WARNING,
    WWW_AUTHENTICATE,
    X_CONTENT_TYPE_OPTIONS,
    X_DNS_PREFETCH_CONTROL,
    X_FRAME_OPTIONS,
    X_XSS_PROTECTION,
};

// Body utilities
pub use http_body_util::{
    BodyExt,
    Full,
    Empty,
    combinators::BoxBody,
};

pub use bytes::{Bytes, BytesMut, Buf, BufMut};

// Extensions for storing custom data
pub use hyper::http::Extensions;

// Re-export our Body type alias
pub use crate::context::Body;

// Builder patterns
pub use hyper::http::{
    request::Builder as RequestBuilder,
    response::Builder as ResponseBuilder,
};

// Utilities for working with bodies
pub mod body {
    use super::*;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use http_body::{Body as HttpBody, Frame};
    
    /// Create an empty body
    pub fn empty() -> BoxBody<Bytes, std::convert::Infallible> {
        Empty::<Bytes>::new().boxed()
    }
    
    /// Create a body from bytes
    pub fn from_bytes(bytes: impl Into<Bytes>) -> BoxBody<Bytes, std::convert::Infallible> {
        Full::new(bytes.into()).boxed()
    }
    
    /// Create a body from a string
    pub fn from_string(s: String) -> BoxBody<Bytes, std::convert::Infallible> {
        Full::new(Bytes::from(s)).boxed()
    }
    
    /// Create a body from a vector
    pub fn from_vec(v: Vec<u8>) -> BoxBody<Bytes, std::convert::Infallible> {
        Full::new(Bytes::from(v)).boxed()
    }
    
    /// Stream body wrapper for custom streaming implementations
    pub struct StreamBody<S> {
        stream: S,
    }
    
    impl<S> StreamBody<S> {
        pub fn new(stream: S) -> Self {
            Self { stream }
        }
    }
    
    // Implement HttpBody for StreamBody if needed for custom streaming
}

// Utilities for working with headers
pub mod headers {
    use super::*;
    
    /// Parse a header value to string
    pub fn to_str(value: &HeaderValue) -> Result<&str, hyper::header::ToStrError> {
        value.to_str()
    }
    
    /// Create a header value from string
    pub fn from_str(s: &str) -> Result<HeaderValue, hyper::header::InvalidHeaderValue> {
        HeaderValue::from_str(s)
    }
    
    /// Create a header value from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<HeaderValue, hyper::header::InvalidHeaderValue> {
        HeaderValue::from_bytes(bytes)
    }
    
    /// Check if a header value contains a substring
    pub fn contains(value: &HeaderValue, needle: &str) -> bool {
        value.to_str().map(|s| s.contains(needle)).unwrap_or(false)
    }
}

// HTTP/2 specific features
pub mod http2 {
    pub use hyper::ext::{Protocol as Http2Protocol};
    
    /// HTTP/2 settings
    pub struct Settings {
        pub enable_push: bool,
        pub initial_window_size: u32,
        pub max_concurrent_streams: u32,
        pub max_frame_size: u32,
        pub max_header_list_size: u32,
    }
    
    impl Default for Settings {
        fn default() -> Self {
            Self {
                enable_push: true,
                initial_window_size: 65535,
                max_concurrent_streams: 100,
                max_frame_size: 16384,
                max_header_list_size: 16384,
            }
        }
    }
}

// Service trait for custom service implementations
pub use hyper::service::Service;

// Client and server connection builders
pub mod conn {
    pub use hyper::server::conn::{http1, http2};
    pub use hyper::client::conn::{
        http1 as client_http1, 
        http2 as client_http2
    };
}

// Utilities
pub use hyper_util::{
    rt::{TokioIo, TokioExecutor, TokioTimer},
    client::legacy::Client,
    server::graceful::GracefulShutdown,
};