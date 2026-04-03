use std::{error::Error, sync::Arc};
use async_trait::async_trait;
use tokio::io::BufReader;

use crate::app::common::RuntimeConfig;
use crate::connection::stream::ConnStream;
use crate::connection::TransportSpec;
use crate::protocol::ProtocolRole;
use crate::url::UrlRoot;

use super::{Message, RequestContext, Stream as ProtocolStream, Transport};

// ----------------------------------------------------------------------------
// Protocol Trait
// ----------------------------------------------------------------------------

/// User-defined protocol handler.
/// 
/// This is the main trait that protocols implement to handle connections.
/// It brings together Transport, Stream, and Message abstractions.
/// 
/// Protocols must be Clone because each connection gets its own instance
/// to maintain per-connection state (like keep-alive, request count, etc.).
#[async_trait]
pub trait Protocol: Clone + Send + Sync + 'static {
    /// The protocol's wire-level connection stream type.
    type Wire: ConnStream;

    /// The transport spec used by this protocol runtime.
    type Spec: TransportSpec<Wire = Self::Wire>;

    /// The protocol's connection-level abstraction.
    type Transport: Transport;
    
    /// The protocol's stream abstraction (use () if no streams).
    type Stream: ProtocolStream;
    
    /// The protocol's message format.
    type Message: Message;
    
    /// The protocol's request context type.
    type Context: RequestContext;

    /// Returns the name of this protocol (for logging and diagnostics). 
    fn name(&self) -> &'static str; 
    
    /// Returns the role of this protocol handler.
    fn role(&self) -> ProtocolRole;
    
    /// Detects if this protocol can handle the connection.
    fn detect(initial_bytes: &[u8]) -> bool 
    where 
        Self: Sized;
    
    /// Handles a connection with this protocol.
    ///
    /// This is where all protocol logic lives. The implementation should
    /// check `self.role()` to determine whether to act as client or server.
    ///
    /// Why this signature is concrete:
    /// - `BufReader<Wire::ReadHalf>`: protocol detection and protocol parsing can
    ///   share one buffered read state without replay/adapters.
    /// - `Wire::WriteHalf` (not buffered here): write buffering policy stays inside
    ///   each protocol implementation (flush behavior is protocol-dependent).
    /// - Concrete wire split types (no generic R/W): each protocol handles exactly
    ///   its wire kind; stream-specific logic remains in the protocol layer.
    async fn handle(
        &mut self,
        reader: BufReader<<Self::Wire as ConnStream>::ReadHalf>,
        writer: <Self::Wire as ConnStream>::WriteHalf,
        config: <Self::Wire as ConnStream>::Meta,
        runtime: Arc<RuntimeConfig>,
        root: Arc<UrlRoot<Self::Context, Self::Spec>>,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
    
    async fn request(
        &mut self,
        reader: BufReader<<Self::Wire as ConnStream>::ReadHalf>,
        writer: <Self::Wire as ConnStream>::WriteHalf,
        config: <Self::Wire as ConnStream>::Meta,
        runtime: Arc<RuntimeConfig>,
        root: Arc<UrlRoot<Self::Context, Self::Spec>>,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}
