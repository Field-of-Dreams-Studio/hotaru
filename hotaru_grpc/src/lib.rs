//! gRPC support for the Hotaru web framework
//!
//! This crate provides gRPC protocol support built on top of tonic and h2per's HTTP/2 implementation.
//! It enables Hotaru applications to serve gRPC services alongside regular HTTP endpoints using
//! the familiar `endpoint!` macro.
//!
//! # Example
//!
//! ```rust
//! use hotaru::prelude::*;
//! use hotaru_grpc::prelude::*;
//!
//! // Multi-protocol app with HTTP and gRPC
//! pub static APP: SApp = Lazy::new(|| {
//!     App::new()
//!         .binding("127.0.0.1:50051")
//!         .handle(
//!             HandlerBuilder::new()
//!                 .protocol(ProtocolBuilder::new(HTTP::server(HttpSafety::default())))
//!                 .protocol(ProtocolBuilder::new(GrpcProtocol::new()))
//!         )
//!         .build()
//! });
//!
//! // gRPC endpoint using familiar endpoint! macro
//! endpoint! {
//!     APP.url("/helloworld.Greeter/SayHello"),
//!     
//!     pub say_hello <GrpcProtocol> {
//!         let request: HelloRequest = req.decode_request()?;
//!         let response = HelloReply {
//!             message: format!("Hello, {}!", request.name),
//!         };
//!         req.encode_response(response);
//!         req
//!     }
//! }
//! ```

pub mod protocol;
pub mod context;
pub mod service;
pub mod transport;

// Re-export key types
pub use protocol::GrpcProtocol;
pub use context::GrpcContext;
pub use service::GrpcService;

// Re-export tonic types for convenience
pub use tonic::{Status as GrpcStatus, Code as GrpcCode, Request as TonicRequest, Response as TonicResponse};
pub use prost::Message;

// Re-export h2per types we build on
pub use h2per::{HyperHttp2, HyperContext};

pub mod prelude {
    //! Common imports for gRPC development
    
    pub use crate::{
        GrpcProtocol,
        GrpcContext, 
        GrpcService,
        GrpcStatus,
        GrpcCode,
        Message,
    };
    
    // Re-export hotaru core types
    pub use hotaru_core::connection::*;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{GrpcRequest, GrpcResponse};
    use crate::transport::GrpcMessage;
    use bytes::{Bytes, BytesMut};
    use hotaru_core::connection::{Protocol, ProtocolRole, RequestContext, Message as MessageTrait};
    use tonic::{Status, Code};

    #[test]
    fn test_grpc_protocol_detection() {
        // Test HTTP/2 preface detection
        let http2_preface = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
        assert!(GrpcProtocol::detect(http2_preface), "Should detect HTTP/2 preface");
        
        // Test non-HTTP/2 data
        let http1_request = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
        assert!(!GrpcProtocol::detect(http1_request), "Should not detect HTTP/1.1");
        
        let random_data = b"random data that is not HTTP/2";
        assert!(!GrpcProtocol::detect(random_data), "Should not detect random data");
    }

    #[test]
    fn test_grpc_protocol_role() {
        let server_protocol = GrpcProtocol::new(ProtocolRole::Server);
        assert_eq!(server_protocol.role(), ProtocolRole::Server);
        
        let client_protocol = GrpcProtocol::new(ProtocolRole::Client);
        assert_eq!(client_protocol.role(), ProtocolRole::Client);
    }

