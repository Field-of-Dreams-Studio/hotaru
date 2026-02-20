# TLS Configuration Usage Guide

## Overview

The `TlsConfig` provides a flexible, builder-pattern configuration for server-side TLS connections with support for:

- ✅ Certificate and private key loading (from files or bytes)
- ✅ Mutual TLS (client certificate authentication)
- ✅ ALPN protocol negotiation (e.g., HTTP/2, HTTP/1.1)
- ✅ Secure defaults using rustls 0.23

## Basic HTTPS Server

```rust
use hotaru_core::connection::Accepter;
use hotaru_tls::{TlsAccepter, TlsConfig};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Configure TLS
    let tls_config = TlsConfig::builder()
        .cert_chain_file("/path/to/server-cert.pem")?
        .private_key_file("/path/to/server-key.pem")?
        .build()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    // Create TLS accepter
    let accepter = TlsAccepter::new(tls_config)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    // Listen for connections
    let listener = TcpListener::bind("0.0.0.0:443").await?;

    loop {
        let (tcp, peer_addr) = listener.accept().await?;

        // Upgrade to TLS
        let tls_stream = accepter.upgrade(tcp, &()).await?;

        // Handle the secure connection
        tokio::spawn(async move {
            // Your protocol handler here
            println!("Secure connection from: {}", peer_addr);
        });
    }
}
```

## HTTP/2 with ALPN

```rust
use hotaru_tls::TlsConfig;

let tls_config = TlsConfig::builder()
    .cert_chain_file("server-cert.pem")?
    .private_key_file("server-key.pem")?
    .alpn_protocols(&["h2", "http/1.1"])  // Advertise HTTP/2 first
    .build()?;
```

## Mutual TLS (mTLS) - Required Client Certificates

```rust
use hotaru_tls::TlsConfig;

// Server requires client certificates signed by ca-cert.pem
let tls_config = TlsConfig::builder()
    .cert_chain_file("server-cert.pem")?
    .private_key_file("server-key.pem")?
    .require_client_auth("ca-cert.pem")?  // Clients MUST present certs
    .build()?;
```

## Optional Client Certificates

```rust
use hotaru_tls::TlsConfig;

// Server accepts client certificates but doesn't require them
let tls_config = TlsConfig::builder()
    .cert_chain_file("server-cert.pem")?
    .private_key_file("server-key.pem")?
    .optional_client_auth("ca-cert.pem")?  // Clients MAY present certs
    .build()?;
```

## Loading from Memory (Embedded Certificates)

```rust
use hotaru_tls::TlsConfig;

const SERVER_CERT: &[u8] = include_bytes!("../../certs/server-cert.pem");
const SERVER_KEY: &[u8] = include_bytes!("../../certs/server-key.pem");

let tls_config = TlsConfig::builder()
    .cert_chain_pem(SERVER_CERT)?
    .private_key_pem(SERVER_KEY)?
    .build()?;
```

## Multi-Protocol Server with Generic Accepter

```rust
use hotaru_core::connection::Accepter;
use hotaru_tls::TlsAccepter;
use hotaru_core::connection::tcp::TcpAccepter;

// HTTP server (port 80)
let http_accepter = TcpAccepter;  // Plain TCP, no TLS

// HTTPS server (port 443)
let https_config = TlsConfig::builder()
    .cert_chain_file("server-cert.pem")?
    .private_key_file("server-key.pem")?
    .alpn_protocols(&["h2", "http/1.1"])
    .build()?;
let https_accepter = TlsAccepter::new(https_config)?;

// Both use the same Accepter trait!
async fn handle_connection<A: Accepter>(
    accepter: &A,
    tcp: tokio::net::TcpStream,
) -> std::io::Result<A::Stream> {
    accepter.upgrade(tcp, &()).await
}
```

## Certificate Chain with Intermediates

```rust
// server-fullchain.pem contains:
// 1. Server certificate
// 2. Intermediate CA certificate
// 3. (Optionally) Root CA certificate

let tls_config = TlsConfig::builder()
    .cert_chain_file("server-fullchain.pem")?  // Rustls handles the chain
    .private_key_file("server-key.pem")?
    .build()?;
```

## Error Handling

```rust
use hotaru_tls::{TlsConfig, TlsConfigError};

match TlsConfig::builder()
    .cert_chain_file("server-cert.pem")
    .and_then(|b| b.private_key_file("server-key.pem"))
    .and_then(|b| b.build())
{
    Ok(config) => {
        println!("TLS configured successfully");
    }
    Err(TlsConfigError::IoError(e)) => {
        eprintln!("File not found or permission denied: {}", e);
    }
    Err(TlsConfigError::InvalidCertificate(e)) => {
        eprintln!("Invalid certificate: {}", e);
    }
    Err(TlsConfigError::InvalidKey(e)) => {
        eprintln!("Invalid private key: {}", e);
    }
    Err(e) => {
        eprintln!("TLS configuration error: {}", e);
    }
}
```

## Generating Self-Signed Certificates (Testing Only)

```bash
# Generate private key
openssl genrsa -out server-key.pem 2048

# Generate self-signed certificate (valid for 365 days)
openssl req -new -x509 -key server-key.pem -out server-cert.pem -days 365

# For testing with specific hostname
openssl req -new -x509 -key server-key.pem -out server-cert.pem -days 365 \
  -subj "/CN=localhost"
```

## Security Best Practices

1. **Never commit private keys to version control**
   - Use `.gitignore` to exclude `*.pem`, `*.key` files
   - Use environment variables or secret management systems

2. **Use full certificate chains**
   - Include intermediate certificates in your chain
   - Some clients don't have intermediate CAs cached

3. **Keep certificates updated**
   - Set up automated renewal (e.g., Let's Encrypt certbot)
   - Monitor expiration dates

4. **Use strong keys**
   - Minimum 2048-bit RSA or 256-bit ECDSA
   - Consider ECDSA for better performance

5. **Enable mutual TLS for sensitive services**
   - Use `require_client_auth()` for internal microservices
   - Verify client certificates against a known CA

## Comparison with Client TLS (ConnectionBuilder)

| Feature | `TlsConfig` (Server) | `ConnectionBuilder` (Client) |
|---------|---------------------|------------------------------|
| Purpose | Accept TLS connections | Initiate TLS connections |
| Certificates | Server cert + key required | Optional custom root CA |
| Client Auth | Configurable (none/optional/required) | N/A |
| ALPN | Advertises protocols | Can negotiate protocols |
| Usage | With `TlsAccepter` | With `ConnectionBuilder::tls(true)` |

## Architecture

```
TCP Accept Flow:
┌─────────────┐
│ TcpListener │
│  .accept()  │
└──────┬──────┘
       │ TcpStream
       ▼
┌─────────────┐
│ TlsAccepter │ ◄── TlsConfig
│  .upgrade() │
└──────┬──────┘
       │ TlsStream<TcpStream>
       ▼
┌─────────────┐
│  Protocol   │
│  .handle()  │
└─────────────┘
```

## Related Types

- `TlsAccepter` - Server-side TLS handshake handler
- `TcpAccepter` - No-op accepter for plain TCP
- `Accepter` trait - Generic stream upgrader interface
- `Stream` trait - Generic stream abstraction
- `ConnectionBuilder` - Client-side connection builder (includes TLS client)

## Next Steps

See `tcp.rs` for the `TcpAccepter` implementation as a simpler example.
See `accepter.rs` for the `Accepter` trait definition.
