//! WebSocket protocol implementation using tokio-tungstenite
//!
//! This wraps the tungstenite WebSocket library to work with Hotaru's protocol system.

use std::sync::Arc;
use std::error::Error;
use std::any::Any;
use std::path::Path;
use async_trait::async_trait;
use tokio::io::{BufReader, BufWriter, ReadHalf, WriteHalf};
use tokio_tungstenite::{WebSocketStream, tungstenite};
use tungstenite::protocol::Message as WsMessage;
use futures_util::{StreamExt, SinkExt};

use hotaru_core::{
    app::application::App,
    connection::{Protocol, ProtocolRole, TcpConnectionStream, Message, Transport},
};

use crate::context::HyperContext;

// ============================================================================
// WebSocket Transport - Tracks upgrade source and connection state
// ============================================================================

/// Transport for WebSocket connections
#[derive(Clone)]
pub struct WebSocketTransport {
    /// Connection identifier (preserved from original protocol)
    connection_id: i128,
    
    /// For HTTP/2, tracks which stream was upgraded
    stream_id: Option<u32>,
    
    /// Source of the WebSocket upgrade
    upgraded_from: UpgradeSource,
    
    /// WebSocket-specific state
    message_count: u64,
    is_closing: bool,
}

/// Tracks how the WebSocket connection was established
#[derive(Clone, Debug)]
pub enum UpgradeSource {
    /// Upgraded from HTTP/1.1 via Upgrade header
    Http1,
    /// Upgraded from HTTP/2 stream via Extended CONNECT
    Http2Stream,
    /// Direct WebSocket connection (no upgrade)
    Direct,
}

impl WebSocketTransport {
    /// Create a new WebSocket transport for direct connections
    pub fn new_direct() -> Self {
        Self {
            connection_id: generate_connection_id(),
            stream_id: None,
            upgraded_from: UpgradeSource::Direct,
            message_count: 0,
            is_closing: false,
        }
    }
    
    /// Create from HTTP/1.1 upgrade
    pub fn from_http1(connection_id: i128) -> Self {
        Self {
            connection_id,
            stream_id: None,
            upgraded_from: UpgradeSource::Http1,
            message_count: 0,
            is_closing: false,
        }
    }
    
    /// Create from HTTP/2 stream upgrade
    pub fn from_http2_stream(connection_id: i128, stream_id: u32) -> Self {
        Self {
            connection_id,
            stream_id: Some(stream_id),
            upgraded_from: UpgradeSource::Http2Stream,
            message_count: 0,
            is_closing: false,
        }
    }
    
    pub fn increment_messages(&mut self) {
        self.message_count += 1;
    }
    
    pub fn mark_closing(&mut self) {
        self.is_closing = true;
    }
}

impl Transport for WebSocketTransport {
    fn id(&self) -> i128 {
        self.connection_id
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Generate a unique connection ID
fn generate_connection_id() -> i128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as i128
}

// ============================================================================
// WebSocket Message Wrapper
// ============================================================================

/// Wrapper around tungstenite's Message type to implement Hotaru's Message trait
#[derive(Debug, Clone)]
pub struct WebSocketMessage(pub WsMessage);

impl Message for WebSocketMessage {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Tungstenite handles encoding internally
        // This is just for compatibility with Hotaru's trait
        match &self.0 {
            WsMessage::Text(text) => buf.extend_from_slice(text.as_bytes()),
            WsMessage::Binary(data) => buf.extend_from_slice(data),
            _ => {}
        }
        Ok(())
    }
    
    fn decode(_buf: &mut bytes::BytesMut) -> Result<Option<Self>, Box<dyn Error + Send + Sync>>
    where
        Self: Sized
    {
        // Tungstenite handles decoding internally
        // This is just for compatibility with Hotaru's trait
        Ok(None)
    }
}

// ============================================================================
// WebSocket Protocol Implementation
// ============================================================================

/// WebSocket protocol handler using tokio-tungstenite
#[derive(Clone)]
pub struct WebSocketProtocol {
    role: ProtocolRole,
    transport: WebSocketTransport,
}

impl WebSocketProtocol {
    pub fn new(role: ProtocolRole) -> Self {
        Self { 
            role,
            transport: WebSocketTransport::new_direct(),
        }
    }
    
    /// Create WebSocketProtocol from HTTP/1.1 upgrade
    pub fn from_http1_upgrade(connection_id: i128) -> Self {
        Self {
            role: ProtocolRole::Server,
            transport: WebSocketTransport::from_http1(connection_id),
        }
    }
    
    /// Create WebSocketProtocol from HTTP/2 stream upgrade
    pub fn from_http2_upgrade(connection_id: i128, stream_id: u32) -> Self {
        Self {
            role: ProtocolRole::Server,
            transport: WebSocketTransport::from_http2_stream(connection_id, stream_id),
        }
    }
    
