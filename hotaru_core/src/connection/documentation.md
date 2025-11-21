# Hotaru Connection Module - Trait-Based Protocol Architecture

## Philosophy

Hotaru provides a **trait-based** protocol system where protocols define their own abstractions. We don't impose any specific communication pattern, state management, or message structure. Protocols implement traits however makes sense for their design.

## The Four-Layer Trait System

```
┌─────────────────────────────────────────────────────────────┐
│                         Protocol                            │
│                   (User-defined handler)                    │
│         Implements the four traits below as needed          │
├─────────────────────────────────────────────────────────────┤
│                    RequestContext Trait                     │
│                  (Handler context type)                     │
│         Links request/response types for handlers           │
├─────────────────────────────────────────────────────────────┤
│                      Transport Trait                        │
│                  (Connection-level abstraction)             │
│         Protocol defines what "connection" means            │
├─────────────────────────────────────────────────────────────┤
│                       Stream Trait                          │
│                  (Optional stream abstraction)              │
│         Protocol defines what "stream" means (or none)      │
├─────────────────────────────────────────────────────────────┤
│                      Message Trait                          │
│                    (Wire format abstraction)                │
│         Protocol defines its message encoding               │
└─────────────────────────────────────────────────────────────┘
```

## Core Design Principles

1. **Traits, Not Structs**: We provide trait boundaries, not concrete implementations
2. **Protocol Freedom**: Each protocol decides its own architecture
3. **No Hidden Magic**: The framework doesn't make decisions for you
4. **Type Safety**: Compile-time guarantees through associated types
5. **Zero Overhead**: Traits compile away, no runtime cost

## The Trait Hierarchy

### RequestContext Trait

The RequestContext trait defines the type that flows through request handlers. This is what `AsyncFinalHandler<C>` and `AsyncMiddleware<C>` work with:

```rust
/// Context that flows through request handlers.
pub trait RequestContext: Send + 'static {
    /// The request type for this context
    type Request;
    
    /// The response type for this context  
    type Response;
    
    /// Handle protocol errors (bad request for server, bad response for client)
    fn handle_error(&mut self);
    
    /// Get the role of this context (Server or Client)
    fn role(&self) -> ProtocolRole;
}
```

Examples:
- **HTTP**: `HttpContext` with `HttpRequest` and `HttpResponse`
- **WebSocket**: `WsContext` with `WsMessage` for both directions
- **Game Protocol**: `GameContext` with `GameCommand` and `GameState`

This trait enables:
- Type-safe handler chains
- Bidirectional protocols (same context type for client/server)
- Protocol-specific error handling
- Integration with hotaru_meta macros

### Transport Trait

The Transport trait represents the protocol's concept of a connection. It could be:
- A simple TCP connection wrapper
- A connection with authentication state
- A multiplexed transport with flow control
- A stateful game session
- Anything the protocol needs

```rust
/// Protocol-defined connection abstraction.
/// 
/// This trait represents whatever "connection" means for your protocol.
/// The framework makes no assumptions about what you store here.
pub trait Transport: Send + Sync + 'static {
    /// Any identifier the protocol wants to use.
    /// Could be random, sequential, time-based, etc.
    fn id(&self) -> i128;
    
    /// Access to protocol-specific transport data.
    /// Protocols cast this to their concrete type.
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
```

### Stream Trait

The Stream trait is **optional**. Some protocols need streams, others don't:
- HTTP/2 has multiplexed request/response streams
- WebSocket might have one bidirectional stream
- Pub/Sub might use streams as topic subscriptions
- Simple request/response might not use streams at all

```rust
/// Protocol-defined stream abstraction (optional).
/// 
/// If your protocol doesn't have streams, use `type Stream = ()`.
/// If it does, implement this trait however makes sense.
pub trait Stream: Send + Sync + 'static {
    /// Stream identifier within the transport.
    /// The protocol defines the ID allocation scheme.
    fn id(&self) -> u32;
    
    /// Access to protocol-specific stream data.
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
```

### Message Trait

The Message trait defines the protocol's wire format. It could be:
- Text lines (HTTP/1.1)
- Binary frames (HTTP/2)
- Length-prefixed protobufs
- JSON messages
- Custom binary format
- Anything that goes over the wire

