//! Hyper-based protocol implementations for HTTP/1, HTTP/2, and HTTP/3.

use std::sync::Arc;
use std::error::Error;
use async_trait::async_trait;
use hyper::server::conn::{http1, http2};
use hyper_util::rt::{TokioIo, TokioExecutor};

use hotaru_core::{
    app::application::App,
    connection::{
        Protocol, Transport, Stream, Message,
        ProtocolRole, TcpConnectionStream,
    },
};
use tokio::io::{BufReader, BufWriter, ReadHalf, WriteHalf};

use crate::transport::{HyperTransport, Http2Transport, Http3Transport};
use crate::stream::{Http2Stream, Http3Stream};
use crate::message::{Http1Message, Http2Message, Http3Message};
use crate::context::HyperContext;
use crate::io_compat::HyperIoCompat;
use crate::service::HotaruService;

// ============================================================================
// HTTP/1.1 Protocol Implementation
// ============================================================================

/// Hyper-based HTTP/1.1 protocol implementation.
#[derive(Clone)]
pub struct HyperHttp1 {
    transport: HyperTransport,
    role: ProtocolRole,
}

impl HyperHttp1 {
    pub fn new(role: ProtocolRole) -> Self {
        Self {
            transport: HyperTransport::new_http1(),
            role,
        }
    }
}

#[async_trait]
impl Protocol for HyperHttp1 {
    type Transport = HyperTransport;
    type Stream = ();  // HTTP/1.1 doesn't have multiplexed streams
    type Message = Http1Message;
    type Context = HyperContext;
    
    fn detect(initial_bytes: &[u8]) -> bool {
        // Check for HTTP/1.x methods
        initial_bytes.starts_with(b"GET ") ||
        initial_bytes.starts_with(b"POST ") ||
        initial_bytes.starts_with(b"PUT ") ||
        initial_bytes.starts_with(b"DELETE ") ||
        initial_bytes.starts_with(b"HEAD ") ||
        initial_bytes.starts_with(b"OPTIONS ") ||
        initial_bytes.starts_with(b"CONNECT ") ||
        initial_bytes.starts_with(b"TRACE ") ||
        initial_bytes.starts_with(b"PATCH ") ||
        // Check for HTTP/1.x response
        initial_bytes.starts_with(b"HTTP/1.")
    }
    
    fn role(&self) -> ProtocolRole {
        self.role
    }
    
    async fn handle(
        &mut self,
        reader: BufReader<ReadHalf<TcpConnectionStream>>,
        writer: BufWriter<WriteHalf<TcpConnectionStream>>,
        app: Arc<App>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match self.role {
            ProtocolRole::Server => {
                // Use buffered readers directly to preserve peeked data
                // Wrap for hyper compatibility while preserving buffered data
                let io = TokioIo::new(HyperIoCompat::new_buffered(reader, writer));
                
                // Create the service that will handle HTTP requests
                let service = HotaruService::<HyperHttp1>::new(app, self.role);
                
                // Build the HTTP/1.1 connection handler
                let conn = http1::Builder::new()
                    // Support HTTP upgrades (e.g., WebSocket)
                    .serve_connection(io, service)
                    .with_upgrades();
                
                // Handle the connection until completion or error
                if let Err(err) = conn.await {
                    eprintln!("HTTP/1.1 server error: {:?}", err);
                    return Err(Box::new(err));
                }
                
                Ok(())
            }
            ProtocolRole::Client => {
                // TODO: Implement client-side HTTP/1.1
                // Would use hyper::client::conn::http1
                unimplemented!("HTTP/1.1 client mode not yet implemented")
            }
        }
    }
}

// ============================================================================
// HTTP/2 Protocol Implementation
// ============================================================================

/// Hyper-based HTTP/2 protocol implementation.
#[derive(Clone)]
pub struct HyperHttp2 {
    transport: Http2Transport,
    role: ProtocolRole,
}

impl HyperHttp2 {
    pub fn new(role: ProtocolRole) -> Self {
        Self {
            transport: Http2Transport::new(),
            role,
        }
    }
}