    /// Generate WebSocket accept key (for manual upgrade response)
    pub fn generate_accept_key(key: &str) -> String {
        use sha1::{Sha1, Digest};
        use base64::{Engine, engine::general_purpose::STANDARD};
        
        const WS_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
        
        let mut hasher = Sha1::new();
        hasher.update(key.as_bytes());
        hasher.update(WS_GUID.as_bytes());
        let result = hasher.finalize();
        
        STANDARD.encode(result)
    }
}

#[async_trait]
impl Protocol for WebSocketProtocol {
    type Transport = WebSocketTransport;
    type Stream = ();
    type Message = WebSocketMessage;
    type Context = HyperContext;
    
    fn detect(initial_bytes: &[u8]) -> bool {
        // Check for WebSocket frame structure
        // This is called after upgrade, so we check for WebSocket frames
        if initial_bytes.len() >= 2 {
            let first_byte = initial_bytes[0];
            let opcode = first_byte & 0x0F;
            
            // Valid WebSocket opcodes
            matches!(opcode, 0x0..=0x2 | 0x8..=0xA)
        } else {
            false
        }
    }
    
    fn role(&self) -> ProtocolRole {
        self.role
    }
    
    async fn handle(
        &mut self,
        reader: BufReader<ReadHalf<TcpConnectionStream>>,
        writer: BufWriter<WriteHalf<TcpConnectionStream>>,
        _app: Arc<App>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("🔌 WebSocket protocol handler started!");
        
        // Reunite the reader and writer into a single stream
        let stream = reader.into_inner().unsplit(writer.into_inner());
        
        // Create WebSocket stream from the TCP connection
        // Note: This assumes the upgrade handshake has already been completed
        let ws_stream = WebSocketStream::from_raw_socket(
            stream,
            tungstenite::protocol::Role::Server,
            None,
        ).await;
        
        // Handle WebSocket messages
        self.handle_websocket(ws_stream).await
    }
}

impl WebSocketProtocol {
    /// Handle WebSocket connection after upgrade
    async fn handle_websocket<S>(
        &self,
        mut ws_stream: WebSocketStream<S>,
    ) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    {
        println!("WebSocket connection established!");
        
        // Send welcome message
        ws_stream.send(WsMessage::Text(
            "Welcome to WebSocket server!".to_string()
        )).await?;
        
        // Echo server loop
        while let Some(msg) = ws_stream.next().await {
            match msg? {
                WsMessage::Text(text) => {
                    println!("WebSocket received text: {}", text);
                    
                    // Echo the message back
                    let response = format!("Echo: {}", text);
                    ws_stream.send(WsMessage::Text(response)).await?;
                    
                    // Check for close command
                    if text.trim() == "close" {
                        println!("Closing WebSocket connection");
                        ws_stream.send(WsMessage::Close(None)).await?;
                        break;
                    }
                }
                WsMessage::Binary(data) => {
                    println!("WebSocket received {} bytes of binary data", data.len());
                    
                    // Echo binary data back
                    ws_stream.send(WsMessage::Binary(data)).await?;
                }
                WsMessage::Ping(data) => {
                    println!("WebSocket received ping");
                    ws_stream.send(WsMessage::Pong(data)).await?;
                }
                WsMessage::Pong(_) => {
                    println!("WebSocket received pong");
                }
                WsMessage::Close(_) => {
                    println!("WebSocket received close");
                    break;
                }
                WsMessage::Frame(_) => {
                    // Raw frame - usually not used
                }
            }
        }
        
        println!("WebSocket connection closed");
        Ok(())
    }
}

// ============================================================================
// Upgrade Helper Functions
// ============================================================================

use hyper::{Request, Response, StatusCode};
use hyper::header::{CONNECTION, UPGRADE};
use http_body_util::{Empty, BodyExt};
use bytes::Bytes;
use crate::context::Body;

/// Check if a request is a WebSocket upgrade request (HTTP/1.1) - generic version
pub fn is_websocket_upgrade_generic<T>(request: &Request<T>) -> bool {
    // Check for required headers
    let headers = request.headers();
    
    // Must have Upgrade: websocket
    if let Some(upgrade) = headers.get(UPGRADE) {
        if let Ok(value) = upgrade.to_str() {
            if !value.eq_ignore_ascii_case("websocket") {
                return false;
            }
        } else {
            return false;
        }
    } else {
        return false;
    }
    
    // Must have Connection: Upgrade
    if let Some(connection) = headers.get(CONNECTION) {
        if let Ok(value) = connection.to_str() {
            if !value.to_lowercase().contains("upgrade") {
                return false;
            }
        } else {
            return false;
        }
    } else {
        return false;
    }
    
    // Must have Sec-WebSocket-Key
    headers.get("Sec-WebSocket-Key").is_some()
}

