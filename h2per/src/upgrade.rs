//! HTTP Protocol Upgrade Management
//! 
//! This module handles HTTP-specific protocol upgrades including:
//! - HTTP/1.1 Upgrade mechanism (RFC 7230)
//! - HTTP/2 Extended CONNECT (RFC 8441)
//! - HTTP/2 cleartext upgrade (h2c)

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use bytes::BytesMut;
use hyper::upgrade::Upgraded;
use tokio::sync::RwLock;

// Re-export for convenience
pub use self::manager::UpgradeManager;
pub use self::handoff::ConnectionHandoff;

// ============================================================================
// HTTP Protocol Types
// ============================================================================

/// Supported HTTP protocols for upgrades
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpProtocol {
    /// HTTP/1.0 or HTTP/1.1
    Http1,
    /// HTTP/2 over TCP (h2c) or TLS (h2)
    Http2,
    /// HTTP/3 over QUIC
    Http3,
    /// WebSocket protocol
    WebSocket,
    /// Server-Sent Events
    SSE,
    /// gRPC (over HTTP/2)
    Grpc,
    /// Custom protocol
    Custom(&'static str),
}

impl HttpProtocol {
    /// Get the protocol string for Upgrade headers
    pub fn as_str(&self) -> &str {
        match self {
            HttpProtocol::Http1 => "HTTP/1.1",
            HttpProtocol::Http2 => "h2c",
            HttpProtocol::Http3 => "h3",
            HttpProtocol::WebSocket => "websocket",
            HttpProtocol::SSE => "text/event-stream",
            HttpProtocol::Grpc => "grpc",
            HttpProtocol::Custom(s) => s,
        }
    }
}

// ============================================================================
// Upgrade Context - HTTP-specific upgrade information
// ============================================================================

/// Detailed HTTP upgrade context
#[derive(Debug, Clone)]
pub struct UpgradeContext {
    /// Target protocol for upgrade
    pub target_protocol: HttpProtocol,
    
    /// Type of HTTP upgrade being performed
    pub upgrade_type: HttpUpgradeType,
    
    /// Current state of the upgrade process
    pub state: UpgradeState,
    
    /// Metadata for the upgrade
    pub metadata: UpgradeMetadata,
    
    /// When the upgrade was initiated
    pub initiated_at: Instant,
}

/// HTTP-specific upgrade mechanisms
#[derive(Debug, Clone)]
pub enum HttpUpgradeType {
    /// HTTP/1.1 Upgrade header mechanism (RFC 7230 Section 6.7)
    Http1Upgrade {
        /// The value of the Upgrade header (e.g., "websocket", "h2c")
        upgrade_protocol: String,
        /// Whether 101 Switching Protocols has been sent
        response_sent: bool,
    },
    
    /// HTTP/2 Extended CONNECT for WebSocket (RFC 8441)
    Http2ExtendedConnect {
        /// The HTTP/2 stream ID being upgraded
        stream_id: u32,
        /// The :protocol pseudo-header value
        protocol: String,
    },
    
    /// HTTP/2 cleartext upgrade (h2c)
    Http2Cleartext {
        /// Base64-encoded HTTP/2 SETTINGS frame from HTTP2-Settings header
        http2_settings: Option<String>,
        /// Whether this is a direct h2c connection (no upgrade)
        direct: bool,
    },
    
    /// HTTP/3 WebSocket over CONNECT
    Http3Connect {
        /// The QUIC stream ID
        stream_id: u64,
    },
}

/// State machine for HTTP upgrades
#[derive(Debug, Clone)]
pub enum UpgradeState {
    /// Upgrade has been requested but not yet processed
    Requested,
    
    /// Validating the upgrade request
    Validating,
    
    /// Sending the upgrade response (101, 200, etc.)
    SendingResponse,
    
    /// Waiting for client confirmation (e.g., HTTP/2 preface)
    WaitingClientConfirmation,
    
    /// Performing the connection/stream handoff
    HandoffInProgress,
    
    /// Upgrade completed successfully
    Completed,
    
    /// Upgrade failed with reason
    Failed(String),
    
    /// Upgrade was cancelled
    Cancelled,
}

/// Metadata specific to HTTP upgrades
#[derive(Debug, Clone)]
pub struct UpgradeMetadata {
    /// For WebSocket: Sec-WebSocket-Key from request
    pub websocket_key: Option<String>,
    
    /// For WebSocket: Computed Sec-WebSocket-Accept
    pub websocket_accept: Option<String>,
    
    /// For WebSocket: Negotiated subprotocols
    pub websocket_protocol: Option<String>,
    
    /// For WebSocket: Negotiated extensions
    pub websocket_extensions: Vec<String>,
    
    /// For HTTP/2: Initial SETTINGS frame
    pub h2_settings: Option<Http2Settings>,
    
    /// Additional headers to include in upgrade response
    pub response_headers: HashMap<String, String>,
    
    /// Whether to keep the connection alive after upgrade
    pub keep_alive: bool,
}

/// HTTP/2 SETTINGS for upgrades
#[derive(Debug, Clone)]
pub struct Http2Settings {
    pub header_table_size: Option<u32>,
    pub enable_push: Option<bool>,
    pub max_concurrent_streams: Option<u32>,
    pub initial_window_size: Option<u32>,
    pub max_frame_size: Option<u32>,
    pub max_header_list_size: Option<u32>,
}

impl Default for UpgradeMetadata {
    fn default() -> Self {
        Self {
            websocket_key: None,
            websocket_accept: None,
            websocket_protocol: None,
            websocket_extensions: Vec::new(),
            h2_settings: None,
            response_headers: HashMap::new(),
            keep_alive: true,
        }
    }
}

// ============================================================================
// Upgrade Manager
// ============================================================================

pub mod manager {
    use super::*;
    use crate::context::HyperContext;
    
    /// Manages HTTP protocol upgrades
    pub struct UpgradeManager {
        /// Active upgrades indexed by connection ID
        upgrades: Arc<RwLock<HashMap<i128, UpgradeContext>>>,
        
        /// Upgrade handlers for different protocol combinations
        handlers: HashMap<(HttpProtocol, HttpProtocol), Box<dyn UpgradeHandler>>,
    }
    
    impl UpgradeManager {
        pub fn new() -> Self {
            let mut manager = Self {
                upgrades: Arc::new(RwLock::new(HashMap::new())),
                handlers: HashMap::new(),
            };
            
            // Register default handlers
            manager.register_default_handlers();
            manager
        }
        
        fn register_default_handlers(&mut self) {
            // HTTP/1.1 -> WebSocket
            self.register_handler(
                HttpProtocol::Http1,
                HttpProtocol::WebSocket,
                Box::new(Http1ToWebSocketHandler),
            );
            
            // HTTP/1.1 -> HTTP/2
            self.register_handler(
                HttpProtocol::Http1,
                HttpProtocol::Http2,
                Box::new(Http1ToHttp2Handler),
            );
            
            // HTTP/2 -> WebSocket
            self.register_handler(
                HttpProtocol::Http2,
                HttpProtocol::WebSocket,
                Box::new(Http2ToWebSocketHandler),
            );
        }
        
        pub fn register_handler(
            &mut self,
            from: HttpProtocol,
            to: HttpProtocol,
            handler: Box<dyn UpgradeHandler>,
        ) {
            self.handlers.insert((from, to), handler);
        }
        
        /// Initiate an upgrade
        pub async fn initiate_upgrade(
            &self,
            ctx: &mut HyperContext,
            target_protocol: HttpProtocol,
        ) -> Result<(), String> {
            let from_protocol = ctx.current_protocol();
            
            // Find appropriate handler
            let handler = self.handlers
                .get(&(from_protocol, target_protocol))
                .ok_or_else(|| format!("No upgrade handler for {:?} -> {:?}", from_protocol, target_protocol))?;
            
            // Create upgrade context
            let upgrade_ctx = handler.create_upgrade_context(ctx)?;
            
            // Store in active upgrades
            let connection_id = ctx.connection_id();
            self.upgrades.write().await.insert(connection_id, upgrade_ctx);
            
            // Update connection status with the target protocol
            ctx.set_upgrade_target(target_protocol);
            
            Ok(())
        }
        
        /// Process an upgrade through its states
        pub async fn process_upgrade(
            &self,
            ctx: &mut HyperContext,
        ) -> Result<UpgradeResult, String> {
            let connection_id = ctx.connection_id();
            
            // Get upgrade context
            let upgrade_ctx = self.upgrades.read().await
                .get(&connection_id)
                .cloned()
                .ok_or_else(|| "No active upgrade for connection".to_string())?;
            
            // Get handler
            let from_protocol = ctx.current_protocol();
            let handler = self.handlers
                .get(&(from_protocol, upgrade_ctx.target_protocol))
                .ok_or_else(|| "No handler found".to_string())?;
            
            // Process based on state
            let result = match upgrade_ctx.state {
                UpgradeState::Requested => handler.validate_upgrade(ctx, &upgrade_ctx).await,
                UpgradeState::Validating => handler.prepare_response(ctx, &upgrade_ctx).await,
                UpgradeState::SendingResponse => handler.await_confirmation(ctx, &upgrade_ctx).await,
                UpgradeState::WaitingClientConfirmation => handler.perform_handoff(ctx, &upgrade_ctx).await,
                _ => Ok(UpgradeResult::Continue),
            };
            
            // Update state
            if let Ok(UpgradeResult::StateChange(new_state)) = &result {
                self.upgrades.write().await
                    .get_mut(&connection_id)
                    .map(|ctx| ctx.state = new_state.clone());
            }
            
            result
        }
        
        /// Complete an upgrade
        pub async fn complete_upgrade(
            &self,
            connection_id: i128,
        ) -> Option<UpgradeContext> {
            self.upgrades.write().await.remove(&connection_id)
        }
    }
    
    /// Result of processing an upgrade state
    pub enum UpgradeResult {
        /// Continue processing
        Continue,
        /// Change to new state
        StateChange(UpgradeState),
        /// Upgrade complete, perform handoff
        Handoff(Box<ConnectionHandoff>),
        /// Upgrade failed
        Failed(String),
    }
    
    /// Trait for protocol-specific upgrade handlers
    #[async_trait::async_trait]
    pub trait UpgradeHandler: Send + Sync {
        /// Create initial upgrade context
        fn create_upgrade_context(&self, ctx: &HyperContext) -> Result<UpgradeContext, String>;
        
        /// Validate the upgrade request
        async fn validate_upgrade(&self, ctx: &mut HyperContext, upgrade: &UpgradeContext) -> Result<UpgradeResult, String>;
        
        /// Prepare the upgrade response
        async fn prepare_response(&self, ctx: &mut HyperContext, upgrade: &UpgradeContext) -> Result<UpgradeResult, String>;
        
        /// Wait for client confirmation
        async fn await_confirmation(&self, ctx: &mut HyperContext, upgrade: &UpgradeContext) -> Result<UpgradeResult, String>;
        
        /// Perform the connection handoff
        async fn perform_handoff(&self, ctx: &mut HyperContext, upgrade: &UpgradeContext) -> Result<UpgradeResult, String>;
    }
    
    // Default handlers
    struct Http1ToWebSocketHandler;
    struct Http1ToHttp2Handler;
    struct Http2ToWebSocketHandler;
    
    // Implementation for HTTP/1.1 -> WebSocket
    #[async_trait::async_trait]
    impl UpgradeHandler for Http1ToWebSocketHandler {
        fn create_upgrade_context(&self, ctx: &HyperContext) -> Result<UpgradeContext, String> {
            Ok(UpgradeContext {
                target_protocol: HttpProtocol::WebSocket,
                upgrade_type: HttpUpgradeType::Http1Upgrade {
                    upgrade_protocol: "websocket".to_string(),
                    response_sent: false,
                },
                state: UpgradeState::Requested,
                metadata: UpgradeMetadata::default(),
                initiated_at: std::time::Instant::now(),
            })
        }
        
        async fn validate_upgrade(&self, ctx: &mut HyperContext, upgrade: &UpgradeContext) -> Result<UpgradeResult, String> {
            // Validate WebSocket upgrade headers
            let headers = ctx.request().headers();
            
            // Check Connection: Upgrade
            if !headers.get("Connection")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.to_lowercase().contains("upgrade"))
                .unwrap_or(false) {
                return Err("Missing Connection: Upgrade header".to_string());
            }
            
            // Check Upgrade: websocket
            if !headers.get("Upgrade")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.to_lowercase() == "websocket")
                .unwrap_or(false) {
                return Err("Missing Upgrade: websocket header".to_string());
            }
            
            // Check Sec-WebSocket-Version: 13
            if headers.get("Sec-WebSocket-Version")
                .and_then(|v| v.to_str().ok()) != Some("13") {
                return Err("Invalid WebSocket version".to_string());
            }
            
            // Check for Sec-WebSocket-Key
            if headers.get("Sec-WebSocket-Key").is_none() {
                return Err("Missing Sec-WebSocket-Key".to_string());
            }
            
            Ok(UpgradeResult::StateChange(UpgradeState::Validating))
        }
        
        async fn prepare_response(&self, ctx: &mut HyperContext, upgrade: &UpgradeContext) -> Result<UpgradeResult, String> {
            // Response preparation is done in the endpoint handler
            // which builds the 101 response with proper headers
            Ok(UpgradeResult::StateChange(UpgradeState::SendingResponse))
        }
        
        async fn await_confirmation(&self, ctx: &mut HyperContext, upgrade: &UpgradeContext) -> Result<UpgradeResult, String> {
            // For HTTP/1.1 -> WebSocket, no client confirmation needed after 101
            Ok(UpgradeResult::StateChange(UpgradeState::HandoffInProgress))
        }
        
        async fn perform_handoff(&self, ctx: &mut HyperContext, upgrade: &UpgradeContext) -> Result<UpgradeResult, String> {
            // The actual handoff is handled by hyper::upgrade::on()
            // and the spawned WebSocket handler task
            Ok(UpgradeResult::StateChange(UpgradeState::Completed))
        }
    }
    
    // Placeholder implementations for other handlers
    #[async_trait::async_trait]
    impl UpgradeHandler for Http1ToHttp2Handler {
        fn create_upgrade_context(&self, ctx: &HyperContext) -> Result<UpgradeContext, String> {
            Ok(UpgradeContext {
                target_protocol: HttpProtocol::Http2,
                upgrade_type: HttpUpgradeType::Http2Cleartext {
                    http2_settings: None,
                    direct: false,
                },
                state: UpgradeState::Requested,
                metadata: UpgradeMetadata::default(),
                initiated_at: std::time::Instant::now(),
            })
        }
        
        async fn validate_upgrade(&self, ctx: &mut HyperContext, upgrade: &UpgradeContext) -> Result<UpgradeResult, String> {
            // TODO: Implement h2c validation
            Ok(UpgradeResult::StateChange(UpgradeState::Validating))
        }
        
        async fn prepare_response(&self, ctx: &mut HyperContext, upgrade: &UpgradeContext) -> Result<UpgradeResult, String> {
            Ok(UpgradeResult::StateChange(UpgradeState::SendingResponse))
        }
        
        async fn await_confirmation(&self, ctx: &mut HyperContext, upgrade: &UpgradeContext) -> Result<UpgradeResult, String> {
            // Need to wait for HTTP/2 connection preface
            Ok(UpgradeResult::StateChange(UpgradeState::WaitingClientConfirmation))
        }
        
        async fn perform_handoff(&self, ctx: &mut HyperContext, upgrade: &UpgradeContext) -> Result<UpgradeResult, String> {
            Ok(UpgradeResult::StateChange(UpgradeState::Completed))
        }
    }
    
    #[async_trait::async_trait]
    impl UpgradeHandler for Http2ToWebSocketHandler {
        fn create_upgrade_context(&self, ctx: &HyperContext) -> Result<UpgradeContext, String> {
            Ok(UpgradeContext {
                target_protocol: HttpProtocol::WebSocket,
                upgrade_type: HttpUpgradeType::Http2ExtendedConnect {
                    stream_id: ctx.stream_id.unwrap_or(0),
                    protocol: "websocket".to_string(),
                },
                state: UpgradeState::Requested,
                metadata: UpgradeMetadata::default(),
                initiated_at: std::time::Instant::now(),
            })
        }
        
        async fn validate_upgrade(&self, ctx: &mut HyperContext, upgrade: &UpgradeContext) -> Result<UpgradeResult, String> {
            // TODO: Implement HTTP/2 Extended CONNECT validation
            Ok(UpgradeResult::StateChange(UpgradeState::Validating))
        }
        
        async fn prepare_response(&self, ctx: &mut HyperContext, upgrade: &UpgradeContext) -> Result<UpgradeResult, String> {
            Ok(UpgradeResult::StateChange(UpgradeState::SendingResponse))
        }
        
        async fn await_confirmation(&self, ctx: &mut HyperContext, upgrade: &UpgradeContext) -> Result<UpgradeResult, String> {
            Ok(UpgradeResult::StateChange(UpgradeState::HandoffInProgress))
        }
        
        async fn perform_handoff(&self, ctx: &mut HyperContext, upgrade: &UpgradeContext) -> Result<UpgradeResult, String> {
            Ok(UpgradeResult::StateChange(UpgradeState::Completed))
        }
    }
}