#[async_trait]
impl Protocol for HyperHttp2 {
    type Transport = Http2Transport;
    type Stream = Http2Stream;
    type Message = Http2Message;
    type Context = HyperContext;
    
    fn detect(initial_bytes: &[u8]) -> bool {
        // Check for HTTP/2 connection preface
        // "PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n"
        initial_bytes.starts_with(b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n") ||
        // Check for direct HTTP/2 over TLS (ALPN negotiated)
        // This would be handled by TLS layer, but we check for HTTP/2 frames
        (initial_bytes.len() >= 9 && 
         initial_bytes[0..3] == [0x00, 0x00, 0x00] && // Frame length
         initial_bytes[3] <= 0x0A) // Valid frame type (0x00-0x0A)
    }
    
    fn role(&self) -> ProtocolRole {
        self.role
    }
    
    async fn handle(
        &mut self,
        reader: BufReader<ReadHalf<TcpConnectionStream>>,
        writer: BufWriter<WriteHalf<TcpConnectionStream>>,
        app: Arc<App>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match self.role {
            ProtocolRole::Server => {
                // Use buffered readers directly to preserve peeked data
                // Wrap for hyper compatibility while preserving buffered data
                let io = TokioIo::new(HyperIoCompat::new_buffered(reader, writer));
                
                // Create the service that will handle HTTP/2 requests
                let service = HotaruService::<HyperHttp2>::new(app, self.role);
                
                // Build the HTTP/2 connection handler
                let mut h2_builder = http2::Builder::new(TokioExecutor::new());
                
                // Configure HTTP/2 settings
                h2_builder
                    .initial_stream_window_size(1024 * 1024)
                    .initial_connection_window_size(1024 * 1024)
                    .max_concurrent_streams(100);
                
                // Enable Extended CONNECT for WebSocket over HTTP/2
                // Note: Extended CONNECT support in Hyper is still evolving
                // This prepares for RFC 8441 support
                
                let conn = h2_builder.serve_connection(io, service);
                
                // Handle the connection until completion or error
                if let Err(err) = conn.await {
                    eprintln!("HTTP/2 server error: {:?}", err);
                    return Err(Box::new(err));
                }
                
                Ok(())
            }
            ProtocolRole::Client => {
                // TODO: Implement client-side HTTP/2
                // Would use hyper::client::conn::http2
                unimplemented!("HTTP/2 client mode not yet implemented")
            }
        }
    }
}

// ============================================================================
// HTTP/3 Protocol Implementation
// ============================================================================

/// Hyper-based HTTP/3 protocol implementation.
#[derive(Clone)]
pub struct HyperHttp3 {
    transport: Http3Transport,
    role: ProtocolRole,
}

impl HyperHttp3 {
    pub fn new(role: ProtocolRole) -> Self {
        Self {
            transport: Http3Transport::new(),
            role,
        }
    }
}

#[async_trait]
impl Protocol for HyperHttp3 {
    type Transport = Http3Transport;
    type Stream = Http3Stream;
    type Message = Http3Message;
    type Context = HyperContext;
    
    fn detect(initial_bytes: &[u8]) -> bool {
        // HTTP/3 runs over QUIC, not TCP
        // This would typically be detected at the transport layer
        // For now, return false as HTTP/3 detection needs QUIC transport
        false
    }
    
    fn role(&self) -> ProtocolRole {
        self.role
    }
    
    async fn handle(
        &mut self,
        _reader: BufReader<ReadHalf<TcpConnectionStream>>,
        _writer: BufWriter<WriteHalf<TcpConnectionStream>>,
        _app: Arc<App>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // TODO: Implement HTTP/3 with QUIC transport
        // Plan:
        // 1. HTTP/3 requires QUIC transport instead of TCP
        // 2. Need to integrate quinn or similar QUIC library
        // 3. Use h3 crate for HTTP/3 implementation
        // 4. Bridge QUIC streams to Hotaru handlers
        
        unimplemented!("HTTP/3 requires QUIC transport, not TCP - implementation pending")
    }
}