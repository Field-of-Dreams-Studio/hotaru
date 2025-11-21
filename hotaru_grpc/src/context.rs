//! gRPC Context implementation
//!
//! Provides GrpcContext that wraps tonic functionality for use with Hotaru endpoints

use bytes::Bytes;
use http::HeaderMap;
use tonic::{Status, Code, metadata::MetadataMap};
use prost::Message;

use hotaru_core::connection::{RequestContext, ProtocolRole};
use h2per::HyperContext;

/// gRPC-specific context for use with Hotaru endpoints
pub struct GrpcContext {
    /// Underlying HTTP/2 context from h2per
    pub inner: HyperContext,
    
    /// gRPC method name (e.g., "SayHello")  
    pub method: String,
    
    /// gRPC service name (e.g., "helloworld.Greeter")
    pub service: String,
    
    /// gRPC metadata (headers)
    pub metadata: MetadataMap,
    
    /// gRPC status (for responses)
    pub status: Status,
    
    /// Request body bytes (protobuf message)
    request_body: Option<Bytes>,
    
    /// Response body bytes (protobuf message)  
    response_body: Option<Bytes>,
}

impl GrpcContext {
    /// Creates a new gRPC context from a Hyper context
    pub fn from_hyper_context(inner: HyperContext) -> Result<Self, Status> {
        // Extract gRPC information from the HTTP request
        let path = inner.request().uri().path();
        let (service, method) = Self::parse_grpc_path(path)?;
        
        // Check content-type
        let content_type = inner.request()
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
            
        if !content_type.starts_with("application/grpc") {
            return Err(Status::new(Code::InvalidArgument, "Not a gRPC request"));
        }
        
        // Extract metadata from headers
        let metadata = Self::extract_metadata(inner.request().headers());
        
        // Get request body from HyperRequest
        let request_body = inner.request().body_bytes.as_ref()
            .map(|bytes| Bytes::from(bytes.clone()));
        
        Ok(Self {
            inner,
            method,
            service,
            metadata,
            status: Status::ok(""),
            request_body,
            response_body: None,
        })
    }
    
    /// Parses gRPC path into service and method
    /// Path format: "/package.Service/Method"
    fn parse_grpc_path(path: &str) -> Result<(String, String), Status> {
        let path = path.trim_start_matches('/');
        let parts: Vec<&str> = path.split('/').collect();
        
        if parts.len() != 2 {
            return Err(Status::new(
                Code::InvalidArgument, 
                "Invalid gRPC path format"
            ));
        }
        
        Ok((parts[0].to_string(), parts[1].to_string()))
    }
    
    /// Extracts gRPC metadata from HTTP headers
    fn extract_metadata(headers: &HeaderMap) -> MetadataMap {
        let mut metadata = MetadataMap::new();
        
        for (name, value) in headers {
            // gRPC metadata headers don't include certain HTTP headers
            let name_str = name.as_str();
            if !name_str.starts_with(':') && 
               name_str != "content-type" &&
               name_str != "user-agent" {
                if let Ok(value_str) = value.to_str() {
                    if let Ok(metadata_value) = value_str.parse() {
                        // Convert to a static str by leaking the string 
                        // This is acceptable for gRPC metadata which has limited lifetime
                        let key: &'static str = Box::leak(name.as_str().to_string().into_boxed_str());
                        metadata.insert(key, metadata_value);
                    }
                }
            }
        }
        
        metadata
    }
    
    /// Decodes the request body as a protobuf message
    pub fn decode_request<T>(&self) -> Result<T, Status> 
    where
        T: Message + Default,
    {
        let body_bytes = self.request_body
            .as_ref()
            .ok_or_else(|| Status::new(Code::InvalidArgument, "No request body"))?;
        
        // gRPC framing: skip the first 5 bytes (1 byte compression flag + 4 bytes length)
        let message_bytes = if body_bytes.len() >= 5 {
            &body_bytes[5..]
        } else {
            return Err(Status::new(Code::InvalidArgument, "Invalid gRPC frame"));
        };
        
        T::decode(message_bytes)
            .map_err(|e| Status::new(Code::InvalidArgument, format!("Decode error: {}", e)))
    }
    
    /// Encodes a response message as protobuf and sets it in the context
    pub fn encode_response<T>(&mut self, message: T) -> Result<(), Status>
    where
        T: Message,
    {
        let mut buf = Vec::new();
        message.encode(&mut buf)
            .map_err(|e| Status::new(Code::Internal, format!("Encode error: {}", e)))?;
        
        // Add gRPC framing: 1 byte compression flag (0) + 4 bytes length + message
        let mut framed = Vec::with_capacity(5 + buf.len());
        framed.push(0); // No compression
        framed.extend_from_slice(&(buf.len() as u32).to_be_bytes());
        framed.extend_from_slice(&buf);
        
        self.response_body = Some(Bytes::from(framed));
        Ok(())
    }
    
    /// Sets the gRPC status
    pub fn set_status(&mut self, status: Status) {
        self.status = status;
    }
    
    /// Sets the gRPC status with code and message
    pub fn set_status_code(&mut self, code: Code, message: &str) {
        self.status = Status::new(code, message);
    }
    
    /// Finalizes the response and updates the inner HTTP context
    pub fn finalize_response(&mut self) {
        // Set response body if we have one
        if let Some(_body) = &self.response_body {
            // Update the inner context with the gRPC response
            // This would need to interact with h2per's response system
            // For now, we'll need to implement the bridge
        }
        
        // Set gRPC status in trailers
        let _status_code = self.status.code() as u32;
        let _status_message = self.status.message();
        
        // Add gRPC trailers (these go at the end of the HTTP/2 stream)
        // h2per will need to support trailers for this to work properly
    }
    
    /// Gets the underlying HyperContext (for compatibility)
    pub fn inner(&self) -> &HyperContext {
        &self.inner
    }
    
    /// Gets the underlying HyperContext mutably
    pub fn inner_mut(&mut self) -> &mut HyperContext {
        &mut self.inner
    }
}

/// gRPC request/response types for RequestContext
pub struct GrpcRequest {
    pub service: String,
    pub method: String,
    pub body: Option<Bytes>,
}

pub struct GrpcResponse {
    pub status: Status,
    pub body: Option<Bytes>,
}

impl RequestContext for GrpcContext {
    type Request = GrpcRequest;
    type Response = GrpcResponse;
    
    fn handle_error(&mut self) {
        // Set a gRPC error status
        self.status = Status::new(Code::Internal, "Internal server error");
        self.finalize_response();
    }
    
    fn role(&self) -> ProtocolRole {
        // gRPC contexts are always server-side for now
        ProtocolRole::Server
    }
}