/// Check if a request is a WebSocket upgrade request (HTTP/1.1) - specific for Body type
pub fn is_websocket_upgrade(request: &Request<Body>) -> bool {
    is_websocket_upgrade_generic(request)
}

/// Check if a request is an HTTP/2 Extended CONNECT for WebSocket
pub fn is_http2_websocket_upgrade(request: &Request<Body>) -> bool {
    let headers = request.headers();
    
    // HTTP/2 uses Extended CONNECT method with :protocol pseudo-header
    // Check for :method = CONNECT
    if request.method() != hyper::Method::CONNECT {
        return false;
    }
    
    // Check for :protocol = websocket pseudo-header
    // In HTTP/2, this would be a pseudo-header, but Hyper may expose it differently
    // For now, check for protocol header (implementations vary)
    if let Some(protocol) = headers.get(":protocol") {
        if let Ok(value) = protocol.to_str() {
            return value.eq_ignore_ascii_case("websocket");
        }
    }
    
    // Alternative: Some implementations use regular headers
    if let Some(protocol) = headers.get("sec-websocket-protocol") {
        return protocol.to_str().is_ok();
    }
    
    false
}

/// Build a WebSocket upgrade response for HTTP/1.1
pub fn build_websocket_response(request: &Request<Body>) -> Result<Response<Body>, Box<dyn Error + Send + Sync>> {
    // Get the WebSocket key
    let key = request.headers()
        .get("Sec-WebSocket-Key")
        .and_then(|v| v.to_str().ok())
        .ok_or("Missing Sec-WebSocket-Key")?;
    
    // Generate accept key
    let accept = WebSocketProtocol::generate_accept_key(key);
    
    // Build 101 Switching Protocols response
    let response = Response::builder()
        .status(StatusCode::SWITCHING_PROTOCOLS)
        .header(CONNECTION, "Upgrade")
        .header(UPGRADE, "websocket")
        .header("Sec-WebSocket-Accept", accept)
        .body(Empty::<Bytes>::new().boxed())?;
    
    Ok(response)
}

/// Build a WebSocket response for HTTP/2 Extended CONNECT
pub fn build_http2_websocket_response(request: &Request<Body>) -> Result<Response<Body>, Box<dyn Error + Send + Sync>> {
    // For HTTP/2 Extended CONNECT, we return 200 OK instead of 101
    // The :protocol pseudo-header has already established the protocol switch
    
    let mut builder = Response::builder()
        .status(StatusCode::OK);
    
    // Copy over any WebSocket protocol headers if present
    if let Some(protocol) = request.headers().get("sec-websocket-protocol") {
        builder = builder.header("sec-websocket-protocol", protocol);
    }
    
    // Add any WebSocket extensions if negotiated
    if let Some(extensions) = request.headers().get("sec-websocket-extensions") {
        builder = builder.header("sec-websocket-extensions", extensions);
    }
    
    // HTTP/2 doesn't use Connection: Upgrade, the stream is already established
    let response = builder.body(Empty::<Bytes>::new().boxed())?;
    
    Ok(response)
}

// ============================================================================
// Protocol Switching Support
// ============================================================================

use std::any::TypeId;
use hotaru_core::connection::ConnectionStatus;

/// Create a ConnectionStatus for switching to WebSocket
pub fn switch_to_websocket() -> ConnectionStatus {
    ConnectionStatus::SwitchProtocol(TypeId::of::<WebSocketProtocol>())
}

/// Check if we should upgrade to WebSocket and return the appropriate status
pub fn check_websocket_upgrade(ctx: &HyperContext) -> Option<ConnectionStatus> {
    if is_websocket_upgrade(&ctx.request.inner) {
        Some(switch_to_websocket())
    } else {
        None
    }
}

// ============================================================================
// Upgrade Handling
// ============================================================================

use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;

/// Handle an upgraded WebSocket connection
pub async fn handle_websocket_upgrade(upgraded: Upgraded) {
    println!("🔌 WebSocket upgrade complete, handling connection...");
    
    // Convert Hyper's Upgraded to a tokio AsyncRead+AsyncWrite
    let io = TokioIo::new(upgraded);
    
    // Create WebSocket stream from the upgraded connection
    let ws_stream = WebSocketStream::from_raw_socket(
        io,
        tungstenite::protocol::Role::Server,
        None,
    ).await;
    
    // Create a WebSocket protocol handler
    let protocol = WebSocketProtocol::new(ProtocolRole::Server);
    
    // Handle the WebSocket connection
    if let Err(e) = protocol.handle_websocket(ws_stream).await {
        eprintln!("WebSocket error: {:?}", e);
    }
    
    println!("🔌 WebSocket connection closed");
}

