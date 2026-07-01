use core::net::SocketAddr;

use crate::connection::{HotaruRead, HotaruWrite, MaybeSend};

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
///
/// Framework-owned supertraits only. Under `std`, the tokio blanket gives
/// every `tokio::io::AsyncRead`/`AsyncWrite` type these impls for free, so
/// existing tokio-based transports (`TcpStream`, `TlsStream`) satisfy the
/// bound without code changes. Under `embedded`, the embedded-io-async
/// blanket plays the same role.
pub trait ConnStream: HotaruRead + HotaruWrite + Unpin + MaybeSend + Sync + 'static {
    /// Read half type produced by `split`.
    type ReadHalf: HotaruRead + Unpin + MaybeSend + 'static;

    /// Write half type produced by `split`.
    ///
    /// Note: `shutdown()` is available via `HotaruWrite`.
    type WriteHalf: HotaruWrite + Unpin + MaybeSend + 'static;

    /// Connection metadata produced by `split`.
    type Meta: ConnMeta;

    /// Split the stream into read and write halves.
    fn split(self) -> (Self::ReadHalf, Self::WriteHalf, Self::Meta);

    /// Returns the remote peer's socket address when available.
    fn peer_addr(&self) -> Option<SocketAddr>;

    /// Returns the local socket address when available.
    fn local_addr(&self) -> Option<SocketAddr>;
}
