# Hotaru gRPC Implementation Plan

## Overview

This document outlines the architecture and implementation plan for adding gRPC support to the Hotaru framework through the `hotaru_grpc` crate.

## Design Principles

### 1. Separation of Concerns
- **h2per**: Provides HTTP/2 transport layer via Hyper
- **hotaru_grpc**: Adds gRPC application layer on top of h2per
- **hotaru_core**: Remains protocol-agnostic

### 2. Consistent API
- Use existing `endpoint!` macro - no new macro needed
- gRPC endpoints work exactly like HTTP endpoints
- Same middleware system applies

### 3. Optional Integration
- Users can use Hotaru without gRPC (no bloat)
- gRPC support is additive, not intrusive
- Compatible with existing HTTP endpoints

## Architecture

### Component Structure

```
hotaru_grpc/
├── src/
│   ├── lib.rs              # Public API and re-exports
│   ├── protocol.rs         # GrpcProtocol implementation
│   ├── message.rs          # gRPC message framing
│   ├── status.rs           # gRPC status codes
│   ├── metadata.rs         # gRPC metadata handling
│   ├── streaming.rs        # Streaming support
│   ├── codec.rs            # Protobuf encoding/decoding
│   └── context.rs          # GrpcContext for endpoints
├── examples/
│   ├── greeter.rs          # Basic unary gRPC service
│   ├── streaming.rs        # Streaming examples
│   └── multi_service.rs    # Multiple services in one app
├── tests/
│   └── integration.rs      # Integration tests
└── proto/
    └── test.proto          # Test protobuf definitions
```

### Protocol Integration Flow

```
Client Request (gRPC)
    ↓
HTTP/2 (h2per) - Handles transport, multiplexing
    ↓
GrpcProtocol - Detects content-type, handles framing
    ↓
endpoint! - Standard Hotaru routing (no new macro!)
    ↓
User Handler - Business logic using GrpcContext
    ↓
GrpcResponse - Frames response with trailers
    ↓
HTTP/2 (h2per) - Sends response
```

## Implementation Plan

### Phase 1: Core Infrastructure

#### 1.1 Protocol Implementation
```rust
pub struct GrpcProtocol {
    http2: HyperHttp2,
}

impl Protocol for GrpcProtocol {
    type Context = GrpcContext;
    type Transport = Http2Transport;
    type Stream = Http2Stream;
    type Message = GrpcMessage;
    
    fn detect(initial_bytes: &[u8]) -> bool {
        // First check HTTP/2, then verify gRPC content-type
    }
}
```

#### 1.2 gRPC Context
```rust
pub struct GrpcContext {
    // Built on HyperContext
    inner: HyperContext,
    // gRPC-specific fields
    method_name: String,
    service_name: String,
    metadata: Metadata,
    status: GrpcStatus,
}

impl GrpcContext {
    // Methods for gRPC-specific operations
    pub fn decode_request<T: prost::Message + Default>(&self) -> Result<T, GrpcStatus>;
    pub fn encode_response<T: prost::Message>(&mut self, response: T);
    pub fn set_status(&mut self, status: GrpcStatus);
}
```

#### 1.3 Message Framing
```rust
pub struct GrpcMessage {
    pub compressed: bool,
    pub length: u32,
    pub data: Vec<u8>,  // Protobuf payload
}
```

### Phase 2: Endpoint Integration

#### 2.1 Using endpoint! for gRPC Services
```rust
use hotaru::prelude::*;
use hotaru_grpc::prelude::*;

// Multi-protocol app
pub static APP: SApp = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:50051")
        .handle(
            HandlerBuilder::new()
                .protocol(ProtocolBuilder::new(HTTP::server(HttpSafety::default())))
                .protocol(ProtocolBuilder::new(GrpcProtocol::new()))
        )
        .build()
});

// Regular HTTP endpoint
endpoint! {
    APP.url("/health"),
    
    pub health <HTTP> {
        json_response(object!({ status: "ok" }))
    }
}

// gRPC unary endpoint - uses same endpoint! macro!
endpoint! {
    APP.url("/helloworld.Greeter/SayHello"),
    
    /// gRPC SayHello method
    pub say_hello <GrpcProtocol> {
        // req is GrpcContext, not HyperContext
        let request: HelloRequest = req.decode_request()?;
        
        let response = HelloReply {
            message: format!("Hello, {}!", request.name),
        };
        
        req.encode_response(response);
        req.set_status(GrpcStatus::Ok);
        
        // Return the context (like HTTP endpoints)
        req
    }
}

// gRPC server streaming endpoint
endpoint! {
    APP.url("/helloworld.Greeter/SayHelloStream"),
    
    /// gRPC streaming method
    pub say_hello_stream <GrpcProtocol> {
        let request: HelloRequest = req.decode_request()?;
        
        // Stream responses
        for i in 0..5 {
            let response = HelloReply {
                message: format!("Hello #{}, {}!", i, request.name),
            };
            req.stream_response(response).await;
        }
        
        req.set_status(GrpcStatus::Ok);
        req
    }
}
```

