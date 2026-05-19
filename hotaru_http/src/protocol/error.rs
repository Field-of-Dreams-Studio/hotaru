use std::fmt;

use hotaru_core::connection::error::ConnectionError;
use hotaru_core::protocol::ProtocolError;

use crate::message::http_value::StatusCode;

/// Comprehensive HTTP error type covering all standard error conditions.
///
/// This enum provides fine-grained error variants for HTTP protocol handling,
/// including I/O errors, connection errors, parsing errors, security violations,
/// and routing failures. Each variant carries contextual information where
/// appropriate.
#[derive(Debug)]
pub enum HttpError {
    // ── I/O & Connection ──────────────────────────────────────────────
    /// Low-level I/O error (read/write failures, EOF, etc.)
    Io(std::io::Error),
    /// Connection-level error (timeout, refused, TLS, etc.)
    Connection(ConnectionError),

    // ── Parsing ───────────────────────────────────────────────────────
    /// Failed to parse the HTTP request/response start line.
    ParseError(String),
    /// Invalid or malformed HTTP header.
    InvalidHeader(String),
    /// Invalid or malformed URI in the request line.
    InvalidUri(String),
    /// Error in chunked transfer encoding parsing.
    ChunkError(String),

    // ── Security / Request Validation ─────────────────────────────────
    /// Request entity is too large (413 Payload Too Large).
    PayloadTooLarge,
    /// HTTP method not allowed for this endpoint (405 Method Not Allowed).
    MethodNotAllowed,
    /// Unsupported media type in request (415 Unsupported Media Type).
    UnsupportedMediaType,
    /// Header section exceeds configured size limits.
    HeaderTooLarge,
    /// Too many headers in the request.
    TooManyHeaders,
    /// Header line exceeds maximum allowed length.
    HeaderLineTooLong,

    // ── HTTP Status ───────────────────────────────────────────────────
    /// Wraps a specific HTTP status code (for user-facing error responses).
    Status(StatusCode),

    // ── Routing ───────────────────────────────────────────────────────
    /// No route matched the request path.
    NoRoute(String),

    // ── Timeout ───────────────────────────────────────────────────────
    /// Request processing timed out.
    Timeout,

    // ── Protocol ──────────────────────────────────────────────────────
    /// HTTP version not supported.
    VersionNotSupported,
    /// Generic protocol violation.
    ProtocolViolation(String),

    // ── Catch-all ─────────────────────────────────────────────────────
    /// Other / unspecified error.
    Other(String),
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpError::Io(err) => write!(f, "I/O error: {}", err),
            HttpError::Connection(err) => write!(f, "Connection error: {}", err),
            HttpError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            HttpError::InvalidHeader(msg) => write!(f, "Invalid header: {}", msg),
            HttpError::InvalidUri(uri) => write!(f, "Invalid URI: {}", uri),
            HttpError::ChunkError(msg) => write!(f, "Chunked transfer error: {}", msg),
            HttpError::PayloadTooLarge => write!(f, "Payload too large"),
            HttpError::MethodNotAllowed => write!(f, "Method not allowed"),
            HttpError::UnsupportedMediaType => write!(f, "Unsupported media type"),
            HttpError::HeaderTooLarge => write!(f, "Header section too large"),
            HttpError::TooManyHeaders => write!(f, "Too many headers"),
            HttpError::HeaderLineTooLong => write!(f, "Header line too long"),
            HttpError::Status(code) => write!(f, "HTTP status error: {:?}", code),
            HttpError::NoRoute(path) => write!(f, "No route matched path: {}", path),
            HttpError::Timeout => write!(f, "Request timed out"),
            HttpError::VersionNotSupported => write!(f, "HTTP version not supported"),
            HttpError::ProtocolViolation(msg) => write!(f, "Protocol violation: {}", msg),
            HttpError::Other(msg) => write!(f, "HTTP error: {}", msg),
        }
    }
}

impl std::error::Error for HttpError {}

impl ProtocolError for HttpError {
    /// Returns `true` if the connection can continue after this error.
    ///
    /// Recoverable errors (where a response can still be sent) return `true`:
    /// - `Status` — user-defined status response
    /// - `NoRoute` — 404, can send response and continue
    /// - `PayloadTooLarge`, `MethodNotAllowed`, `UnsupportedMediaType` — security checks
    /// - `HeaderTooLarge`, `TooManyHeaders`, `HeaderLineTooLong` — malformed request
    /// - `ParseError`, `InvalidHeader`, `InvalidUri`, `ChunkError` — parsing failures
    /// - `VersionNotSupported`, `ProtocolViolation` — protocol issues
    /// - `Timeout` — timeout
    /// - `Other` — catch-all
    ///
    /// Non-recoverable errors return `false`:
    /// - `Io` — I/O errors usually mean the connection is broken
    /// - `Connection` — connection-level failures
    fn can_continue(&self) -> bool {
        matches!(
            self,
            HttpError::Status(_)
                | HttpError::NoRoute(_)
                | HttpError::PayloadTooLarge
                | HttpError::MethodNotAllowed
                | HttpError::UnsupportedMediaType
                | HttpError::HeaderTooLarge
                | HttpError::TooManyHeaders
                | HttpError::HeaderLineTooLong
                | HttpError::ParseError(_)
                | HttpError::InvalidHeader(_)
                | HttpError::InvalidUri(_)
                | HttpError::ChunkError(_)
                | HttpError::VersionNotSupported
                | HttpError::ProtocolViolation(_)
                | HttpError::Timeout
                | HttpError::Other(_)
        )
    }
}

// ── From impls ────────────────────────────────────────────────────────

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

/// Convert an `HttpError` into the most appropriate HTTP `StatusCode`.
///
/// This is useful when you need to map an `HttpError` to a response status
/// code for error responses.
impl From<&HttpError> for StatusCode {
    fn from(err: &HttpError) -> Self {
        match err {
            HttpError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            HttpError::Connection(_) => StatusCode::BAD_GATEWAY,
            HttpError::ParseError(_) => StatusCode::BAD_REQUEST,
            HttpError::InvalidHeader(_) => StatusCode::BAD_REQUEST,
            HttpError::InvalidUri(_) => StatusCode::BAD_REQUEST,
            HttpError::ChunkError(_) => StatusCode::BAD_REQUEST,
            HttpError::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            HttpError::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            HttpError::UnsupportedMediaType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            HttpError::HeaderTooLarge => StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE,
            HttpError::TooManyHeaders => StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE,
            HttpError::HeaderLineTooLong => StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE,
            HttpError::Status(code) => code.clone(),
            HttpError::NoRoute(_) => StatusCode::NOT_FOUND,
            HttpError::Timeout => StatusCode::REQUEST_TIMEOUT,
            HttpError::VersionNotSupported => StatusCode::HTTP_VERSION_NOT_SUPPORTED,
            HttpError::ProtocolViolation(_) => StatusCode::BAD_REQUEST,
            HttpError::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