```rust
/// Protocol-defined message format.
/// 
/// This trait defines how your protocol encodes/decodes messages.
/// The framework doesn't care about the format.
pub trait Message: Send + Sync + 'static {
    /// Encode this message to bytes.
    /// The protocol defines the encoding format.
    fn encode(&self, buf: &mut BytesMut) -> Result<(), Box<dyn Error>>;
    
    /// Try to decode a message from bytes.
    /// Returns Ok(Some(message)) if a complete message was decoded.
    /// Returns Ok(None) if more bytes are needed.
    fn decode(buf: &mut BytesMut) -> Result<Option<Self>, Box<dyn Error>> 
    where 
        Self: Sized;
}
```

### Protocol Trait

The Protocol trait ties everything together:

```rust
/// User-defined protocol handler.
/// 
/// This is where protocols implement their logic using whatever
/// combination of Transport, Stream, and Message makes sense.
#[async_trait]
pub trait Protocol: Send + Sync + 'static {
    /// The protocol's connection abstraction.
    type Transport: Transport;
    
    /// The protocol's stream abstraction.
    /// Use () if the protocol doesn't have streams.
    type Stream: Stream;
    
    /// The protocol's message format.
    type Message: Message;
    
    /// Detect if this protocol can handle the connection.
    /// Peek at bytes, check headers, use magic numbers, etc.
    fn detect(initial_bytes: &[u8]) -> bool where Self: Sized;
    
    /// Handle a connection with this protocol.
    /// This is where all protocol logic lives.
    async fn handle(
        &mut self,
        tcp_stream: TcpConnectionStream,
        app: Arc<App>,
    ) -> Result<(), Box<dyn Error>>;
}
```

## Implementation Examples

### Example 1: Simple Request/Response Protocol

A basic protocol without streams or complex state:

```rust
/// Simple transport with just an ID
struct SimpleTransport {
    id: i128,
    request_count: u64,
}

impl Transport for SimpleTransport {
    fn id(&self) -> i128 { self.id }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

/// Simple text-based messages
struct SimpleMessage {
    command: String,
    payload: String,
}

impl Message for SimpleMessage {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), Box<dyn Error>> {
        // Simple line protocol: "COMMAND:payload\n"
        buf.extend_from_slice(self.command.as_bytes());
        buf.extend_from_slice(b":");
        buf.extend_from_slice(self.payload.as_bytes());
        buf.extend_from_slice(b"\n");
        Ok(())
    }
    
    fn decode(buf: &mut BytesMut) -> Result<Option<Self>, Box<dyn Error>> {
        // Look for newline
        if let Some(pos) = buf.iter().position(|&b| b == b'\n') {
            let line = buf.split_to(pos + 1);
            let line = std::str::from_utf8(&line[..pos])?;
            
            if let Some((cmd, payload)) = line.split_once(':') {
                return Ok(Some(SimpleMessage {
                    command: cmd.to_string(),
                    payload: payload.to_string(),
                }));
            }
        }
        Ok(None) // Need more data
    }
}

/// The protocol implementation
struct SimpleProtocol {
    handlers: HashMap<String, Box<dyn Fn(String) -> String>>,
}

#[async_trait]
impl Protocol for SimpleProtocol {
    type Transport = SimpleTransport;
    type Stream = (); // No streams
    type Message = SimpleMessage;
    
    fn detect(initial_bytes: &[u8]) -> bool {
        // Look for our protocol signature
        initial_bytes.starts_with(b"SIMPLE/1.0")
    }
    
    async fn handle(
        &mut self,
        mut tcp: TcpConnectionStream,
        _app: Arc<App>,
    ) -> Result<(), Box<dyn Error>> {
        let transport = SimpleTransport {
            id: generate_id(),
            request_count: 0,
        };
        
        let mut buffer = BytesMut::new();
        
        loop {
            // Read data
            tcp.read_buf(&mut buffer).await?;
            
            // Decode messages
            while let Some(msg) = SimpleMessage::decode(&mut buffer)? {
                // Handle message
                let response = self.handle_message(msg);
                
                // Encode and send response
                let mut response_buf = BytesMut::new();
                response.encode(&mut response_buf)?;
                tcp.write_all(&response_buf).await?;
            }
        }
    }
}
```

### Example 2: HTTP/1.1 Protocol

HTTP/1.1 with its sequential request/response pattern:

