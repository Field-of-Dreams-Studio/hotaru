//! gRPC Protocol implementation built on tonic and h2per
//!
//! This module provides the GrpcProtocol that integrates tonic's gRPC implementation
//! with Hotaru's protocol system.

use std::sync::Arc;
use std::error::Error;
use async_trait::async_trait;
use tokio::io::{BufReader, BufWriter, ReadHalf, WriteHalf};
use http::HeaderMap;

use hotaru_core::{
    app::application::App,
    connection::{Protocol, ProtocolRole, TcpConnectionStream},
};

use h2per::HyperHttp2;
use crate::context::GrpcContext;

/// gRPC protocol implementation that wraps tonic functionality
#[derive(Clone)]
pub struct GrpcProtocol {
    inner: HyperHttp2,
    role: ProtocolRole,
}

impl GrpcProtocol {
    /// Creates a new gRPC protocol instance
    pub fn new(role: ProtocolRole) -> Self {
        Self {
            inner: HyperHttp2::new(role),
            role,
        }
    }
    
    /// Checks if the request headers indicate gRPC
    fn is_grpc_request(headers: &HeaderMap) -> bool {
        headers
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|ct| ct.starts_with("application/grpc"))
            .unwrap_or(false)
    }
}

#[async_trait]
impl Protocol for GrpcProtocol {
    type Transport = ();
    type Stream = ();
    type Message = crate::transport::GrpcMessage;
    type Context = GrpcContext;
    
    fn detect(initial_bytes: &[u8]) -> bool {
        // First check if this looks like HTTP/2
        if !HyperHttp2::detect(initial_bytes) {
            return false;
        }
        
        // We'll do final gRPC detection based on headers in the service layer
        // since we need the full HTTP/2 request to check content-type
        true
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
        // Delegate to the underlying HTTP/2 implementation
        // The gRPC-specific handling happens in the service layer
        self.inner.handle(reader, writer, app).await
    }
}