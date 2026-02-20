# Connection Module Architecture

> Last updated: 2026-02-19
> Status: Active redesign — legacy types still present, migration in progress

---

## New Design Overview

The connection module has been redesigned around four composable traits that give each
application a **static, zero-cost transport policy**. One port binds to exactly one
stream type; there is no runtime dispatch over steam kinds.

```
TcpListener → TcpStream → Accepter::upgrade() → TS::Wire (ConnStream)
                                                       │
                              ┌────────────────────────┤
                              ▼                        ▼
                    ReadHalf<TS::Wire>      WriteHalf<TS::Wire>
                              │                        │
                        ConnMeta (addresses, handshake data)
                              │
                    Protocol::handle(reader, writer, meta, app)
```

---

## Traits

### `ConnStream` — `stream.rs`

The wire-level stream abstraction. Every concrete transport implements this.

```rust
pub trait ConnStream: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static {
    type ReadHalf: AsyncRead + Unpin + Send + 'static;
    type WriteHalf: AsyncWrite + Unpin + Send + 'static;
    type Meta: ConnMeta;

    fn split(self) -> (Self::ReadHalf, Self::WriteHalf, Self::Meta);
    fn peer_addr(&self) -> std::io::Result<SocketAddr>;
    fn local_addr(&self) -> std::io::Result<SocketAddr>;
}
```

`shutdown()` is NOT in this trait — it is available for free via `AsyncWriteExt`
on any `WriteHalf: AsyncWrite`.

### `ConnMeta` — `stream.rs`

Extensible metadata captured at split-time (socket addresses, TLS handshake
data, etc.). Carried into every protocol handler call.

```rust
pub trait ConnMeta: Send + Sync + 'static {
    fn local_addr(&self)  -> Option<SocketAddr> { None }
    fn remote_addr(&self) -> Option<SocketAddr> { None }
}
```

### `Accepter` — `accepter.rs`

Server-side stream upgrader. Config lives **inside the struct**, not in the
method signature — supporting stateful acceptors (hot-reloadable TLS certs,
ALPN policy, connection limits).

```rust
#[async_trait]
pub trait Accepter: Send + Sync + 'static {
    type Stream: ConnStream;
    async fn upgrade(&self, tcp: TcpStream) -> std::io::Result<Self::Stream>;
}
```

### `Connector` — `connector.rs`

Client-side connection establisher. Same principle: config is in the struct.

```rust
#[async_trait]
pub trait Connector: Send + Sync + 'static {
    type Stream: ConnStream;
    type Target;
    async fn connect(&self, target: Self::Target) -> std::io::Result<Self::Stream>;
}
```

### `TransportSpec` — `transport_spec.rs`

Static policy that ties one Wire type to one Accepter and one Connector.
This is what `App<TS>` is generic over.

```rust
pub trait TransportSpec: Send + Sync + 'static {
    type Wire:      ConnStream;
    type Accepter:  Accepter<Stream  = Self::Wire>;
    type Connector: Connector<Stream = Self::Wire>;
}
```

---

## Concrete Implementations

### Plain TCP — `tcp.rs`

| Item | Type |
|------|------|
| `TcpStream impl ConnStream` | Wire = `TcpStream` |
| `TcpAccepter impl Accepter` | No-op, passthrough |
| `TcpConnector impl Connector` | Target = `String` ("host:port") |
| `TcpConnectorAddr impl Connector` | Target = `SocketAddr` |
| `TcpTransport impl TransportSpec` | Wire = `TcpStream` |

### TLS — `tls.rs` *(modules currently commented out in connection.rs)*

| Item | Type |
|------|------|
| `ClientTlsStream<TcpStream> impl ConnStream` | Used by `TlsConnector` |
| `ServerTlsStream<TcpStream> impl ConnStream` | Used by `TlsAccepter` |
| `TlsStream impl ConnStream` | **Unified enum** — required for `TransportSpec` |
| `TlsAccepter impl Accepter` | Server handshake; wraps result as `TlsStream::Server` |
| `TlsConnector impl Connector` | Client handshake; wraps result as `TlsStream::Client` |
| `TlsTransport impl TransportSpec` | Wire = `TlsStream` *(pending)* |

**Why `TlsStream` enum?**
`TransportSpec` requires `Accepter::Stream == Connector::Stream == Wire`. But
`tokio_rustls` gives distinct `client::TlsStream` and `server::TlsStream` types.
The unified `TlsStream { Client(...), Server(...) }` enum is the bridge.

