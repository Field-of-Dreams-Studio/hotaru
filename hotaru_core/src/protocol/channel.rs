/// Cheap, cloneable handle to one protocol-defined channel.
///
/// `Clone` is a cheap handle clone, not wire duplication. Single-wire
/// protocols implement the trait with internally serialized state (typically
/// `Arc<Mutex<...>>` or an actor-style task). Multiplexed protocols can model
/// logical streams as separate cloned handles backed by one physical
/// connection.
pub trait Channel: Clone + Send + Sync + 'static {
    /// Reflects both local close and observed peer close. Implementations
    /// must update state on EOF, fatal parse/write errors, protocol close
    /// frames, and explicit local close.
    fn is_open(&self) -> bool;

    /// Idempotent. Cloned holders may call without coordination.
    fn close(&self);
}

/// Continuation signal returned by `Protocol::handle` and `Protocol::send`.
///
/// Meaningful when one call processes one unit (HTTP/1 request, single frame).
/// Protocols whose `handle` owns the whole connection lifetime (HTTP/2,
/// WebSocket) may only ever return `Close`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolFlow {
    Continue,
    Close,
} 