```rust
/// HTTP/1.1 transport with connection state
struct Http1Transport {
    id: i128,
    keep_alive: bool,
    request_count: u64,
}

impl Transport for Http1Transport {
    fn id(&self) -> i128 { self.id }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

/// HTTP/1.1 message (request or response)
enum Http1Message {
    Request(HttpRequest),
    Response(HttpResponse),
}

impl Message for Http1Message {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), Box<dyn Error>> {
        match self {
            Http1Message::Request(req) => {
                // Encode request line
                buf.extend_from_slice(req.method.as_bytes());
                buf.extend_from_slice(b" ");
                buf.extend_from_slice(req.path.as_bytes());
                buf.extend_from_slice(b" HTTP/1.1\r\n");
                
                // Encode headers
                for (name, value) in &req.headers {
                    buf.extend_from_slice(name.as_bytes());
                    buf.extend_from_slice(b": ");
                    buf.extend_from_slice(value.as_bytes());
                    buf.extend_from_slice(b"\r\n");
                }
                buf.extend_from_slice(b"\r\n");
                
                // Encode body
                buf.extend_from_slice(&req.body);
            }
            Http1Message::Response(res) => {
                // Similar encoding for response
            }
        }
        Ok(())
    }
    
    fn decode(buf: &mut BytesMut) -> Result<Option<Self>, Box<dyn Error>> {
        // Parse HTTP/1.1 message
        // Return None if incomplete
        // Return Some(message) if complete
    }
}

/// HTTP/1.1 protocol handler
struct HTTP {
    routes: Arc<Url<HTTP>>,
}

#[async_trait]
impl Protocol for HTTP {
    type Transport = Http1Transport;
    type Stream = (); // No streams in HTTP/1.1
    type Message = Http1Message;
    
    fn detect(initial_bytes: &[u8]) -> bool {
        // Check for HTTP methods
        initial_bytes.starts_with(b"GET ") ||
        initial_bytes.starts_with(b"POST ") ||
        initial_bytes.starts_with(b"PUT ") ||
        initial_bytes.starts_with(b"DELETE ")
    }
    
    async fn handle(
        &mut self,
        mut tcp: TcpConnectionStream,
        app: Arc<App>,
    ) -> Result<(), Box<dyn Error>> {
        let mut transport = Http1Transport {
            id: generate_id(),
            keep_alive: true,
            request_count: 0,
        };
        
        let mut buffer = BytesMut::new();
        
        loop {
            // Read and decode request
            tcp.read_buf(&mut buffer).await?;
            
            if let Some(Http1Message::Request(req)) = Http1Message::decode(&mut buffer)? {
                // Check keep-alive
                transport.keep_alive = req.headers.get("connection") != Some("close");
                transport.request_count += 1;
                
                // Route and handle
                let response = self.handle_request(req, &app).await?;
                
                // Encode and send response
                let mut response_buf = BytesMut::new();
                Http1Message::Response(response).encode(&mut response_buf)?;
                tcp.write_all(&response_buf).await?;
                
                // Check if we should close
                if !transport.keep_alive {
                    break;
                }
            }
        }
        
        Ok(())
    }
}
```

### Example 3: Multiplexed HTTP/2 Protocol

HTTP/2 with concurrent streams:

```rust
/// HTTP/2 transport with stream management
struct Http2Transport {
    id: i128,
    streams: HashMap<u32, Http2Stream>,
    next_stream_id: u32,
    settings: Http2Settings,
}

impl Transport for Http2Transport {
    fn id(&self) -> i128 { self.id }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

/// HTTP/2 stream state
struct Http2Stream {
    id: u32,
    state: StreamState,
    headers: Option<Headers>,
    data: Vec<u8>,
}

impl Stream for Http2Stream {
    fn id(&self) -> u32 { self.id }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

/// HTTP/2 frame
struct Http2Frame {
    stream_id: u32,
    frame_type: FrameType,
    flags: u8,
    payload: Vec<u8>,
}

impl Message for Http2Frame {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), Box<dyn Error>> {
        // 9-byte header
        buf.put_u24(self.payload.len() as u32);
        buf.put_u8(self.frame_type as u8);
        buf.put_u8(self.flags);
        buf.put_u32(self.stream_id);
        
        // Payload
        buf.extend_from_slice(&self.payload);
        Ok(())
    }
    
    fn decode(buf: &mut BytesMut) -> Result<Option<Self>, Box<dyn Error>> {
        if buf.len() < 9 {
            return Ok(None); // Need more data
        }
        
        // Parse header
        let length = u32::from_be_bytes([0, buf[0], buf[1], buf[2]]) as usize;
        let frame_type = FrameType::from(buf[3]);
        let flags = buf[4];
        let stream_id = u32::from_be_bytes([buf[5], buf[6], buf[7], buf[8]]) & 0x7FFFFFFF;
        
        if buf.len() < 9 + length {
            return Ok(None); // Need more data
        }
        
        // Extract payload
        buf.advance(9);
        let payload = buf.split_to(length).to_vec();
        
        Ok(Some(Http2Frame {
            stream_id,
            frame_type,
            flags,
            payload,
        }))
    }
}

#[async_trait]
impl Protocol for Http2Protocol {
    type Transport = Http2Transport;
    type Stream = Http2Stream;
    type Message = Http2Frame;
    
    fn detect(initial_bytes: &[u8]) -> bool {
        // HTTP/2 connection preface
        initial_bytes.starts_with(b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n")
    }
    
    async fn handle(
        &mut self,
        mut tcp: TcpConnectionStream,
        app: Arc<App>,
    ) -> Result<(), Box<dyn Error>> {
        let mut transport = Http2Transport {
            id: generate_id(),
            streams: HashMap::new(),
            next_stream_id: 2, // Server uses even IDs
            settings: Http2Settings::default(),
        };
        
        // Send settings
        self.send_settings(&mut tcp).await?;
        
        let mut buffer = BytesMut::new();
        
        loop {
            // Read data
            tcp.read_buf(&mut buffer).await?;
            
            // Process frames
            while let Some(frame) = Http2Frame::decode(&mut buffer)? {
                if frame.stream_id == 0 {
                    // Connection-level frame
                    self.handle_connection_frame(&mut transport, frame).await?;
                } else {
                    // Stream-level frame
                    self.handle_stream_frame(&mut transport, frame, &app).await?;
                }
            }
        }
    }
}
```

### Example 4: WebSocket Protocol

Bidirectional messaging over a single stream:

```rust
/// WebSocket transport
struct WsTransport {
    id: i128,
    is_client: bool,
    state: WsState,
}

/// WebSocket uses a single bidirectional stream
struct WsStream {
    incoming: VecDeque<WsMessage>,
    outgoing: VecDeque<WsMessage>,
}

impl Stream for WsStream {
    fn id(&self) -> u32 { 1 } // Always stream 1
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

/// WebSocket message
struct WsMessage {
    opcode: Opcode,
    payload: Vec<u8>,
    is_final: bool,
}

impl Message for WsMessage {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), Box<dyn Error>> {
        // WebSocket frame format
        let mut header = 0u8;
        
        // FIN bit
        if self.is_final {
            header |= 0x80;
        }
        
        // Opcode
        header |= self.opcode as u8;
        buf.put_u8(header);
        
        // Payload length
        let len = self.payload.len();
        if len < 126 {
            buf.put_u8(len as u8);
        } else if len < 65536 {
            buf.put_u8(126);
            buf.put_u16(len as u16);
        } else {
            buf.put_u8(127);
            buf.put_u64(len as u64);
        }
        
        // Payload
        buf.extend_from_slice(&self.payload);
        Ok(())
    }
    
    fn decode(buf: &mut BytesMut) -> Result<Option<Self>, Box<dyn Error>> {
        // Parse WebSocket frame
        // Handle fragmentation, masking, etc.
    }
}

#[async_trait]
impl Protocol for WebSocketProtocol {
    type Transport = WsTransport;
    type Stream = WsStream;
    type Message = WsMessage;
    
    fn detect(initial_bytes: &[u8]) -> bool {
        // Look for WebSocket upgrade
        // This would actually check HTTP headers
        initial_bytes.windows(9).any(|w| w == b"Upgrade: ")
    }
    
    async fn handle(
        &mut self,
        mut tcp: TcpConnectionStream,
        app: Arc<App>,
    ) -> Result<(), Box<dyn Error>> {
        // Perform WebSocket handshake
        self.handshake(&mut tcp).await?;
        
        let transport = WsTransport {
            id: generate_id(),
            is_client: false,
            state: WsState::Open,
        };
        
        let mut stream = WsStream {
            incoming: VecDeque::new(),
            outgoing: VecDeque::new(),
        };
        
        let mut buffer = BytesMut::new();
        
        loop {
            select! {
                // Read incoming messages
                _ = tcp.readable() => {
                    tcp.read_buf(&mut buffer).await?;
                    
                    while let Some(msg) = WsMessage::decode(&mut buffer)? {
                        match msg.opcode {
                            Opcode::Close => return Ok(()),
                            Opcode::Ping => {
                                // Send pong
                                let pong = WsMessage {
                                    opcode: Opcode::Pong,
                                    payload: msg.payload,
                                    is_final: true,
                                };
                                let mut pong_buf = BytesMut::new();
                                pong.encode(&mut pong_buf)?;
                                tcp.write_all(&pong_buf).await?;
                            }
                            _ => {
                                // Handle application message
                                stream.incoming.push_back(msg);
                            }
                        }
                    }
                }
                
                // Send outgoing messages
                msg = stream.outgoing.pop_front() => {
                    if let Some(msg) = msg {
                        let mut msg_buf = BytesMut::new();
                        msg.encode(&mut msg_buf)?;
                        tcp.write_all(&msg_buf).await?;
                    }
                }
            }
        }
    }
}
```

