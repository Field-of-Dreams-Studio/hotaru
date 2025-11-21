//! Stream implementations for HTTP/2 and HTTP/3.
//! HTTP/1.1 doesn't use streams, so it uses the unit type.

use std::any::Any;
use hotaru_core::connection::Stream;

// ============================================================================
// Base Stream type (used by HTTP/1.1 which doesn't have streams)
// ============================================================================

pub struct HyperStream;

// ============================================================================
// HTTP/2 Stream Implementation
// ============================================================================

#[derive(Clone)]
pub struct Http2Stream {
    id: u32,
    priority: Http2Priority,
    dependency: Option<u32>,
    weight: u8,
}

#[derive(Clone, Debug)]
pub struct Http2Priority {
    pub exclusive: bool,
    pub stream_dependency: u32,
    pub weight: u8,
}

impl Http2Stream {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            priority: Http2Priority {
                exclusive: false,
                stream_dependency: 0,
                weight: 16,
            },
            dependency: None,
            weight: 16,
        }
    }
    
    pub fn with_priority(id: u32, priority: Http2Priority) -> Self {
        Self {
            id,
            priority: priority.clone(),
            dependency: Some(priority.stream_dependency),
            weight: priority.weight,
        }
    }
}

impl Stream for Http2Stream {
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

// ============================================================================
// HTTP/3 Stream Implementation
// ============================================================================

#[derive(Clone)]
pub struct Http3Stream {
    id: u64,  // HTTP/3 uses u64 for stream IDs
    stream_type: Http3StreamType,
}

#[derive(Clone, Debug)]
pub enum Http3StreamType {
    /// Bidirectional stream for requests/responses
    Request,
    /// Unidirectional stream for server push
    Push(u64),
    /// Control stream
    Control,
    /// QPACK encoder stream
    QpackEncoder,
    /// QPACK decoder stream  
    QpackDecoder,
    /// Unknown/extension stream type
    Unknown(u64),
}

impl Http3Stream {
    pub fn new_request(id: u64) -> Self {
        Self {
            id,
            stream_type: Http3StreamType::Request,
        }
    }
    
    pub fn new_push(id: u64, push_id: u64) -> Self {
        Self {
            id,
            stream_type: Http3StreamType::Push(push_id),
        }
    }
    
    pub fn new_control(id: u64) -> Self {
        Self {
            id,
            stream_type: Http3StreamType::Control,
        }
    }
}

impl Stream for Http3Stream {
    fn id(&self) -> u32 {
        // Truncate u64 to u32 for compatibility with trait
        // In practice, stream IDs rarely exceed u32 range
        self.id as u32
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// Note: HTTP/3 actually uses u64 for stream IDs due to QUIC
impl Http3Stream {
    /// Get the full 64-bit stream ID
    pub fn id_u64(&self) -> u64 {
        self.id
    }
}