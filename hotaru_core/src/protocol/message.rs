use bytes::BytesMut;
use std::error::Error;

// ----------------------------------------------------------------------------
// Message Trait
// ----------------------------------------------------------------------------

/// Protocol-defined message format.
///
/// A "message" is whatever goes over the wire for your protocol:
/// - HTTP/1.1: Text-based request/response
/// - HTTP/2: Binary frames
/// - WebSocket: Frames with opcode
/// - Custom: Any format you design
pub trait Message: Send + Sync + 'static {
    /// Encodes this message into bytes.
    fn encode(&self, buf: &mut BytesMut) -> Result<(), Box<dyn Error + Send + Sync>>;

    /// Attempts to decode a message from bytes.
    /// Returns Ok(Some(message)) if complete, Ok(None) if needs more data.
    fn decode(buf: &mut BytesMut) -> Result<Option<Self>, Box<dyn Error + Send + Sync>>
    where
        Self: Sized;
}
