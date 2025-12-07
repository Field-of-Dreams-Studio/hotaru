//! Transport implementations for different HTTP versions.

use std::any::Any;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use hotaru_core::connection::Transport;

// ============================================================================
// Unified Hyper Transport for HTTP/1.1
// ============================================================================

#[derive(Clone)]
pub struct HyperTransport {
    connection_id: i128,
    version: HttpVersion,
    keep_alive: bool,
    request_count: u64,
}

#[derive(Clone, Debug)]
pub enum HttpVersion {
    Http1_0,
    Http1_1,
    Http2,
    Http3,
}

impl HyperTransport {
    pub fn new_http1() -> Self {
        Self {
            connection_id: generate_connection_id(),
            version: HttpVersion::Http1_1,
            keep_alive: true,
            request_count: 0,
        }
    }
    
    /// Convert this HTTP/1.1 transport for WebSocket upgrade
    pub fn into_websocket_transport(self) -> crate::websocket::WebSocketTransport {
        crate::websocket::WebSocketTransport::from_http1(self.connection_id)
    }
    
    /// Get the connection ID for protocol switching
    pub fn connection_id(&self) -> i128 {
        self.connection_id
    }
}

impl Transport for HyperTransport {
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

// ============================================================================
// HTTP/2 Transport with Stream Management
// ============================================================================

#[derive(Clone)]
pub struct Http2Transport {
    connection_id: i128,
    streams: Arc<RwLock<HashMap<u32, StreamState>>>,
    local_settings: Http2Settings,
    remote_settings: Http2Settings,
    next_stream_id: Arc<RwLock<u32>>,
}

#[derive(Clone, Debug)]
pub struct Http2Settings {
    pub header_table_size: u32,
    pub enable_push: bool,
    pub max_concurrent_streams: Option<u32>,
    pub initial_window_size: u32,
    pub max_frame_size: u32,
    pub max_header_list_size: Option<u32>,
}

impl Default for Http2Settings {
    fn default() -> Self {
        Self {
            header_table_size: 4096,
            enable_push: true,
            max_concurrent_streams: Some(100),
            initial_window_size: 65535,
            max_frame_size: 16384,
            max_header_list_size: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StreamState {
    pub id: u32,
    pub state: StreamLifecycleState,
    pub window_size: i32,
}

#[derive(Clone, Debug)]
pub enum StreamLifecycleState {
    Idle,
    Open,
    HalfClosedLocal,
    HalfClosedRemote,
    Closed,
    Upgraded, // Stream has been upgraded to WebSocket
}

impl Http2Transport {
    pub fn new() -> Self {
        Self {
            connection_id: generate_connection_id(),
            streams: Arc::new(RwLock::new(HashMap::new())),
            local_settings: Http2Settings::default(),
            remote_settings: Http2Settings::default(),
            next_stream_id: Arc::new(RwLock::new(1)),
        }
    }
    
    pub fn create_stream(&self) -> u32 {
        let mut next_id = self.next_stream_id.write().unwrap();
        let stream_id = *next_id;
        *next_id += 2; // HTTP/2 uses odd numbers for client, even for server
        
        let mut streams = self.streams.write().unwrap();
        streams.insert(stream_id, StreamState {
            id: stream_id,
            state: StreamLifecycleState::Open,
            window_size: self.local_settings.initial_window_size as i32,
        });
        
        stream_id
    }
    
    /// Convert a specific HTTP/2 stream for WebSocket upgrade
    pub fn upgrade_stream_to_websocket(&self, stream_id: u32) -> crate::websocket::WebSocketTransport {
        // Mark the stream as upgraded in our state
        if let Ok(mut streams) = self.streams.write() {
            if let Some(stream) = streams.get_mut(&stream_id) {
                stream.state = StreamLifecycleState::Upgraded;
            }
        }
        
        crate::websocket::WebSocketTransport::from_http2_stream(self.connection_id, stream_id)
    }
    
    /// Get the connection ID for protocol switching
    pub fn connection_id(&self) -> i128 {
        self.connection_id
    }
}

impl Transport for Http2Transport {
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

// ============================================================================
// HTTP/3 Transport with QUIC Integration
// ============================================================================

#[derive(Clone)]
pub struct Http3Transport {
    connection_id: i128,
    streams: Arc<RwLock<HashMap<u64, Http3StreamState>>>,
    settings: Http3Settings,
}

#[derive(Clone, Debug)]
pub struct Http3Settings {
    pub max_field_section_size: Option<u64>,
    pub max_table_capacity: Option<u64>,
    pub blocked_streams: Option<u64>,
}

impl Default for Http3Settings {
    fn default() -> Self {
        Self {
            max_field_section_size: None,
            max_table_capacity: None,
            blocked_streams: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Http3StreamState {
    pub id: u64,
    pub stream_type: Http3StreamType,
}

#[derive(Clone, Debug)]
pub enum Http3StreamType {
    Request,
    Push,
    Control,
    QpackEncoder,
    QpackDecoder,
}

impl Http3Transport {
    pub fn new() -> Self {
        Self {
            connection_id: generate_connection_id(),
            streams: Arc::new(RwLock::new(HashMap::new())),
            settings: Http3Settings::default(),
        }
    }
}

impl Transport for Http3Transport {
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

// ============================================================================
// Helper Functions
// ============================================================================

fn generate_connection_id() -> i128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as i128;
    timestamp
}