#### 2.2 Service Organization
```rust
// Organize endpoints by service
mod greeter {
    use super::*;
    
    endpoint! {
        APP.url("/helloworld.Greeter/SayHello"),
        pub say_hello <GrpcProtocol> { /* ... */ }
    }
    
    endpoint! {
        APP.url("/helloworld.Greeter/SayHelloStream"),
        pub say_hello_stream <GrpcProtocol> { /* ... */ }
    }
}

mod calculator {
    use super::*;
    
    endpoint! {
        APP.url("/calculator.Calculator/Add"),
        pub add <GrpcProtocol> { /* ... */ }
    }
}
```

### Phase 3: Advanced Features

#### 3.1 Middleware Support
```rust
middleware! {
    /// gRPC authentication middleware
    pub GrpcAuth <GrpcProtocol> {
        let auth_header = req.metadata.get("authorization");
        if !validate_auth(auth_header) {
            req.set_status(GrpcStatus::Unauthenticated);
            return req;
        }
        next(req).await
    }
}

endpoint! {
    APP.url("/secure.Service/Method"),
    middleware = [GrpcAuth, LogRequest],
    
    pub secure_method <GrpcProtocol> {
        // This endpoint requires authentication
        // Standard middleware works with gRPC!
    }
}
```

#### 3.2 Streaming Implementation
```rust
impl GrpcContext {
    // For server streaming
    pub async fn stream_response<T: prost::Message>(&mut self, message: T);
    
    // For client streaming
    pub async fn next_request<T: prost::Message + Default>(&mut self) -> Option<T>;
    
    // For bidirectional streaming
    pub fn into_stream<Req, Res>(self) -> GrpcStream<Req, Res>
    where 
        Req: prost::Message + Default,
        Res: prost::Message;
}
```

#### 3.3 URL Routing
gRPC uses the URL pattern: `/{service}/{method}`

```rust
// Service: helloworld.Greeter
// Method: SayHello
// URL: /helloworld.Greeter/SayHello

endpoint! {
    APP.url("/helloworld.Greeter/SayHello"),
    pub say_hello <GrpcProtocol> { /* ... */ }
}

// Can also use pattern matching
endpoint! {
    APP.url("/<service>/<method>"),
    pub generic_grpc <GrpcProtocol> {
        let service = req.pattern("service").unwrap();
        let method = req.pattern("method").unwrap();
        
        match (service.as_str(), method.as_str()) {
            ("helloworld.Greeter", "SayHello") => { /* ... */ }
            ("calculator.Calculator", "Add") => { /* ... */ }
            _ => {
                req.set_status(GrpcStatus::Unimplemented);
                req
            }
        }
    }
}
```

## Key Advantages of Using endpoint!

### 1. No New Learning Curve
Developers already know `endpoint!` - no new macro to learn

### 2. Consistent Middleware
All existing middleware works with gRPC endpoints

### 3. Unified Routing
Same URL pattern matching system

### 4. Familiar Patterns
gRPC endpoints follow same patterns as HTTP endpoints

### 5. Multi-Protocol Apps
Easy to mix HTTP and gRPC in the same application

## Implementation Phases

### Phase 1: Foundation (Week 1-2)
- [ ] GrpcProtocol implementation
- [ ] GrpcContext with decode/encode methods
- [ ] Basic message framing
- [ ] Simple unary endpoint example

### Phase 2: Core Features (Week 3-4)
- [ ] Status codes and error handling
- [ ] Metadata support
- [ ] URL routing patterns
- [ ] Basic streaming support

### Phase 3: Advanced Features (Week 5-6)
- [ ] Full streaming implementation
- [ ] Middleware integration
- [ ] Compression support
- [ ] Performance optimization

### Phase 4: Production Ready (Week 7-8)
- [ ] Comprehensive testing
- [ ] Documentation and examples
- [ ] Performance benchmarks
- [ ] Error handling improvements

## Example Application Structure

```rust
use hotaru::prelude::*;
use hotaru_grpc::prelude::*;

pub static APP: SApp = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:50051")
        .handle(
            HandlerBuilder::new()
                .protocol(ProtocolBuilder::new(GrpcProtocol::new()))
        )
        .build()
});

// Greeter service
endpoint! {
    APP.url("/helloworld.Greeter/SayHello"),
    pub greeter_say_hello <GrpcProtocol> {
        let request: HelloRequest = req.decode_request()?;
        let response = HelloReply {
            message: format!("Hello, {}!", request.name),
        };
        req.encode_response(response);
        req
    }
}

// Calculator service  
endpoint! {
    APP.url("/calculator.Calculator/Add"),
    pub calculator_add <GrpcProtocol> {
        let request: AddRequest = req.decode_request()?;
        let response = AddResponse {
            result: request.a + request.b,
        };
        req.encode_response(response);
        req
    }
}

#[tokio::main]
async fn main() {
    println!("gRPC server running on :50051");
    APP.clone().run().await;
}
```

## Benefits of This Approach

1. **Simplicity**: No new macros, uses existing `endpoint!`
2. **Consistency**: Same patterns for HTTP and gRPC
3. **Flexibility**: Easy to mix protocols in one app
4. **Familiarity**: Developers already know the patterns
5. **Maintainability**: Less code, fewer abstractions
6. **Performance**: Direct integration with Hotaru's routing

This approach leverages Hotaru's existing strengths while adding gRPC support in the most natural way possible.