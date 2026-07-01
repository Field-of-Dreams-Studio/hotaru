use core::fmt;

#[derive(Debug)]
pub enum ConnectionError {
    /// Underlying `std::io::Error`. Only available under `feature = "std"`
    /// — embedded builds surface transport errors through the transport's
    /// own `Error` associated type instead.
    #[cfg(feature = "std")]
    IoError(std::io::Error),
    TlsError(String),
    ConnectionTimeout,
    HostResolutionFailed(String),
    AuthenticationFailed,
    ConnectionRefused,
    ConnectionClosed,
    ProtocolError(String),
    PoolExhausted,

    PayloadTooLarge,
    InvalidFrameFormat,
    MethodNotAllowed,
    BadRequest(String),
    UnsupportedProtocolVersion,
    FrameDecodingError(String),
    FrameEncodingError(String),
    InternalServerError(String),

    Other(String),
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "std")]
            Self::IoError(err) => write!(f, "I/O error: {}", err),
            Self::TlsError(err) => write!(f, "TLS error: {}", err),
            Self::ConnectionTimeout => write!(f, "Connection timed out"),
            Self::HostResolutionFailed(h) => write!(f, "Failed to resolve host: {}", h),
            Self::AuthenticationFailed => write!(f, "Authentication failed"),
            Self::ConnectionRefused => write!(f, "Connection refused"),
            Self::ConnectionClosed => write!(f, "Connection closed unexpectedly"),
            Self::ProtocolError(err) => write!(f, "Protocol error: {}", err),
            Self::PoolExhausted => write!(f, "Connection pool exhausted"),

            Self::PayloadTooLarge => write!(f, "Payload too large"),
            Self::InvalidFrameFormat => write!(f, "Invalid frame format"),
            Self::MethodNotAllowed => write!(f, "Method not allowed"),
            Self::BadRequest(err) => write!(f, "Bad request: {}", err),
            Self::UnsupportedProtocolVersion => write!(f, "Unsupported protocol version"),
            Self::FrameDecodingError(err) => write!(f, "Frame decoding error: {}", err),
            Self::FrameEncodingError(err) => write!(f, "Frame encoding error: {}", err),
            Self::InternalServerError(err) => write!(f, "Internal server error: {}", err),

            Self::Other(err) => write!(f, "Other error: {}", err),
        }
    }
}

impl core::error::Error for ConnectionError {}

#[cfg(feature = "std")]
impl From<std::io::Error> for ConnectionError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

pub type Result<T> = core::result::Result<T, ConnectionError>;