/// Handle WebSocket upgrade specifically for file downloads
pub async fn handle_download_websocket(upgraded: Upgraded) {
    use std::path::{Path, PathBuf};
    use tokio::fs;
    use serde_json::json;
    
    println!("📥 WebSocket download connection established");
    
    // Convert Hyper's Upgraded to a tokio AsyncRead+AsyncWrite
    let io = TokioIo::new(upgraded);
    
    // Create WebSocket stream from the upgraded connection
    let mut ws_stream = WebSocketStream::from_raw_socket(
        io,
        tungstenite::protocol::Role::Server,
        None,
    ).await;
    
    // Define the programfiles directory path
    let programfiles_dir = PathBuf::from("programfiles");
    
    // Handle download commands
    while let Some(msg) = ws_stream.next().await {
        match msg {
            Ok(WsMessage::Text(text)) => {
                // Parse command
                if let Ok(cmd) = serde_json::from_str::<serde_json::Value>(&text) {
                    let command = cmd["command"].as_str().unwrap_or("");
                    
                    match command {
                        "list" => {
                            // List files in programfiles directory
                            match list_files(&programfiles_dir).await {
                                Ok(files) => {
                                    let response = json!({
                                        "type": "file_list",
                                        "files": files
                                    });
                                    ws_stream.send(WsMessage::Text(response.to_string())).await.ok();
                                }
                                Err(e) => {
                                    let error = json!({
                                        "type": "error",
                                        "message": format!("Failed to list files: {}", e)
                                    });
                                    ws_stream.send(WsMessage::Text(error.to_string())).await.ok();
                                }
                            }
                        }
                        "download" => {
                            if let Some(filename) = cmd["filename"].as_str() {
                                // Sanitize filename to prevent directory traversal
                                let filename = Path::new(filename).file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("");
                                
                                let file_path = programfiles_dir.join(filename);
                                
                                // Stream file to client
                                if let Err(e) = stream_file(&mut ws_stream, &file_path, filename).await {
                                    let error = json!({
                                        "type": "error",
                                        "message": format!("Failed to download file: {}", e)
                                    });
                                    ws_stream.send(WsMessage::Text(error.to_string())).await.ok();
                                }
                            }
                        }
                        _ => {
                            let error = json!({
                                "type": "error",
                                "message": format!("Unknown command: {}", command)
                            });
                            ws_stream.send(WsMessage::Text(error.to_string())).await.ok();
                        }
                    }
                }
            }
            Ok(WsMessage::Close(_)) => {
                println!("📥 WebSocket download connection closed");
                break;
            }
            Err(e) => {
                eprintln!("WebSocket error: {:?}", e);
                break;
            }
            _ => {}
        }
    }
}

/// List files in a directory
async fn list_files(dir: &Path) -> Result<Vec<serde_json::Value>, Box<dyn Error + Send + Sync>> {
    use tokio::fs;
    use serde_json::json;
    
    let mut files = Vec::new();
    let mut entries = fs::read_dir(dir).await?;
    
    while let Some(entry) = entries.next_entry().await? {
        let metadata = entry.metadata().await?;
        if metadata.is_file() {
            let file_info = json!({
                "name": entry.file_name().to_string_lossy(),
                "size": metadata.len()
            });
            files.push(file_info);
        }
    }
    
    Ok(files)
}

/// Stream a file over WebSocket
async fn stream_file<S>(
    ws_stream: &mut WebSocketStream<S>,
    file_path: &Path,
    filename: &str,
) -> Result<(), Box<dyn Error + Send + Sync>>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    use tokio::fs::File;
    use tokio::io::AsyncReadExt;
    use serde_json::json;
    
    // Open the file
    let mut file = File::open(file_path).await?;
    let metadata = file.metadata().await?;
    let file_size = metadata.len();
    
    // Send file start message
    let start_msg = json!({
        "type": "file_start",
        "filename": filename,
        "size": file_size
    });
    ws_stream.send(WsMessage::Text(start_msg.to_string())).await?;
    
    // Stream file in chunks
    const CHUNK_SIZE: usize = 8192; // 8KB chunks
    let mut buffer = vec![0u8; CHUNK_SIZE];
    
    loop {
        let bytes_read = file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        
        // Send chunk as binary frame
        let chunk = buffer[..bytes_read].to_vec();
        ws_stream.send(WsMessage::Binary(chunk)).await?;
    }
    
    // Send file complete message
    let complete_msg = json!({
        "type": "file_complete"
    });
    ws_stream.send(WsMessage::Text(complete_msg.to_string())).await?;
    
    println!("✅ File '{}' sent successfully ({} bytes)", filename, file_size);
    
    Ok(())
}