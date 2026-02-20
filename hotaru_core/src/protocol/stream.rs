use std::any::Any;

// ----------------------------------------------------------------------------
// Stream Trait
// ----------------------------------------------------------------------------

/// Protocol-defined stream abstraction.
/// 
/// A "stream" means different things to different protocols:
/// - HTTP/2: Multiplexed request/response pairs
/// - WebSocket: Single bidirectional message stream
/// - Pub/Sub: Topic subscriptions
/// - Game Protocol: Different channels (movement, chat, combat)
pub trait Stream: Send + Sync + 'static {
    /// Returns the stream identifier.
    fn id(&self) -> u32;
    
    /// Returns a reference to the stream as `Any`.
    fn as_any(&self) -> &dyn Any;
    
    /// Returns a mutable reference to the stream as `Any`.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Unit type stream for protocols that don't use streams.
impl Stream for () {
    fn id(&self) -> u32 { 0 }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