    #[test]
    fn test_grpc_message_encoding() {
        let body = Bytes::from(vec![1, 2, 3, 4, 5]);
        let message = GrpcMessage::new(body.clone());
        
        let mut buf = BytesMut::new();
        message.encode(&mut buf).unwrap();
        
        // Check gRPC framing: 1 byte compression + 4 bytes length + message
        assert_eq!(buf.len(), 1 + 4 + body.len());
        assert_eq!(buf[0], 0); // No compression
        
        let length = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);
        assert_eq!(length as usize, body.len());
    }

    #[test]
    fn test_grpc_message_decoding() {
        // Create a properly framed gRPC message
        let mut buf = BytesMut::new();
        buf.extend_from_slice(&[0]); // No compression
        buf.extend_from_slice(&5u32.to_be_bytes()); // Length = 5
        buf.extend_from_slice(&[1, 2, 3, 4, 5]); // Message body
        
        let decoded = GrpcMessage::decode(&mut buf).unwrap();
        assert!(decoded.is_some());
        
        let message = decoded.unwrap();
        assert_eq!(message.body().unwrap(), &Bytes::from(vec![1, 2, 3, 4, 5]));
    }

    #[test]
    fn test_grpc_message_incomplete_decoding() {
        // Test with incomplete frame header
        let mut buf = BytesMut::from(&[0, 0, 0][..]); // Only 3 bytes, need 5 for header
        let decoded = GrpcMessage::decode(&mut buf).unwrap();
        assert!(decoded.is_none(), "Should return None for incomplete header");
        
        // Test with incomplete message body
        let mut buf = BytesMut::new();
        buf.extend_from_slice(&[0]); // No compression
        buf.extend_from_slice(&10u32.to_be_bytes()); // Length = 10
        buf.extend_from_slice(&[1, 2, 3, 4, 5]); // Only 5 bytes, but expecting 10
        
        let decoded = GrpcMessage::decode(&mut buf).unwrap();
        assert!(decoded.is_none(), "Should return None for incomplete body");
    }

    #[test]
    fn test_grpc_message_with_status() {
        let mut message = GrpcMessage::error(3, "Invalid argument");
        assert_eq!(message.grpc_status, Some(3));
        assert_eq!(message.grpc_message, Some("Invalid argument".to_string()));
        
        message = message.with_status(5, "Not found");
        assert_eq!(message.grpc_status, Some(5));
        assert_eq!(message.grpc_message, Some("Not found".to_string()));
    }

    #[test]
    fn test_grpc_context_path_parsing() {
        // Test valid gRPC paths
        let valid_paths = [
            ("/helloworld.Greeter/SayHello", ("helloworld.Greeter", "SayHello")),
            ("/myservice.Calculator/Add", ("myservice.Calculator", "Add")),
            ("/com.example.UserService/GetUser", ("com.example.UserService", "GetUser")),
        ];
        
        for (path, (expected_service, expected_method)) in valid_paths {
            // This would normally be called internally by GrpcContext::from_hyper_context
            // We're testing the parsing logic here
            let path = path.trim_start_matches('/');
            let parts: Vec<&str> = path.split('/').collect();
            
            assert_eq!(parts.len(), 2);
            assert_eq!(parts[0], expected_service);
            assert_eq!(parts[1], expected_method);
        }
    }

    #[test]
    fn test_grpc_context_request_context_trait() {
        // Test that GrpcContext implements RequestContext correctly
        fn assert_request_context<T: RequestContext>() {}
        assert_request_context::<GrpcContext>();
        
        // Test the types are correct
        type _Request = <GrpcContext as RequestContext>::Request;
        type _Response = <GrpcContext as RequestContext>::Response;
        
        // These should compile without error
        let _: fn(_Request) = |_: GrpcRequest| {};
        let _: fn(_Response) = |_: GrpcResponse| {};
    }

    #[test]
    fn test_grpc_status_codes() {
        let statuses = [
            (Code::Ok, 0),
            (Code::Cancelled, 1),
            (Code::Unknown, 2),
            (Code::InvalidArgument, 3),
            (Code::DeadlineExceeded, 4),
            (Code::NotFound, 5),
            (Code::AlreadyExists, 6),
            (Code::PermissionDenied, 7),
            (Code::ResourceExhausted, 8),
            (Code::FailedPrecondition, 9),
            (Code::Aborted, 10),
            (Code::OutOfRange, 11),
            (Code::Unimplemented, 12),
            (Code::Internal, 13),
            (Code::Unavailable, 14),
            (Code::DataLoss, 15),
            (Code::Unauthenticated, 16),
        ];
        
        for (code, expected_value) in statuses {
            assert_eq!(code as u32, expected_value);
            
            let status = Status::new(code, "test message");
            assert_eq!(status.code(), code);
            assert_eq!(status.message(), "test message");
        }
    }

    #[test]
    fn test_transport_ids() {
        use crate::transport::{GrpcTransport, GrpcStream};
        use h2per::transport::Http2Transport;
        use h2per::stream::Http2Stream;
        use hotaru_core::connection::{Transport, Stream};
        
        // Test transport ID generation
        let http2_transport = Http2Transport::new();
        let transport = GrpcTransport::new(http2_transport);
        let id1 = transport.id();
        
        // Small delay to ensure different timestamp
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        let http2_transport2 = Http2Transport::new();
        let transport2 = GrpcTransport::new(http2_transport2);
        let id2 = transport2.id();
        
        // IDs should be different (based on timestamp)
        assert_ne!(id1, id2);
        
        // Test stream ID generation
        let http2_stream = Http2Stream::new(1);
        let stream1 = GrpcStream::new(http2_stream);
        
        let http2_stream2 = Http2Stream::new(3);
        let stream2 = GrpcStream::new(http2_stream2);
        
        // Stream IDs should be different and odd (client-initiated)
        assert_ne!(stream1.id(), stream2.id());
        assert_eq!(stream1.id() % 2, 1, "Stream ID should be odd");
        assert_eq!(stream2.id() % 2, 1, "Stream ID should be odd");
    }
}