//! Unified protocol abstraction with trait-based design.
//! 
//! This module provides a flexible, protocol-agnostic system where protocols
//! define their own abstractions for transport, streams, and messages.

use std::{any::Any, fmt, sync::Arc, error::Error};
use bytes::BytesMut;
use async_trait::async_trait;

use crate::{
    app::application::App,
    connection::{TcpReader, TcpWriter},
};

// ============================================================================
// Error System (keeping the existing error traits)
// ============================================================================

/// High-level, transport‑agnostic error classification.
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

/// Object‑safe protocol error trait retained for extensibility.
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

// ============================================================================
// Core Protocol Traits
// ============================================================================

/// Role of the protocol handler - server or client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolRole {
    /// Server role - accepts connections and handles requests
    Server,
    /// Client role - initiates connections and sends requests  
    Client,
}

/// Protocol index type for efficient registry lookups.
pub type ProtocolIndex = u16;

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
    /// Returns an identifier for this connection.
    fn id(&self) -> i128;
    
    /// Returns a reference to the transport as `Any` for downcasting.
    fn as_any(&self) -> &dyn Any;
    
    /// Returns a mutable reference to the transport as `Any`.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Unit type transport for protocols that don't need connection state.
impl Transport for () {
    fn id(&self) -> i128 { 0 }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

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

// ----------------------------------------------------------------------------
// RequestContext Trait
// ----------------------------------------------------------------------------

/// Context that flows through request handlers.
/// 
/// This trait links the request/response types that handlers work with.
/// It's the type that flows through `AsyncFinalHandler<C>` and `AsyncMiddleware<C>`.
/// 
/// Both server and client contexts implement this trait, with the role
/// determining the direction of communication.
pub trait RequestContext: Send + 'static {
    /// The request type for this context
    type Request;
    
    /// The response type for this context
    type Response;
    
    /// Handle protocol errors (bad request for server, bad response for client)
    fn handle_error(&mut self);
    
    /// Get the role of this context (Server or Client)
    fn role(&self) -> ProtocolRole;
}

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
    /// The protocol's connection-level abstraction.
    type Transport: Transport;
    
    /// The protocol's stream abstraction (use () if no streams).
    type Stream: Stream;
    
    /// The protocol's message format.
    type Message: Message;
    
    /// The protocol's request context type.
    type Context: RequestContext;
    
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
    /// The method receives TcpReader/TcpWriter which provide buffered I/O
    /// and connection metadata (socket addresses).
    async fn handle(
        &mut self,
        reader: TcpReader,
        writer: TcpWriter,
        app: Arc<App>,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
    
    // TODO: Future client-specific methods for Tx replacement
    // async fn connect(&mut self, addr: &str, app: Arc<App>) -> Result<(), Box<dyn Error>>;
    // async fn request(&mut self, message: Self::Message) -> Result<Self::Message, Box<dyn Error>>;
}

// ============================================================================
// Legacy compatibility types (keeping existing code working)
// ============================================================================

/// Frame type categorization retained from prior design.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameType { 
    Request, 
    Response, 
    Data, 
    Control, 
    Metadata, 
    Unknown 
}

/// Binary frame abstraction (legacy, will be replaced by Message trait).
pub trait Frame: Sized + Send + Sync + 'static {
    fn parse(buf: &mut BytesMut) -> ProtocolResult<Option<Self>>;
    fn encode(&self, buf: &mut BytesMut) -> ProtocolResult<()>;
    fn frame_type(&self) -> FrameType;
    fn stream_id(&self) -> Option<u32> { None }
    fn is_control_frame(&self) -> bool { matches!(self.frame_type(), FrameType::Control) }
    fn requires_response(&self) -> bool { matches!(self.frame_type(), FrameType::Request) }
    fn size(&self) -> usize;
}

/// Lightweight stream configuration placeholder (legacy).
#[derive(Debug, Clone, Default)]
pub struct StreamConfig { 
    pub initial_window: Option<u32>,
    pub priority: Option<u32>,
}

/// Minimal connection context (legacy, to be removed).
pub struct ConnectionContext {
    pub role: ProtocolRole,
    pub reusable: bool,
    pub closed: bool,
}

impl ConnectionContext {
    pub fn mark_for_reuse(&mut self) { self.reusable = true }
    pub fn is_reusable(&self) -> bool { self.reusable }
}

// /// Legacy Protocol trait (will be removed once migration is complete).
// #[async_trait]
// pub trait LegacyProtocol: Send + Sync + 'static {
//     type Frame: Frame;
//     type Config: Clone + Send + Sync + 'static;

//     fn role(&self) -> ProtocolRole;
//     fn test_protocol(initial_bytes: &[u8]) -> bool where Self: Sized;

//     async fn initialize(&mut self, config: Self::Config, ctx: &mut ConnectionContext) -> ProtocolResult<()> { Ok(()) }
//     async fn shutdown(&mut self, ctx: &mut ConnectionContext) -> ProtocolResult<()> { Ok(()) }

//     async fn open_stream(&mut self, cfg: StreamConfig, ctx: &mut ConnectionContext) -> ProtocolResult<u32> { 
//         Err(ProtocolErrorBox::new(StaticProtocolError { 
//             kind: ProtocolErrorKind::Unsupported, 
//             msg: "multiplexing not supported" 
//         })) 
//     }
//     async fn close_stream(&mut self, stream_id: u32, ctx: &mut ConnectionContext) -> ProtocolResult<()> { Ok(()) }

//     async fn process_frame(&mut self, stream_id: u32, frame: Self::Frame, ctx: &mut ConnectionContext) -> ProtocolResult<Option<Self::Frame>> {
//         Ok(None)
//     }

//     async fn read_frame(&mut self, r: &mut BufReader<ReadHalf<TcpConnectionStream>>, ctx: &mut ConnectionContext) -> ProtocolResult<(u32, Self::Frame)> {
//         Err(ProtocolErrorBox::new(StaticProtocolError { 
//             kind: ProtocolErrorKind::Unsupported, 
//             msg: "read_frame not implemented" 
//         }))
//     }

//     async fn write_frame(&mut self, w: &mut BufWriter<WriteHalf<TcpConnectionStream>>, stream_id: u32, frame: Self::Frame, ctx: &mut ConnectionContext) -> ProtocolResult<()> {
//         Err(ProtocolErrorBox::new(StaticProtocolError { 
//             kind: ProtocolErrorKind::Unsupported, 
//             msg: "write_frame not implemented" 
//         }))
//     }

//     async fn negotiate_upgrade(&mut self, target: &str, ctx: &mut ConnectionContext) -> ProtocolResult<Option<Arc<dyn LegacyProtocol<Frame = Self::Frame, Config = Self::Config>>>> {
//         Ok(None)
//     }

//     fn is_reusable(&self, ctx: &ConnectionContext) -> bool { ctx.is_reusable() }
//     fn mark_for_reuse(&self, ctx: &mut ConnectionContext) { ctx.mark_for_reuse(); }
// } 