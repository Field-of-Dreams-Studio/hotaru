# hotaru_tls

TLS transport for the [Hotaru](../hotaru) web framework. Plugs into `hotaru_core`'s `Inbound` / `Outbound` / `ConnStream` / `TransportSpec` abstractions so any Hotaru protocol can ride on TLS.

Most users reach these types via `hotaru_http`'s `tls` feature (or the umbrella `hotaru` crate's `https` feature). Direct consumption is for transport-level tooling.

## What's exposed

- `TlsStream`, `TlsAccepter`, `TlsConnector` — wire primitives.
- `TlsInbound` / `TlsOutbound` — runtime objects implementing `hotaru_core::Inbound` / `Outbound`.
- `TlsInboundTarget` / `TlsOutboundTarget` — bind / connect target shapes (host, port, config).
- `TlsTransport` — `TransportSpec` impl pinning `Wire = TlsStream`.
- `TlsConfig` (server) / `TlsClientConfig` (client) — rustls-backed config builders.
- `flexible::{TcpOrTlsStream, ConnectionBuilder}` — runtime TCP-or-TLS selection on one listener.

## Minimal HTTPS client (via the umbrella crate)

```rust
use hotaru::http::*;
use hotaru::prelude::*;

pub static CLIENT: Lazy<Arc<Client<TlsTransport>>> = Lazy::new(|| {
    Client::<TlsTransport>::new()
        .target(TlsOutboundTarget::new("example.com", 443, TlsClientConfig::default()))
        .single_protocol(ProtocolBuilder::new(HTTPS::client(HttpSafety::default())))
        .build()
});
```

Enable on the `hotaru` umbrella with `features = ["https"]` (which forwards to `hotaru_http/tls` which depends on `hotaru_tls`).

## Version

`0.8.0`. Depends on `hotaru_core = 0.8.0`.
