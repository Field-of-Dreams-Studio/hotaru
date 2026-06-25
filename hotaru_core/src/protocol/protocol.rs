use core::future::Future;
use core::time::Duration;
use alloc::sync::Arc;
use tokio::io::BufReader;

use crate::{app::common::RuntimeConfig, protocol::ProtocolFlow};
use crate::connection::TransportSpec;
use crate::connection::stream::ConnStream;
use crate::protocol::ProtocolRole;
use crate::url::UrlRoot;

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
///
/// The where-clause ties the context's error type to the transport's IO
/// error, replacing the previous global `From<std::io::Error>` bound on
/// `RequestContext::Error`. Custom transports define their own `IoError`;
/// the context impl provides `From<TS::IoError>` (or `From<std::io::Error>`
/// when the transport is the default TCP one).
pub trait Protocol: Clone + Send + Sync + 'static
where
    <Self::Context as RequestContext>::Error:
        From<<Self::TS as TransportSpec>::IoError>,
{
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

    /// Context type, pinned so its `Channel` equals `Self::Channel`.
    type Context: RequestContext<Channel = Self::Channel>;

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

    /// Tokenize a URL pattern into the framework's token stream.
    ///
    /// Default impl uses [`crate::url::tokenize`]. Protocols with a
    /// non-HTTP URL syntax (MQTT topic filters, gRPC method paths, etc.)
    /// override this to plug in a custom lexer while still emitting
    /// `Vec<RawToken>` for the shared `tokens_to_patterns` stage.
    fn tokenize_url(
        input: &str,
    ) -> Result<Vec<crate::url::RawToken>, crate::url::PatternError>
    where
        Self: Sized,
    {
        crate::url::tokenize(input)
    }

    /// Split an incoming URL/topic literal into segments for the walker.
    ///
    /// Counterpart to [`Protocol::tokenize_url`]: that one handles the
    /// pattern side at registration; this one handles the literal side
    /// at request dispatch.
    ///
    /// The default impl is intentionally minimal — it returns the whole
    /// literal as a single segment. The framework does not assume any
    /// particular separator convention; each protocol declares its own.
    ///
    /// Note for overriders: an empty returned `Vec` makes
    /// [`crate::url::UrlRoot::walk`] consult the root-endpoint slot (the
    /// segmentless slot used by MQTT and any protocol with a meaningful
    /// empty address). A non-empty `Vec` walks the tree.
    fn lit_parser<'a>(input: &'a str) -> Vec<&'a str>
    where
        Self: Sized,
    {
        vec![input]
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
    fn handle(
        channel: &Self::Channel,
        runtime: Arc<RuntimeConfig>,
        root: Arc<UrlRoot<Self::Context, Self::TS>>,
    ) -> impl Future<Output = Result<ProtocolFlow, CtxError<Self>>> + Send;

    /// Client-side: produce a `Self::Channel` for one outbound exchange.
    /// The impl owns whatever sits behind it — a fresh dial through
    /// `outbound.connect()`, a stream carved from a pooled session, a
    /// packet-id slot in a long-lived MQTT connection. None of that is
    /// visible to the caller.
    ///
    /// `&self` so the impl can hang internal pool/session state on the
    /// Protocol instance. `outbound` is the already-built transport
    /// instance from `Client::ensure_outbound`, handed in as an owned
    /// `Arc` so pool-bearing impls can stash it as a cache key without
    /// further allocation.
    fn acquire_channel(
        &self,
        runtime: &Arc<RuntimeConfig>,
        outbound: Arc<<Self::TS as TransportSpec>::Outbound>,
    ) -> impl Future<Output = Result<Self::Channel, CtxError<Self>>> + Send;

    /// Outpoint final handler: send the request in `ctx`, read the response
    /// back into `ctx`, return ctx. Impl reads channel + request + any
    /// safety config from ctx via same-crate accessors on the concrete type.
    fn send(
        ctx: Self::Context,
    ) -> impl Future<Output = Result<Self::Context, CtxError<Self>>> + Send;

    /// Install a channel into a freshly-built context. Impl writes the
    /// channel into Context's private slot via its same-crate accessor.
    fn install_channel(ctx: &mut Self::Context, channel: Self::Channel);
}
