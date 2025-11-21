//! gRPC Transport layer implementation
//!
//! This module provides gRPC-specific transport types that wrap h2per's HTTP/2 transport.

use std::error::Error;
use std::any::Any;
use bytes::{Bytes, BytesMut};

use hotaru_core::connection::{Transport, Stream, Message};
use h2per::transport::Http2Transport;
use h2per::stream::Http2Stream;

/// gRPC message wrapper
#[derive(Debug, Clone)]
pub struct GrpcMessage {
    /// The protobuf message body
    pub body: Option<Bytes>,
    
    /// gRPC specific metadata
    pub grpc_status: Option<u32>,
    pub grpc_message: Option<String>,
}

impl GrpcMessage {
    /// Creates a new gRPC message with body
    pub fn new(body: Bytes) -> Self {
        Self {
            body: Some(body),
            grpc_status: None,
            grpc_message: None,
        }
    }
    
    /// Creates a gRPC error message
    pub fn error(code: u32, message: impl Into<String>) -> Self {
        Self {
            body: None,
            grpc_status: Some(code),
            grpc_message: Some(message.into()),
        }
    }
    
    /// Sets gRPC status
    pub fn with_status(mut self, code: u32, message: impl Into<String>) -> Self {
        self.grpc_status = Some(code);
        self.grpc_message = Some(message.into());
        self
    }
    
    /// Gets the protobuf message bytes
    pub fn body(&self) -> Option<&Bytes> {
        self.body.as_ref()
    }
    
    /// Sets the protobuf message bytes
    pub fn set_body(&mut self, body: Bytes) {
        self.body = Some(body);
    }
}

impl Message for GrpcMessage {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Encode gRPC message with framing
        if let Some(body) = self.body() {
            // gRPC framing: 1 byte compression flag + 4 bytes length + message
            buf.extend_from_slice(&[0]); // No compression
            buf.extend_from_slice(&(body.len() as u32).to_be_bytes());
            buf.extend_from_slice(body);
        }
        
        // Add gRPC trailers if we have status
        if let Some(_status) = self.grpc_status {
            // This would be encoded as HTTP/2 trailers
            // For now, just note that status is set
        }
        
        Ok(())
    }
    
    fn decode(buf: &mut BytesMut) -> Result<Option<Self>, Box<dyn Error + Send + Sync>> 
    where
        Self: Sized,
    {
        // Check if we have enough bytes for gRPC framing
        if buf.len() < 5 {
            return Ok(None); // Need more data
        }
        
        // Parse gRPC frame header
        let _compression_flag = buf[0];
        let length = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]) as usize;
        
        // Check if we have the complete message
        if buf.len() < 5 + length {
            return Ok(None); // Need more data
        }
        
        // Extract the message bytes  
        let _ = buf.split_to(5); // Skip header
        let body = buf.split_to(length).freeze();
        
        Ok(Some(GrpcMessage::new(body)))
    }
}


/// gRPC transport that wraps HTTP/2 transport
pub struct GrpcTransport {
    inner: Http2Transport,
    id: i128,
}

impl GrpcTransport {
    /// Creates a new gRPC transport
    pub fn new(inner: Http2Transport) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as i128;
        
        Self { inner, id }
    }
}

impl Transport for GrpcTransport {
    fn id(&self) -> i128 {
        self.id
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// gRPC stream that wraps HTTP/2 stream
pub struct GrpcStream {
    inner: Http2Stream,
    id: u32,
}

impl GrpcStream {
    /// Creates a new gRPC stream
    pub fn new(inner: Http2Stream) -> Self {
        // Generate a stream ID (could be from HTTP/2 stream ID)
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(1);
        let id = COUNTER.fetch_add(2, Ordering::Relaxed); // Odd numbers for client-initiated
        
        Self { inner, id }
    }
}

impl Stream for GrpcStream {
    fn id(&self) -> u32 {
        self.id
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}