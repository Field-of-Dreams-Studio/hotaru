# Protocol Upgrade/Downgrade Implementation Plan for Hyper-Hotaru

## Overview
Integrate Hyper's upgrade capabilities with Hotaru's existing protocol switching mechanism using `ConnectionStatus::SwitchProtocol(TypeId)`.

## Hotaru's Existing Architecture

### 1. ConnectionStatus Enum
```rust
pub enum ConnectionStatus {
    Established,     // Initial state
    Upgraded,        // After upgrade
    Connected,       // Active connection
    Stopped,         // Terminate connection
    SwitchProtocol(TypeId)  // Request protocol switch
}
```

### 2. Protocol Handler Interface
```rust
trait ProtocolHandlerTrait {
    fn handle_upgrade(
        &self,
        app: Arc<App>,
        reader: BufReader<ReadHalf<TcpConnectionStream>>,
        writer: BufWriter<WriteHalf<TcpConnectionStream>>,
        params: RwLock<Params>,
        locals: RwLock<Locals>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}
```

## Implementation Strategy

### Phase 1: Protocol Detection & Signaling

1. **Detect Upgrade Request in HyperHttp1**:
   - Check for `Upgrade` header in requests
   - Check for `Connection: Upgrade` header
   - Return `ConnectionStatus::SwitchProtocol(TypeId)` when upgrade is needed

2. **Protocol TypeId Mapping**:
   ```rust
   // In h2per/src/protocol.rs
   impl HyperHttp1 {
       fn check_upgrade(&self, req: &Request) -> Option<ConnectionStatus> {
           if let Some(upgrade) = req.headers().get(UPGRADE) {
               match upgrade.to_str().ok()? {
                   "h2c" => Some(ConnectionStatus::SwitchProtocol(
                       TypeId::of::<HyperHttp2>()
                   )),
                   "websocket" => Some(ConnectionStatus::SwitchProtocol(
                       TypeId::of::<WebSocketProtocol>()
                   )),
                   _ => None
               }
           }
       }
   }
   ```

### Phase 2: Upgrade Response Handling

1. **Send 101 Switching Protocols**:
   ```rust
   // Before returning SwitchProtocol status
   let response = Response::builder()
       .status(StatusCode::SWITCHING_PROTOCOLS)
       .header(CONNECTION, "Upgrade")
       .header(UPGRADE, "h2c")  // or "websocket"
       .body(Empty::new())
       .unwrap();
   
   // Send response, then return switch status
   ```

2. **Preserve Connection State**:
   - Save params and locals before switching
   - Pass to new protocol's `handle_upgrade()`

### Phase 3: Protocol-Specific Implementations

#### A. WebSocket Upgrade (HTTP/1.1 → WebSocket)
```rust
pub struct WebSocketProtocol;

impl Protocol for WebSocketProtocol {
    fn detect(initial_bytes: &[u8]) -> bool {
        // WebSocket frames detection
    }
    
    async fn handle(
        &mut self,
        reader: BufReader<ReadHalf<TcpConnectionStream>>,
        writer: BufWriter<WriteHalf<TcpConnectionStream>>,
        app: Arc<App>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Handle WebSocket frames
        // Use tokio-tungstenite or similar
    }
}
```

#### B. HTTP/2 Upgrade (h2c - HTTP/1.1 → HTTP/2)
```rust
impl HyperHttp2 {
    fn handle_h2c_upgrade(
        &mut self,
        reader: BufReader<ReadHalf<TcpConnectionStream>>,
        writer: BufWriter<WriteHalf<TcpConnectionStream>>,
        settings: &str,  // From HTTP2-Settings header
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Continue as HTTP/2 connection
        // Apply settings from upgrade request
    }
}
```

#### C. HTTP/3 Advertisement (Alt-Svc)
```rust
impl HyperHttp1 {
    fn add_alt_svc(&self, response: &mut Response) {
        // Add Alt-Svc header for HTTP/3
        response.headers_mut().insert(
            "Alt-Svc",
            "h3=\":443\"; ma=86400"
        );
    }
}
```

### Phase 4: Downgrade Support

1. **Fallback Mechanism**:
   ```rust
   impl HyperHttp2 {
       fn should_downgrade(&self, error: &Error) -> Option<ConnectionStatus> {
           // On certain errors, downgrade to HTTP/1.1
           Some(ConnectionStatus::SwitchProtocol(
               TypeId::of::<HyperHttp1>()
           ))
       }
   }
   ```

2. **Client Capability Detection**:
   - Check ALPN negotiation results
   - Detect client protocol support
   - Automatic fallback when needed

### Phase 5: Integration Points

1. **Modify HyperHttp1::handle()**:
   ```rust
   async fn handle(...) -> Result<(), Box<dyn Error + Send + Sync>> {
       // ... existing code ...
       
       // Check for upgrade request
       if let Some(status) = self.check_upgrade(&request) {
           // Send 101 response
           self.send_switching_protocols(&mut writer).await?;
           
           // Signal protocol switch
           return Ok(status);
       }
       
       // Continue normal HTTP/1.1 handling
   }
   ```

2. **Registry Updates**:
   - Register WebSocket protocol handler
   - Update protocol detection order
   - Handle upgrade paths in registry

## Benefits of This Approach

1. **Uses Existing Infrastructure**: Leverages Hotaru's `ConnectionStatus` and `handle_upgrade()`
2. **Clean Separation**: Each protocol handles its own upgrade logic
3. **Extensible**: Easy to add new protocols (HTTP/3, custom protocols)
4. **State Preservation**: Params and locals transfer between protocols
5. **Hyper Integration**: Uses Hyper's built-in upgrade support where available

## Implementation Order

1. **Basic WebSocket upgrade** (simplest case)
2. **HTTP/2 upgrade (h2c)** (more complex)
3. **HTTP/3 advertisement** (Alt-Svc header)
4. **Downgrade mechanisms** (error handling)
5. **Custom protocol support** (extensibility)

## Testing Strategy

1. **Unit Tests**: Test each upgrade path
2. **Integration Tests**: Full upgrade scenarios
3. **Performance Tests**: Measure upgrade overhead
4. **Compatibility Tests**: Various client behaviors

## Example Usage

```rust
endpoint! {
    APP.url("/ws"),
    
    pub websocket_endpoint <HYPER1> {
        // Check if client wants WebSocket
        if req.wants_upgrade() == Some("websocket") {
            // Validate WebSocket headers
            if validate_websocket_request(&req) {
                // Signal upgrade
                return ConnectionStatus::SwitchProtocol(
                    TypeId::of::<WebSocketProtocol>()
                );
            }
        }
        
        // Regular HTTP response if not upgrading
        text_response("WebSocket endpoint")
    }
}
```