// ============================================================================
// Connection Handoff
// ============================================================================

pub mod handoff {
    use super::*;
    use tokio::net::TcpStream;
    
    /// Resources and information for connection handoff
    pub struct ConnectionHandoff {
        /// Connection ID to preserve
        pub connection_id: i128,
        
        /// The underlying TCP stream (for connection-level upgrades)
        pub tcp_stream: Option<Upgraded>,
        
        /// Buffered but unprocessed data
        pub buffer: BytesMut,
        
        /// For stream-level upgrades (HTTP/2, HTTP/3)
        pub stream_info: Option<StreamHandoff>,
        
        /// Upgrade context with metadata
        pub upgrade_context: UpgradeContext,
    }
    
    /// Information for stream-level handoffs
    pub struct StreamHandoff {
        /// HTTP/2 stream ID
        pub h2_stream_id: Option<u32>,
        
        /// HTTP/3 stream ID  
        pub h3_stream_id: Option<u64>,
        
        /// Whether this stream is bidirectional
        pub bidirectional: bool,
        
        /// Flow control window
        pub window_size: u32,
    }
    
    impl ConnectionHandoff {
        /// Create handoff for HTTP/1.1 -> WebSocket
        pub fn websocket_from_http1(
            connection_id: i128,
            upgraded: Upgraded,
            buffer: BytesMut,
            context: UpgradeContext,
        ) -> Self {
            Self {
                connection_id,
                tcp_stream: Some(upgraded),
                buffer,
                stream_info: None,
                upgrade_context: context,
            }
        }
        
