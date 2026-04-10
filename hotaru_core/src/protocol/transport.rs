use std::any::Any;

// ----------------------------------------------------------------------------
// Transport Trait
// ----------------------------------------------------------------------------

/// Protocol-defined connection abstraction.
///
/// This trait represents whatever "connection" means for your protocol.
/// It could be:
/// - A simple wrapper around a TCP connection ID
/// - A stateful connection with authentication and session data
/// - A multiplexed transport managing multiple streams
/// - Anything the protocol needs to track at the connection level
pub trait Transport: Send + Sync + 'static {
    type Id: Send + Sync + 'static; // Connection identifier type (e.g. i128 for TCP) 

    /// Returns an identifier for this connection.
    fn id(&self) -> Self::Id;

    /// Returns a reference to the transport as `Any` for downcasting.
    fn as_any(&self) -> &dyn Any;

    /// Returns a mutable reference to the transport as `Any`.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Unit type transport for protocols that don't need connection state.
impl Transport for () {
    type Id = i128;
    fn id(&self) -> i128 {
        0
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