### Example 5: Custom Game Protocol

A completely custom protocol with its own concepts:

```rust
/// Game transport with player session
struct GameTransport {
    id: i128,
    player_id: PlayerId,
    session_token: String,
    world: Arc<GameWorld>,
}

/// Game channels as streams
enum GameStream {
    Movement(MovementChannel),
    Combat(CombatChannel),
    Chat(ChatChannel),
}

impl Stream for GameStream {
    fn id(&self) -> u32 {
        match self {
            GameStream::Movement(_) => 1,
            GameStream::Combat(_) => 2,
            GameStream::Chat(_) => 3,
        }
    }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

/// Game messages
enum GameMessage {
    Move { x: f32, y: f32 },
    Attack { target: EntityId },
    Chat { text: String },
}

impl Message for GameMessage {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), Box<dyn Error>> {
        // Custom binary protocol
        match self {
            GameMessage::Move { x, y } => {
                buf.put_u8(1); // Message type
                buf.put_f32(*x);
                buf.put_f32(*y);
            }
            GameMessage::Attack { target } => {
                buf.put_u8(2);
                buf.put_u32(*target);
            }
            GameMessage::Chat { text } => {
                buf.put_u8(3);
                buf.put_u16(text.len() as u16);
                buf.extend_from_slice(text.as_bytes());
            }
        }
        Ok(())
    }
    
    fn decode(buf: &mut BytesMut) -> Result<Option<Self>, Box<dyn Error>> {
        if buf.is_empty() {
            return Ok(None);
        }
        
        match buf[0] {
            1 => { // Move
                if buf.len() < 9 { return Ok(None); }
                let x = f32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);
                let y = f32::from_be_bytes([buf[5], buf[6], buf[7], buf[8]]);
                buf.advance(9);
                Ok(Some(GameMessage::Move { x, y }))
            }
            2 => { // Attack
                if buf.len() < 5 { return Ok(None); }
                let target = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);
                buf.advance(5);
                Ok(Some(GameMessage::Attack { target }))
            }
            3 => { // Chat
                if buf.len() < 3 { return Ok(None); }
                let len = u16::from_be_bytes([buf[1], buf[2]]) as usize;
                if buf.len() < 3 + len { return Ok(None); }
                let text = String::from_utf8(buf[3..3+len].to_vec())?;
                buf.advance(3 + len);
                Ok(Some(GameMessage::Chat { text }))
            }
            _ => Err("Unknown message type".into())
        }
    }
}

#[async_trait]
impl Protocol for GameProtocol {
    type Transport = GameTransport;
    type Stream = GameStream;
    type Message = GameMessage;
    
    fn detect(initial_bytes: &[u8]) -> bool {
        // Game magic bytes
        initial_bytes.starts_with(b"GAME")
    }
    
    async fn handle(
        &mut self,
        mut tcp: TcpConnectionStream,
        app: Arc<App>,
    ) -> Result<(), Box<dyn Error>> {
        // Authenticate
        let player = self.authenticate(&mut tcp).await?;
        
        let mut transport = GameTransport {
            id: generate_id(),
            player_id: player.id,
            session_token: generate_token(),
            world: self.world.clone(),
        };
        
        // Create channels
        let movement = GameStream::Movement(MovementChannel::new());
        let combat = GameStream::Combat(CombatChannel::new());
        let chat = GameStream::Chat(ChatChannel::new());
        
        let mut buffer = BytesMut::new();
        
        loop {
            tcp.read_buf(&mut buffer).await?;
            
            while let Some(msg) = GameMessage::decode(&mut buffer)? {
                match msg {
                    GameMessage::Move { x, y } => {
                        self.world.move_player(player.id, x, y).await?;
                    }
                    GameMessage::Attack { target } => {
                        self.world.attack(player.id, target).await?;
                    }
                    GameMessage::Chat { text } => {
                        self.world.broadcast_chat(player.id, text).await?;
                    }
                }
            }
        }
    }
}
```