        /// Create handoff for HTTP/2 stream -> WebSocket
        pub fn websocket_from_http2_stream(
            connection_id: i128,
            stream_id: u32,
            buffer: BytesMut,
            context: UpgradeContext,
        ) -> Self {
            Self {
                connection_id,
                tcp_stream: None,
                buffer,
                stream_info: Some(StreamHandoff {
                    h2_stream_id: Some(stream_id),
                    h3_stream_id: None,
                    bidirectional: true,
                    window_size: 65535,
                }),
                upgrade_context: context,
            }
        }
        
        /// Create handoff for HTTP/1.1 -> HTTP/2
        pub fn http2_from_http1(
            connection_id: i128,
            tcp_stream: Upgraded,
            buffer: BytesMut,
            context: UpgradeContext,
        ) -> Self {
            Self {
                connection_id,
                tcp_stream: Some(tcp_stream),
                buffer,
                stream_info: None,
                upgrade_context: context,
            }
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

impl crate::context::HyperContext {
    /// Get current HTTP protocol
    pub fn current_protocol(&self) -> HttpProtocol {
        match self.version {
            crate::context::HttpVersion::Http1_0 | crate::context::HttpVersion::Http1_1 => HttpProtocol::Http1,
            crate::context::HttpVersion::Http2 => HttpProtocol::Http2,
            crate::context::HttpVersion::Http3 => HttpProtocol::Http3,
        }
    }
    
    /// Set upgrade target protocol
    pub fn set_upgrade_target(&mut self, target: HttpProtocol) {
        // Store the target protocol in the upgrade context
        if let Some(ref mut upgrade_ctx) = self.upgrade_context {
            upgrade_ctx.target_protocol = target;
        }
        
        // Signal protocol switch to Hotaru core
        // We'll need to update this to work without TypeId
        self.upgrade_target = Some(target);
    }
    
    /// Get connection ID for upgrade tracking
    pub fn connection_id(&self) -> i128 {
        // This would need to be added to HyperContext
        // For now, generate from timestamp
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i128
    }
}