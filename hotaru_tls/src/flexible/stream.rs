//! Flexible TCP/TLS stream — runtime-determined transport.
//!
//! The `TcpOrTlsStream` enum encapsulates either a plain TCP stream or a
//! (client-side) TLS stream, allowing consumers to work with either type
//! transparently via the `ConnStream` trait.

use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{
    self, AsyncBufRead, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, BufWriter, ReadBuf,
    ReadHalf, WriteHalf,
};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;

use hotaru_core::connection::{ConnMeta, ConnStream};

/// Connection metadata for flexible TCP/TLS streams.
pub struct FlexMeta {
    local: Option<SocketAddr>,
    remote: Option<SocketAddr>,
}

impl ConnMeta for FlexMeta {
    fn local_addr(&self) -> Option<SocketAddr> { self.local }
    fn remote_addr(&self) -> Option<SocketAddr> { self.remote }
}

/// A connection that is either plain TCP or client-side TLS.
///
/// Use this when the transport is decided at runtime (e.g. based on a URL
/// scheme). For static TLS-only connections, prefer `TlsStream` +
/// `TransportSpec` instead.
pub enum TcpOrTlsStream {
    /// A plain TCP connection.
    Tcp(TcpStream),
    /// A TLS connection over TCP (client side).
    Tls(TlsStream<TcpStream>),
}

impl TcpOrTlsStream {
    /// Reconstruct a `TcpOrTlsStream` from split halves.
    pub fn from_parts(read_half: ReadHalf<Self>, write_half: WriteHalf<Self>) -> Self {
        read_half.unsplit(write_half)
    }

    /// Wrap a plain TCP stream.
    pub fn new_tcp(stream: TcpStream) -> Self {
        TcpOrTlsStream::Tcp(stream)
    }

    /// Wrap a TLS stream.
    pub fn new_tls(stream: TlsStream<TcpStream>) -> Self {
        TcpOrTlsStream::Tls(stream)
    }

    /// Returns the remote peer's socket address.
    pub fn peer_addr(&self) -> std::io::Result<SocketAddr> {
        match self {
            TcpOrTlsStream::Tcp(s) => s.peer_addr(),
            TcpOrTlsStream::Tls(s) => s.get_ref().0.peer_addr(),
        }
    }

    /// Returns the local socket address.
    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        match self {
            TcpOrTlsStream::Tcp(s) => s.local_addr(),
            TcpOrTlsStream::Tls(s) => s.get_ref().0.local_addr(),
        }
    }

    /// Gracefully shut down the write half.
    pub async fn shutdown(&mut self) -> std::io::Result<()> {
        match self {
            TcpOrTlsStream::Tcp(s) => s.shutdown().await,
            TcpOrTlsStream::Tls(s) => s.shutdown().await,
        }
    }
}

impl AsyncRead for TcpOrTlsStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            TcpOrTlsStream::Tcp(s) => Pin::new(s).poll_read(cx, buf),
            TcpOrTlsStream::Tls(s) => Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for TcpOrTlsStream {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        match self.get_mut() {
            TcpOrTlsStream::Tcp(s) => Pin::new(s).poll_write(cx, buf),
            TcpOrTlsStream::Tls(s) => Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            TcpOrTlsStream::Tcp(s) => Pin::new(s).poll_flush(cx),
            TcpOrTlsStream::Tls(s) => Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            TcpOrTlsStream::Tcp(s) => Pin::new(s).poll_shutdown(cx),
            TcpOrTlsStream::Tls(s) => Pin::new(s).poll_shutdown(cx),
        }
    }
}

impl ConnStream for TcpOrTlsStream {
    type ReadHalf = ReadHalf<Self>;
    type WriteHalf = WriteHalf<Self>;
    type Meta = FlexMeta;

    fn split(self) -> (Self::ReadHalf, Self::WriteHalf, Self::Meta) {
        let meta = FlexMeta {
            local: self.local_addr().ok(),
            remote: self.peer_addr().ok(),
        };
        let (r, w) = io::split(self);
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
// TcpReader / TcpWriter — buffered wrappers with address caching
// ============================================================================

/// Buffered TCP/TLS reader with connection metadata.
pub struct TcpReader {
    inner: BufReader<ReadHalf<TcpOrTlsStream>>,
    local_addr: Option<SocketAddr>,
    remote_addr: Option<SocketAddr>,
}

/// Buffered TCP/TLS writer.
pub struct TcpWriter {
    inner: BufWriter<WriteHalf<TcpOrTlsStream>>,
}

impl TcpReader {
    pub fn new(
        inner: BufReader<ReadHalf<TcpOrTlsStream>>,
        local_addr: Option<SocketAddr>,
        remote_addr: Option<SocketAddr>,
    ) -> Self {
        Self { inner, local_addr, remote_addr }
    }

    pub fn local_addr(&self) -> Option<SocketAddr> { self.local_addr }
    pub fn remote_addr(&self) -> Option<SocketAddr> { self.remote_addr }

    pub async fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        tokio::io::AsyncBufReadExt::fill_buf(&mut self.inner).await
    }

    pub fn consume(&mut self, amt: usize) {
        tokio::io::AsyncBufReadExt::consume(&mut self.inner, amt)
    }

    pub fn buffer(&self) -> &[u8] {
        self.inner.buffer()
    }
}

impl TcpWriter {
    pub fn new(inner: BufWriter<WriteHalf<TcpOrTlsStream>>) -> Self {
        Self { inner }
    }

    pub async fn flush(&mut self) -> std::io::Result<()> {
        tokio::io::AsyncWriteExt::flush(&mut self.inner).await
    }

    pub async fn shutdown(&mut self) -> std::io::Result<()> {
        tokio::io::AsyncWriteExt::shutdown(&mut self.inner).await
    }
}

/// Split a `TcpOrTlsStream` into buffered reader/writer, capturing socket addresses.
pub fn split_connection(conn: TcpOrTlsStream) -> (TcpReader, TcpWriter) {
    let local_addr = conn.local_addr().ok();
    let remote_addr = conn.peer_addr().ok();
    let (r, w, _meta) = ConnStream::split(conn);
    (
        TcpReader::new(BufReader::new(r), local_addr, remote_addr),
        TcpWriter::new(BufWriter::new(w)),
    )
}

impl AsyncRead for TcpReader {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl AsyncBufRead for TcpReader {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<&[u8]>> {
        Pin::new(&mut self.get_mut().inner).poll_fill_buf(cx)
    }

    fn consume(mut self: Pin<&mut Self>, amt: usize) {
        Pin::new(&mut self.inner).consume(amt)
    }
}

impl AsyncWrite for TcpWriter {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}