### TLS Configuration

| File | Purpose |
|------|---------|
| `tls_config.rs` | Server TLS: cert chain, private key, client auth (mTLS), ALPN |
| `tls_client_config.rs` | Client TLS: trusted roots, client cert, ALPN, verification skip |

Both use a builder pattern. Config is baked into the accepter/connector struct at
construction time — never passed per-connection.

### Legacy — `legacy_stream.rs`

Kept for backward compatibility with existing HTTP code. **Do not use in new code.**

| Item | Notes |
|------|-------|
| `TcpConnectionStream` enum | Wraps `TcpStream \| TlsStream<TcpStream>`; still exports `ConnStream` impl |
| `TcpReader` / `TcpWriter` | Buffered wrappers with address caching; replaced by `ConnMeta` |
| `split_connection()` | Returns `(TcpReader, TcpWriter)`; replaced by `ConnStream::split()` |

---

## App Layer Alignment

### Done ✅

| Component | Status |
|-----------|--------|
| `App<TS: TransportSpec>` | Generic; stores `accepter: TS::Accepter` |
| `App::handle_connection()` | Calls `accepter.upgrade(tcp)` → `TS::Wire` |
| `Protocol` trait | Generic over `Self::Wire: ConnStream`; handle takes raw halves + `Meta` |
| `ProtocolRegistry<TS>` | Generic over `TransportSpec` |
| `ProtocolHandler<P>` where `TS: TransportSpec<Wire=P::Wire>` | Correctly bound |
| `ProtocolRegistryBuilder<TS>` | Correctly generic |

### Not Done ❌ — Known Gaps

| Component | Gap | File |
|-----------|-----|------|
| `Http1Protocol::handle` | Uses `TcpReader`/`TcpWriter` instead of generic halves; missing `meta` param | `http/traits.rs` |
| `HttpContext::read_request` | Hardcoded to `ReadHalf<TcpConnectionStream>` | `http/context.rs:204` |
| `HttpContext::send_response` | Hardcoded to `WriteHalf<TcpConnectionStream>` | `http/context.rs:217` |
| `HttpContext::send_request` | Client request uses `TcpConnector` directly; hardcoded TCP-only | `http/context.rs:534` |
| `HttpContext::write_frame` | `BufWriter<WriteHalf<TcpConnectionStream>>` | `http/context.rs:632` |
| `HttpContext::read_next_frame` | `BufReader<ReadHalf<TcpConnectionStream>>` | `http/context.rs:647` |
| TLS modules | Commented out in `connection.rs` | `connection.rs:11-13` |
| `builder.rs` | `ConnectionBuilder` still exported; uses legacy types | `connection.rs:15` |

---

## Migration Guide (for new protocols)

Implement `Protocol` correctly by using generic halves, not `TcpReader`/`TcpWriter`:

```rust
use hotaru_core::connection::{ConnStream, TransportSpec};
use hotaru_core::protocol::Protocol;

#[async_trait]
impl Protocol for MyProtocol {
    type Wire = TcpStream;   // or TlsStream, or your own ConnStream impl
    // ... other associated types ...

    async fn handle<TS>(
        &mut self,
        reader: BufReader<<Self::Wire as ConnStream>::ReadHalf>,
        writer: <Self::Wire as ConnStream>::WriteHalf,
        meta: <Self::Wire as ConnStream>::Meta,       // socket addrs + handshake data
        app: Arc<App<TS>>,
    ) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        TS: TransportSpec<Wire = Self::Wire>,
    {
        let remote = meta.remote_addr();
        // ... protocol logic using reader/writer directly ...
    }
}
```

---

## File Map

```
hotaru_core/src/connection/
├── connection.rs           re-exports (primary entry point)
├── stream.rs               ConnStream + ConnMeta traits
├── accepter.rs             Accepter trait
├── connector.rs            Connector trait
├── transport_spec.rs       TransportSpec trait
├── tcp.rs                  TcpStream/TcpAccepter/TcpConnector/TcpTransport impls
├── tls.rs                  TlsStream enum + TlsAccepter/TlsConnector impls
├── tls_config.rs           Server-side TLS configuration builder
├── tls_client_config.rs    Client-side TLS configuration builder
├── legacy_stream.rs        TcpConnectionStream, TcpReader, TcpWriter (do not use)
├── builder.rs              ConnectionBuilder (do not use in new code)
├── error.rs                ConnectionError
└── test.rs                 Integration tests (some still use legacy signatures)
```
