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
///
/// The associated `BytesMut` is the protocol's chosen buffer representation
/// (e.g. `Vec<u8>`, `bytes::BytesMut`, or a custom growable buffer). It must
/// be:
///
/// - `Default` — so framework code can construct an empty buffer to fill.
/// - `Extend<u8>` — so framework code can write bytes received off the wire
///   into the buffer before calling `decode`.
/// - `AsRef<[u8]>` — so framework code can read the encoded bytes back out
///   to send over the wire.
/// - `AsMut<[u8]>` — so the impl can mutate buffer contents in place.
///
/// These bounds keep the buffer fully opaque outside the impl while still
/// letting the framework feed it and read from it.
pub trait Message: Send + Sync + 'static {
    type BytesMut: AsRef<[u8]>
        + AsMut<[u8]>
        + Default
        + Extend<u8>
        + Send
        + Sync
        + 'static;

    /// Encodes this message into bytes appended to `buf`.
    fn encode(&self, buf: &mut Self::BytesMut) -> Result<(), Box<dyn Error + Send + Sync>>;

    /// Attempts to decode a message from `buf`.
    /// Returns `Ok(Some(message))` if a full message was parsed (and consumed
    /// from `buf`), `Ok(None)` if more bytes are needed.
    fn decode(buf: &mut Self::BytesMut) -> Result<Option<Self>, Box<dyn Error + Send + Sync>>
    where
        Self: Sized;
}
