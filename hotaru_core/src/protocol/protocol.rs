use async_trait::async_trait;
use std::{error::Error, sync::Arc, time::Duration};
use tokio::io::BufReader;

use crate::{app::common::RuntimeConfig, protocol::ProtocolFlow};
use crate::connection::TransportSpec;
use crate::connection::stream::ConnStream;
use crate::protocol::ProtocolRole;
use crate::url::{UrlNode, UrlRoot};

use super::{Message, RequestContext, Stream as ProtocolStream, Channel};

// ----------------------------------------------------------------------------
// Protocol Trait
// ----------------------------------------------------------------------------

/// Convenience alias: the error type produced by `P`'s context.
/// Protocol's Error is stored in Context because it's often needed for request handling and middleware, so this alias makes it easier to refer to. 
pub type CtxError<P> = <<P as Protocol>::Context as RequestContext>::Error;

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
    type TS: TransportSpec<Wire = Self::Wire>;

    /// The protocol's connection-level abstraction.
    type Channel: Channel;

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

    /// Returns the default connection-timeout for this protocol.
    ///
    /// Called when [`TimeoutSetting::Inherit`] is configured so the protocol
    /// can supply its own policy. Return `None` for no timeout (suitable for
    /// long-lived protocols such as MQTT), or `Some(d)` for a fixed duration.
    fn default_connection_timeout(&self) -> Option<Duration> {
        Some(Duration::from_secs(30))
    }

    /// Detects if this protocol can handle the connection.
    fn detect(initial_bytes: &[u8]) -> bool
    where
        Self: Sized;

    /// Construct a channel handle from a freshly split wire.
    fn open_channel(
        self,
        reader: BufReader<<<Self::TS as TransportSpec>::Wire as ConnStream>::ReadHalf>,
        writer: <<Self::TS as TransportSpec>::Wire as ConnStream>::WriteHalf,
        meta: <<Self::TS as TransportSpec>::Wire as ConnStream>::Meta,
    ) -> Self::Channel;

    /// Handles a connection with this protocol.
    ///
    /// This is where all protocol logic lives. The implementation should
    /// check `self.role()` to determine whether to act as client or server.
    /// 
    /// The framework calls this in a loop while `channel.is_open()`.
    ///
    /// - Reader and Writer wrapped in Channel 
    /// - Runtime 
    /// - Root URL 
    async fn handle(
        channel: &Self::Channel,
        runtime: Arc<RuntimeConfig>,
        root: Arc<UrlRoot<Self::Context, Self::TS>>,
    ) -> Result<ProtocolFlow, CtxError<Self>>; 

    /// Client-side: send the request currently in `ctx` and read the response.
    /// Invoked by the generated outpoint final handler.
    async fn send(
        channel: &Self::Channel,
        ctx: &mut Self::Context,
        outpoint: &Arc<UrlNode<Self::Context, Self::TS>>,
    ) -> Result<ProtocolFlow, CtxError<Self>>;
}
