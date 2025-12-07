# HTTP Module Architecture Documentation

## Overview

The HTTP module in Hotaru Core provides a complete HTTP/1.1 protocol implementation that integrates with the Protocol Abstraction Layer. It handles both server and client operations, supporting request parsing, response generation, and bidirectional communication.

## Core Components

### 1. Protocol Implementation (`traits.rs`)

#### HTTP
The main protocol handler that implements the `Protocol` trait:
- **Server mode**: Handles incoming HTTP requests and generates responses
- **Client mode**: Sends HTTP requests and processes responses
- **Protocol detection**: Identifies HTTP/1.1 traffic by checking for HTTP methods (GET, POST, etc.)

#### HttpTransport
Connection-level state management:
- Tracks keep-alive status
- Counts requests per connection
- Manages connection lifecycle
- Stores safety configuration (limits, timeouts)

#### HttpMessage
Message wrapper enum that implements the `Message` trait:
- `Request(HttpRequest)`: Client-to-server messages
- `Response(HttpResponse)`: Server-to-client messages
- Handles encoding/decoding for the Protocol trait

### 2. Request and Response Types

#### HttpRequest (`request.rs`)
```rust
pub struct HttpRequest {
    pub meta: HttpMeta,    // Headers and metadata
    pub body: HttpBody     // Request body
}
```
- Provides `send()` method for transmission
- Supports lazy parsing from streams
- Integrates with safety limits

#### HttpResponse (`response.rs`)
```rust
pub struct HttpResponse {
    pub meta: HttpMeta,    // Headers and metadata
    pub body: HttpBody     // Response body
}
```
- Mirrors request structure
- Provides `send()` method for transmission
- Supports streaming and chunked encoding

### 3. Metadata System (`meta.rs`)

#### HttpMeta
Central metadata container for both requests and responses:
```rust
pub struct HttpMeta {
    pub start_line: HttpStartLine,           // Request/Response line
    pub header: HashMap<String, HeaderValue>, // HTTP headers
    content_type: Option<HttpContentType>,    // Parsed content type
    content_length: Option<usize>,            // Content length
    cookies: Option<CookieMap>,               // Parsed cookies
    content_disposition: Option<ContentDisposition>, // File downloads
}
```

Key methods:
- `represent()`: Formats complete HTTP header section
- `from_stream()`: Parses headers from network stream
- Getters/setters for special headers (content-type, cookies, etc.)

#### HttpStartLine (`start_line.rs`)
Enum handling both request and response start lines:
- `Request(RequestStartLine)`: Method, path, HTTP version
- `Response(ResponseStartLine)`: HTTP version, status code

### 4. Body Handling (`body.rs`)

#### HttpBody
Flexible body representation supporting multiple formats:
```rust
pub enum HttpBody {
    Text(String),           // Plain text
    Binary(Vec<u8>),       // Raw bytes
    Form(UrlEncodedForm),  // URL-encoded form data
    Files(MultiForm),      // Multipart form data
    Json(Value),           // JSON data
    Empty,                 // No body
    Unparsed,             // Not yet parsed
    Buffer {              // Raw buffer with metadata
        data: Vec<u8>,
        content_type: HttpContentType,
        content_coding: ContentCodings,
    }
}
```

Key methods:
- `into_static()`: Converts body to bytes and updates metadata headers
- `parse_buffer()`: Parses buffer based on content-type
- `read_buffer()`: Reads body from stream with safety limits

### 5. Context System (`context.rs`)

#### HttpContext (formerly HttpReqCtx/HttpResCtx)
Unified context for request/response handling:
```rust
pub struct HttpContext {
    pub request: HttpRequest,
    pub response: HttpResponse,
    pub executable: Executable,  // Server or Client mode
    pub host: Option<String>,    // For client connections
    pub safety: HttpSafety,      // Safety configuration
    pub params: Params,          // Middleware parameters
    pub locals: Locals,          // Middleware state
}
```

#### Executable Enum
Distinguishes between server and client contexts:
```rust
pub enum Executable {
    Request { app: Arc<App>, endpoint: Arc<Url<HttpContext>> }, // Server
    Response,  // Client
}
```

### 6. Network Operations (`net.rs`)

Core networking functions that handle the actual I/O:

#### parse_lazy()
Parses HTTP messages from network streams:
- Reads headers first via `HttpMeta::from_stream()`
- Then reads body based on headers
- Respects safety limits and timeouts

#### send()
Sends HTTP messages to network streams:
1. Calls `body.into_static(&mut meta)` to:
   - Convert body to bytes
   - Set Content-Length header
   - Set Content-Type header if needed
2. Calls `meta.represent()` to format headers
3. Writes headers and body to stream
4. Flushes the stream

### 7. Safety and Limits (`safety.rs`)

#### HttpSafety
Configuration for security limits:
- Maximum body size
- Maximum header size
- Allowed HTTP methods
- Allowed content types
- Timeout settings

### 8. Supporting Types

#### Forms (`form.rs`)
- `UrlEncodedForm`: x-www-form-urlencoded data
- `MultiForm`: multipart/form-data with file uploads

#### Cookies (`cookie.rs`)
- `Cookie`: Individual cookie with attributes
- `CookieMap`: Collection of cookies

#### HTTP Values (`http_value.rs`)
- `HttpMethod`: GET, POST, PUT, DELETE, etc.
- `HttpContentType`: MIME type parsing
- `StatusCode`: HTTP status codes
- `HttpVersion`: HTTP/1.0, HTTP/1.1

#### Headers
- `HeaderValue`: Single or multiple header values
- `ContentDisposition`: File download headers
- `ContentCodings`: Compression encodings

## Data Flow

### Server Request Handling

1. **Connection Accept**: TCP connection established
2. **Protocol Detection**: `HTTP::detect()` identifies HTTP/1.1
3. **Request Parsing**: 
   - `parse_lazy()` reads from stream
   - `HttpMeta::from_stream()` parses headers
   - `HttpBody::read_buffer()` reads body
4. **Context Creation**: `HttpContext::new_server()` with app and endpoint
5. **Handler Execution**: Endpoint runs with context
6. **Response Generation**: Handler modifies `context.response`
7. **Response Sending**: 
   - `response.send()` calls `net::send()`
   - `body.into_static()` sets headers
   - `meta.represent()` formats headers
   - Body bytes written to stream

### Client Request Flow

1. **Context Creation**: `HttpContext::new_client()` with host
2. **Request Building**: Set method, path, headers, body
3. **Connection**: Establish TCP/TLS connection
4. **Request Sending**: Same as response sending above
5. **Response Reading**: `parse_lazy()` reads response
6. **Response Processing**: Client handles response

## Integration Points

### With Protocol Layer
- `HTTP` implements `Protocol` trait
- `HttpTransport` implements `Transport` trait
- `HttpMessage` implements `Message` trait
- `HttpContext` implements `RequestContext` trait

### With Application Layer
- URLs registered with `Arc<Url<HttpContext>>`
- Middleware operates on `HttpContext`
- Handlers receive and return `HttpContext`

### With Connection Layer
- Uses `TcpConnectionStream` for I/O
- Supports both plain TCP and TLS
- Handles connection pooling via keep-alive

## Key Design Patterns

1. **Lazy Parsing**: Bodies are not parsed until needed
2. **Zero-Copy Where Possible**: Direct byte slices for efficiency
3. **Type Safety**: Strong typing for all HTTP components
4. **Unified Context**: Single context type for both directions
5. **Metadata Precedence**: Field values override header map
6. **Safety First**: All operations respect configured limits

## Performance Optimizations

1. **Buffer Reuse**: Pre-allocated buffers for headers
2. **Streaming**: Support for chunked encoding
3. **Keep-Alive**: Connection reuse for multiple requests
4. **Selective Parsing**: Only parse what's needed
5. **Direct Serialization**: `represent()` builds headers efficiently

## Error Handling

- Connection errors bubble up to protocol handler
- Malformed requests return 400 Bad Request
- Size limit violations return 413 Payload Too Large
- Method not allowed returns 405
- Unsupported media type returns 415

## Future Enhancements

1. **HTTP/2 Support**: Via separate Http2Protocol
2. **WebSocket Upgrade**: Protocol switching support
3. **Compression**: gzip/deflate encoding
4. **Caching**: ETag and Last-Modified support
5. **Range Requests**: Partial content delivery