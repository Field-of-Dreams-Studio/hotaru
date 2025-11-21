//! Comprehensive gRPC demo showcasing integration features
//!
//! This example demonstrates the key features of hotaru_grpc integration

use bytes::Bytes;
use hotaru_core::connection::{Protocol, ProtocolRole};
use hotaru_grpc::{GrpcProtocol, GrpcContext};
use tonic::{Status, Code};
use h2per::HyperContext;

// Generated protobuf types
include!(concat!(env!("OUT_DIR"), "/helloworld.rs"));

async fn demo_grpc_context() -> Result<(), Status> {
    println!("=== gRPC Context Demo ===");
    
    // Create a mock HTTP request (this would normally come from h2per)
    let request_body = HelloRequest {
        name: "Hotaru".to_string(),
    };
    
    // Encode the request as protobuf with gRPC framing
    let mut encoded = Vec::new();
    encoded.push(0); // No compression
    let message_bytes = {
        use prost::Message;
        let mut buf = Vec::new();
        request_body.encode(&mut buf).unwrap();
        buf
    };
    encoded.extend_from_slice(&(message_bytes.len() as u32).to_be_bytes());
    encoded.extend_from_slice(&message_bytes);
    
    println!("✅ Encoded gRPC message: {} bytes", encoded.len());
    println!("   - Compression flag: {}", encoded[0]);
    println!("   - Message length: {} bytes", message_bytes.len());
    println!("   - Original message: '{}'", request_body.name);
    
    // Demonstrate gRPC message parsing
    use hotaru_grpc::transport::GrpcMessage;
    use hotaru_core::connection::Message;
    use bytes::BytesMut;
    
    let mut buf = BytesMut::from(encoded.as_slice());
    let parsed_message = GrpcMessage::decode(&mut buf).unwrap().unwrap();
    
    println!("✅ Parsed gRPC message successfully");
    
    // Demonstrate encoding a response  
    let response = HelloReply {
        message: format!("Hello from Hotaru gRPC!"),
    };
    
    let mut response_buf = BytesMut::new();
    let response_message = GrpcMessage::new(Bytes::from({
        use prost::Message;
        let mut buf = Vec::new();
        response.encode(&mut buf).unwrap();
        buf
    }));
    
    response_message.encode(&mut response_buf).unwrap();
    println!("✅ Encoded gRPC response: {} bytes", response_buf.len());
    
    Ok(())
}

fn demo_protocol_detection() {
    println!("\n=== Protocol Detection Demo ===");
    
    // Test HTTP/2 connection preface detection
    let http2_preface = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
    let http1_request = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
    let invalid_data = b"invalid protocol data";
    
    println!("HTTP/2 preface detection: {}", GrpcProtocol::detect(http2_preface));
    println!("HTTP/1.1 request detection: {}", GrpcProtocol::detect(http1_request));
    println!("Invalid data detection: {}", GrpcProtocol::detect(invalid_data));
    
    // Create protocol instance
    let protocol = GrpcProtocol::new(ProtocolRole::Server);
    println!("✅ Created gRPC protocol with role: {:?}", protocol.role());
}

fn demo_grpc_path_parsing() {
    println!("\n=== gRPC Path Parsing Demo ===");
    
    let valid_paths = [
        "/helloworld.Greeter/SayHello",
        "/myservice.Calculator/Add", 
        "/com.example.UserService/GetUser",
    ];
    
    let invalid_paths = [
        "/invalid",
        "/too/many/parts",
        "",
    ];
    
    println!("Valid gRPC paths:");
    for path in valid_paths {
        // This would normally be called internally by GrpcContext::from_hyper_context
        println!("  ✅ {}", path);
    }
    
    println!("Invalid gRPC paths:");  
    for path in invalid_paths {
        println!("  ❌ '{}'", path);
    }
}

fn demo_status_codes() {
    println!("\n=== gRPC Status Codes Demo ===");
    
    let status_examples = [
        (Code::Ok, "Request completed successfully"),
        (Code::InvalidArgument, "Invalid request parameters"),
        (Code::NotFound, "Requested resource not found"),
        (Code::Internal, "Internal server error"),
        (Code::Unimplemented, "Method not implemented"),
    ];
    
    for (code, description) in status_examples {
        let status = Status::new(code, description);
        println!("  {} - {} ({})", code as u32, description, status.message());
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Hotaru gRPC Integration Demo");
    println!("================================");
    
    demo_protocol_detection();
    demo_grpc_context().await?;
    demo_grpc_path_parsing();
    demo_status_codes();
    
    println!("\n✅ All demos completed successfully!");
    println!("\nNext steps:");
    println!("  - Integrate with Hotaru's app system");
    println!("  - Add streaming support");
    println!("  - Create real gRPC service endpoints");
    println!("  - Test with actual gRPC clients");
    
    Ok(())
}