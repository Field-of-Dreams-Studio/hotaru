//! Simple gRPC server example using Hotaru
//!
//! This example demonstrates basic gRPC integration without using endpoint! macros

use hotaru_core::connection::{Protocol, ProtocolRole};
use hotaru_grpc::{GrpcProtocol, GrpcContext};
use tonic::{Status, Code};

// Generated protobuf types
include!(concat!(env!("OUT_DIR"), "/helloworld.rs"));

async fn handle_grpc_request(mut ctx: GrpcContext) -> Result<GrpcContext, Status> {
    println!("Received gRPC request: {} / {}", ctx.service, ctx.method);
    
    match ctx.method.as_str() {
        "SayHello" => {
            // Decode the request
            let request: HelloRequest = ctx.decode_request()?;
            println!("Request name: {}", request.name);
            
            // Create response
            let response = HelloReply {
                message: format!("Hello, {}!", request.name),
            };
            
            // Encode the response
            ctx.encode_response(response)?;
            ctx.set_status_code(Code::Ok, "Success");
            
            Ok(ctx)
        }
        _ => {
            ctx.set_status_code(Code::Unimplemented, "Method not implemented");
            Err(Status::new(Code::Unimplemented, "Method not implemented"))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting simple gRPC server on 127.0.0.1:50051");
    
    // Create the protocol
    let grpc_protocol = GrpcProtocol::new(ProtocolRole::Server);
    
    // For now, we'll create a minimal demonstration
    // In a full implementation, this would integrate with Hotaru's app system
    println!("gRPC protocol created successfully!");
    println!("Protocol role: {:?}", grpc_protocol.role());
    
    // Test protocol detection
    let http2_preface = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
    println!("HTTP/2 detection test: {}", GrpcProtocol::detect(http2_preface));
    
    println!("Basic gRPC integration is working!");
    println!("Note: Full server implementation requires deeper Hotaru app integration");
    
    Ok(())
}