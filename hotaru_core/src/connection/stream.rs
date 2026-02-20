use std::net::SocketAddr;
use tokio::io::{AsyncRead, AsyncWrite};

/// Per-connection metadata produced when a wire stream is split.
///
/// Keep this extensible so each transport can attach extra handshake/runtime
/// data (e.g. ALPN, peer cert info, proxy metadata) without changing core APIs.
pub trait ConnMeta: Send + Sync + 'static {
    /// Returns the local socket address when available.
    fn local_addr(&self) -> Option<SocketAddr> {
        None
    }

    /// Returns the remote peer socket address when available.
    fn remote_addr(&self) -> Option<SocketAddr> {
        None
    }
}

/// Stream abstraction for protocol-specific transports.
pub trait ConnStream: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static {
    /// Read half type produced by `split`.
    type ReadHalf: AsyncRead + Unpin + Send + 'static;

    /// Write half type produced by `split`.
    ///
    /// Note: `shutdown()` is available via `AsyncWriteExt` since `WriteHalf: AsyncWrite`.
    type WriteHalf: AsyncWrite + Unpin + Send + 'static;

    /// Connection metadata produced by `split`.
    type Meta: ConnMeta;

    /// Split the stream into read and write halves.
    fn split(self) -> (Self::ReadHalf, Self::WriteHalf, Self::Meta);

    /// Returns the remote peer's socket address.
    fn peer_addr(&self) -> std::io::Result<SocketAddr>;

    /// Returns the local socket address.
    fn local_addr(&self) -> std::io::Result<SocketAddr>;
}
