use std::{any::Any, fmt};

// ============================================================================
// Error System (keeping the existing error traits)
// ============================================================================

/// High-level, transport-agnostic error classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolErrorKind {
    Io,
    Timeout,
    Frame,
    FlowControl,
    Config,
    Upgrade,
    Closed,
    Unsupported,
    Other,
}

/// Object-safe protocol error trait retained for extensibility.
pub trait ProtocolError: fmt::Debug + fmt::Display + Send + Sync + 'static {
    fn kind(&self) -> ProtocolErrorKind { ProtocolErrorKind::Other }
    fn is_retryable(&self) -> bool { false }
    fn as_any(&self) -> &dyn Any where Self: Sized { self }
}

/// Thin boxed error wrapper used as the canonical error type.
#[derive(Debug)]
pub struct ProtocolErrorBox(pub Box<dyn ProtocolError>);

impl ProtocolErrorBox {
    pub fn new<E: ProtocolError>(e: E) -> Self { Self(Box::new(e)) }
    pub fn kind(&self) -> ProtocolErrorKind { self.0.kind() }
    pub fn is_retryable(&self) -> bool { self.0.is_retryable() }
    // Note: as_any() cannot be called on trait objects due to Sized bound
}

impl fmt::Display for ProtocolErrorBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { fmt::Display::fmt(&*self.0, f) }
}
impl std::error::Error for ProtocolErrorBox {}

impl From<std::io::Error> for ProtocolErrorBox { 
    fn from(e: std::io::Error) -> Self { 
        ProtocolErrorBox::new(IoProtocolError(e)) 
    }
}

/// Canonical IO error wrapper implementing `ProtocolError`.
#[derive(Debug)]
pub struct IoProtocolError(pub std::io::Error);
impl fmt::Display for IoProtocolError { 
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { 
        write!(f, "IO error: {}", self.0) 
    }
}
impl ProtocolError for IoProtocolError { 
    fn kind(&self) -> ProtocolErrorKind { ProtocolErrorKind::Io } 
}

/// Simple static error helper.
#[derive(Debug)]
pub struct StaticProtocolError { 
    pub kind: ProtocolErrorKind, 
    pub msg: &'static str 
}
impl fmt::Display for StaticProtocolError { 
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { 
        f.write_str(self.msg) 
    }
}
impl ProtocolError for StaticProtocolError { 
    fn kind(&self) -> ProtocolErrorKind { self.kind } 
}

pub type ProtocolResult<T> = Result<T, ProtocolErrorBox>;