## Module Structure

```
connection/
├── mod.rs                  # Module exports
├── architecture.md         # This documentation
│
├── transport/              # Transport trait and utilities
│   ├── mod.rs             
│   └── traits.rs          # Transport trait definition
│
├── stream/                 # Stream trait and utilities
│   ├── mod.rs             
│   └── traits.rs          # Stream trait definition
│
├── message/                # Message trait and utilities
│   ├── mod.rs             
│   └── traits.rs          # Message trait definition
│
├── protocol/               # Protocol trait and registry
│   ├── mod.rs             
│   ├── traits.rs          # Protocol trait definition
│   ├── registry.rs        # Protocol registry (enhanced)
│   └── adapter.rs         # Rx/Tx compatibility
│
├── legacy/                 # Backward compatibility
│   ├── receive.rs         # Rx trait (to be deprecated)
│   ├── transmit.rs        # Tx trait (to be deprecated)
│   └── connection.rs      # Old connection types
│
└── examples/               # Example protocol implementations
    ├── simple.rs          # Simple request/response
    ├── http1.rs           # HTTP/1.1
    ├── http2.rs           # HTTP/2
    ├── websocket.rs       # WebSocket
    └── custom.rs          # Custom protocols
```

## Key Benefits

### 1. True Protocol Independence

The framework provides boundaries, not implementations. Each protocol defines:
- What a "connection" means
- Whether it needs "streams" and what they are
- How messages are encoded/decoded
- Its entire communication pattern

### 2. Type Safety

Associated types ensure compile-time correctness:
```rust
// Compiler enforces that Transport implements the trait
type Transport = MyTransport;

// Can't accidentally mix protocols
let msg: Http2Frame = /* ... */;
websocket_protocol.handle_message(msg); // Compile error!
```

### 3. Zero Overhead

Traits compile to static dispatch:
```rust
// This has zero runtime cost
MyMessage::decode(&mut buf)

// vs dynamic dispatch (has cost)
trait_object.decode(&mut buf)
```

### 4. Flexibility

Protocols can:
- Define any state management approach
- Use any message encoding
- Implement any communication pattern
- Mix and match abstractions

### 5. Composability

Protocols can be composed and layered:
```rust
// TLS wrapper for any protocol
struct TlsWrapper<P: Protocol> {
    inner: P,
}

// Protocol upgrading (HTTP -> WebSocket)
struct UpgradableProtocol {
    http: HttpProtocol,
    websocket: WebSocketProtocol,
}
```

## Migration from Rx/Tx

The trait-based system coexists with Rx/Tx during migration:

1. **Phase 1**: Add trait definitions alongside Rx/Tx
2. **Phase 2**: Create adapters that implement traits using Rx/Tx
3. **Phase 3**: Gradually rewrite protocols to use traits directly
4. **Phase 4**: Remove Rx/Tx and adapters

Example adapter:
```rust
/// Adapter to use Rx protocols with new trait system
struct RxAdapter<R: Rx> {
    inner: R,
}

impl<R: Rx> Protocol for RxAdapter<R> {
    type Transport = LegacyTransport;
    type Stream = ();
    type Message = LegacyFrame;
    
    fn detect(bytes: &[u8]) -> bool {
        R::test_protocol(bytes)
    }
    
    async fn handle(&mut self, tcp: TcpConnectionStream, app: Arc<App>) {
        // Call old Rx methods
        self.inner.process(tcp, app).await
    }
}
```

## Summary

This trait-based architecture gives protocol implementers complete freedom while providing just enough structure for Hotaru to manage connections. The three traits (Transport, Stream, Message) are **boundaries, not implementations** - each protocol fills them in however makes sense.

The key insight: **We don't tell protocols how to work, we just ask them what they need.**