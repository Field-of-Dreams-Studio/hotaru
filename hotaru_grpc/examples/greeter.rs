//! gRPC Greeter example using Hotaru
//!
//! This example demonstrates how to create a gRPC service using Hotaru's endpoint! macro
//! with the GrpcProtocol implementation.

use std::sync::Arc;
use once_cell::sync::Lazy;

use hotaru::prelude::*;
use hotaru_grpc::prelude::*;

// Generated protobuf types
include!(concat!(env!("OUT_DIR"), "/helloworld.rs"));

/// Multi-protocol application with gRPC support
pub static APP: Lazy<Arc<App>> = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:50051")
        .handle(
            HandlerBuilder::new()
                .protocol(ProtocolBuilder::new(GrpcProtocol::new(ProtocolRole::Server)))
        )
        .build()
});

// gRPC endpoint using the familiar endpoint! macro
endpoint! {
    APP.url("/helloworld.Greeter/SayHello"),
    
    pub say_hello<GrpcProtocol> {
        // Decode the incoming protobuf request
        let request: HelloRequest = req.decode_request()
            .map_err(|e| {
                req.set_status_code(GrpcCode::InvalidArgument, &format!("Decode error: {}", e));
                e
            })?;
        
        println!("Received gRPC request: name = {}", request.name);
        
        // Create response
        let response = HelloReply {
            message: format!("Hello, {}!", request.name),
        };
        
        // Encode the response
        req.encode_response(response)
            .map_err(|e| {
                req.set_status_code(GrpcCode::Internal, &format!("Encode error: {}", e));
                e
            })?;
        
        // Set success status
        req.set_status_code(GrpcCode::Ok, "");
        
        req
    }
}

// Server streaming gRPC endpoint
endpoint! {
    APP.url("/helloworld.Greeter/SayHelloStream"),
    
    pub say_hello_stream<GrpcProtocol> {
        // Decode the request
        let request: HelloRequest = req.decode_request()
            .map_err(|e| {
                req.set_status_code(GrpcCode::InvalidArgument, &format!("Decode error: {}", e));
                e
            })?;
        
        println!("Received streaming gRPC request: name = {}", request.name);
        
        // For streaming, we would need to implement additional streaming support
        // For now, just return a single response
        let response = HelloReply {
            message: format!("Hello stream, {}!", request.name),
        };
        
        req.encode_response(response)
            .map_err(|e| {
                req.set_status_code(GrpcCode::Internal, &format!("Encode error: {}", e));
                e
            })?;
        
        req.set_status_code(GrpcCode::Ok, "");
        
        req
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Hotaru gRPC server on 127.0.0.1:50051");
    
    // Initialize the app
    Lazy::force(&APP);
    
    // Start the server
    APP.start().await?;
    
    Ok(())
}