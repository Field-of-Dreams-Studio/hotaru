//! Plain TCP stream implementation of ConnStream.

use async_trait::async_trait;
use std::net::SocketAddr;
use tokio::io::{AsyncRead, AsyncWrite, ReadHalf, WriteHalf};
use tokio::net::TcpStream;

use super::accepter::Accepter;
use super::connector::Connector;
use super::stream::{ConnMeta, ConnStream};

/// Connection metadata for plain TCP connections.
pub struct TcpMeta {
    local: Option<SocketAddr>,
    remote: Option<SocketAddr>,
}

impl ConnMeta for TcpMeta {
    fn local_addr(&self) -> Option<SocketAddr> { self.local }
    fn remote_addr(&self) -> Option<SocketAddr> { self.remote }
}

impl ConnStream for TcpStream {
    type ReadHalf = ReadHalf<TcpStream>;
    type WriteHalf = WriteHalf<TcpStream>;
    type Meta = TcpMeta;

    fn split(self) -> (Self::ReadHalf, Self::WriteHalf, Self::Meta) {
        let meta = TcpMeta {
            local: self.local_addr().ok(),
            remote: self.peer_addr().ok(),
        };
        let (r, w) = tokio::io::split(self);
        (r, w, meta)
    }

    fn peer_addr(&self) -> std::io::Result<SocketAddr> {
        self.peer_addr()
    }

    fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.local_addr()
    }
}

// ============================================================================
// TcpAccepter - No-op accepter for plain TCP
// ============================================================================

/// Plain TCP accepter that performs no upgrade.
///
/// This accepter simply returns the raw TCP stream without any encryption
/// or additional handshake. Use this for unencrypted HTTP, custom protocols,
/// or when TLS termination is handled by a reverse proxy.
///
/// # Example
/// ```no_run
/// use hotaru_core::connection::tcp::TcpAccepter;
/// use hotaru_core::connection::Accepter;
/// use tokio::net::TcpListener;
///
/// # async fn example() -> std::io::Result<()> {
/// let listener = TcpListener::bind("127.0.0.1:8080").await?;
/// let accepter = TcpAccepter;
///
/// loop {
///     let (tcp, _peer_addr) = listener.accept().await?;
///     let stream = accepter.upgrade(tcp).await?;
///     // Handle the stream with your protocol...
/// }
/// # }
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct TcpAccepter;

#[async_trait]
impl Accepter for TcpAccepter {
    type Stream = TcpStream;

    async fn upgrade(&self, tcp: TcpStream) -> std::io::Result<Self::Stream> {
        // No-op: just return the stream as-is
        Ok(tcp)
    }
}

// ============================================================================
// TcpConnector - Plain TCP outbound connector
// ============================================================================

/// Plain TCP connector for outbound connections.
///
/// This connector establishes TCP connections without encryption.
/// Use this for:
/// - Plain HTTP clients
/// - Internal service communication (when TLS termination is at edge)
/// - Custom protocols without encryption
/// - Testing and development
///
/// # Example
/// ```no_run
/// use hotaru_core::connection::tcp::TcpConnector;
/// use hotaru_core::connection::Connector;
///
/// # async fn example() -> std::io::Result<()> {
/// let connector = TcpConnector;
///
/// // Connect to a server
/// let stream = connector.connect("example.com:80".to_string()).await?;
///
/// // Use the stream with your protocol...
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct TcpConnector;

#[async_trait]
impl Connector for TcpConnector {
    type Stream = TcpStream;
    type Target = String; // "host:port" format

    async fn connect(&self, target: Self::Target) -> std::io::Result<Self::Stream> {
        TcpStream::connect(target).await
    }
}

/// Alternative connector that accepts SocketAddr directly.
#[derive(Debug, Clone, Copy, Default)]
pub struct TcpConnectorAddr;

#[async_trait]
impl Connector for TcpConnectorAddr {
    type Stream = TcpStream;
    type Target = SocketAddr;

    async fn connect(&self, target: Self::Target) -> std::io::Result<Self::Stream> {
        TcpStream::connect(target).await
    }
}

// ============================================================================
// TcpTransport - TransportSpec implementation for plain TCP
// ============================================================================

/// Transport policy for plain (unencrypted) TCP.
///
/// Binds together `TcpStream` as the wire type with `TcpAccepter`
/// (server side) and `TcpConnector` (client side).
///
/// # Example
/// ```no_run
/// use hotaru_core::connection::tcp::TcpTransport;
/// use hotaru_core::connection::TransportSpec;
///
/// fn requires_transport<T: TransportSpec>() {}
/// requires_transport::<TcpTransport>();
/// ```
pub struct TcpTransport;

impl super::transport_spec::TransportSpec for TcpTransport {
    type Wire = TcpStream;
    type Accepter = TcpAccepter;
    type Connector = TcpConnector;

    fn default_accepter() -> Option<Self::Accepter> {
        Some(TcpAccepter)
    }

    fn default_connector() -> Option<Self::Connector> {
        Some(TcpConnector)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test_tcp_stream_split() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Spawn server
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            socket.write_all(b"pong").await.unwrap();
        });

        // Client
        let stream = TcpStream::connect(addr).await.unwrap();
        let (mut read, _write) = ConnStream::split(stream);

        let mut buf = [0u8; 4];
        read.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"pong");
    }

    #[tokio::test]
    async fn test_tcp_addresses() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let server_addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            let _ = listener.accept().await;
        });

        let stream = TcpStream::connect(server_addr).await.unwrap();

        assert_eq!(
            ConnStream::peer_addr(&stream).unwrap(),
            server_addr
        );

        let local = ConnStream::local_addr(&stream).unwrap();
        assert!(local.port() > 0); // Ephemeral port
    }

    #[tokio::test]
    async fn test_tcp_accepter() {
        use super::super::accepter::Accepter;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let server_addr = listener.local_addr().unwrap();

        // Spawn server
        tokio::spawn(async move {
            let (tcp, _) = listener.accept().await.unwrap();
            let accepter = TcpAccepter;
            let mut stream = accepter.upgrade(tcp).await.unwrap();

            // Echo server
            let mut buf = [0u8; 4];
            use tokio::io::AsyncReadExt;
            stream.read_exact(&mut buf).await.unwrap();
            use tokio::io::AsyncWriteExt;
            stream.write_all(&buf).await.unwrap();
        });

        // Client
        let mut client = TcpStream::connect(server_addr).await.unwrap();
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        client.write_all(b"ping").await.unwrap();

        let mut response = [0u8; 4];
        client.read_exact(&mut response).await.unwrap();
        assert_eq!(&response, b"ping");
    }
}
