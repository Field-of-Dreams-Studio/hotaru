//! gRPC Service integration for Hotaru
//!
//! This module provides the bridge between tonic services and Hotaru's endpoint system.

use std::sync::Arc;
use tonic::{Status, Code};
use h2per::HyperContext;

use hotaru_core::app::application::App;
use crate::context::GrpcContext;

/// gRPC service wrapper that integrates with Hotaru's service system
pub struct GrpcService {
    /// Service name (e.g., "helloworld.Greeter")
    pub name: String,
}

impl GrpcService {
    /// Creates a new gRPC service wrapper
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
        }
    }
    
    /// Handles incoming gRPC requests by converting them to GrpcContext
    pub async fn handle_request(
        &self,
        hyper_context: HyperContext,
        _app: Arc<App>,
    ) -> Result<GrpcContext, Status> {
        // Convert HyperContext to GrpcContext
        let grpc_context = GrpcContext::from_hyper_context(hyper_context)?;
        
        // Verify this is for our service
        if grpc_context.service != self.name {
            return Err(Status::new(
                Code::NotFound, 
                format!("Service {} not found", grpc_context.service)
            ));
        }
        
        Ok(grpc_context)
    }
    
    /// Creates a gRPC error response  
    /// Note: This is a placeholder - would need proper HyperContext initialization
    pub fn error_response(status: Status) -> Result<GrpcContext, Status> {
        // For now, return an error since we can't create HyperContext without a request
        Err(Status::new(status.code(), "Cannot create error response without request context"))
    }
}

// Service trait implementation removed - will be handled by endpoint! macro integration