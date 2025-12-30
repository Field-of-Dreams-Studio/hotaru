//! This module provides an abstraction over plain TCP and TLS connections using Tokio.
//!
//! The `TcpConnectionStream` enum encapsulates either a plain TCP stream or a TLS stream.
//! This allows consumers to work with either connection type transparently.
//!
//! By separating the connection from buffering, users of this module can choose to apply buffering
//! (e.g., via `tokio::io::BufReader` or `tokio::io::BufWriter`) as necessary in their application.

use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{self, AsyncBufRead, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, BufWriter, ReadBuf, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;

/// Represents a connection which can be either plain TCP or secured with TLS.
pub enum TcpConnectionStream {
    /// A plain TCP connection.
    Tcp(TcpStream),
    /// A secure TLS connection built on top of a TCP stream.
    Tls(TlsStream<TcpStream>),
}

impl TcpConnectionStream {
    /// Reconstructs a TcpConnectionStream from split read and write halves.
    /// 
    /// This is used after protocol detection to pass the full stream to the protocol handler.
    pub fn from_parts(read_half: ReadHalf<Self>, write_half: WriteHalf<Self>) -> Self {
        read_half.unsplit(write_half)
    }
    
    /// Creates a new `Connection` instance wrapping a plain TCP stream.
    ///
    /// # Arguments
    ///
    /// * `stream` - A `TcpStream` representing the underlying TCP connection.
    ///
    /// # Returns
    ///
    /// A `Connection::Tcp` variant wrapping the provided `TcpStream`.
    pub fn new_tcp(stream: TcpStream) -> Self {
        TcpConnectionStream::Tcp(stream)
    }

    /// Creates a new `Connection` instance wrapping a TLS stream.
    ///
    /// # Arguments
    ///
    /// * `stream` - A `TlsStream<TcpStream>` representing the underlying TLS-secured connection.
    ///
    /// # Returns
    ///
    /// A `Connection::Tls` variant wrapping the provided `TlsStream<TcpStream>`.
    pub fn new_tls(stream: TlsStream<TcpStream>) -> Self {
        TcpConnectionStream::Tls(stream)
    } 
    

    /// Provides mutable access to the underlying stream for read operations.
    ///
    /// # Returns
    ///
    /// A mutable reference to a type that implements `AsyncRead`. This can be used to perform
    /// read operations on the connection.
    pub fn reader_mut(&mut self) -> &mut (dyn AsyncRead + Unpin) {
        match self {
            TcpConnectionStream::Tcp(stream) => stream,
            TcpConnectionStream::Tls(stream) => stream,
        }
    } 

    /// Splits the connection into separate read and write halves.
    ///
    /// Note: This uses `tokio::io::split` to separate the underlying stream. The returned halves
    /// can be used concurrently in separate tasks.
    ///
    /// # Returns
    ///
    /// A tuple where the first element is the read half implementing `AsyncRead`
    /// and the second element is the write half implementing `AsyncWrite`.
    pub fn split(self) -> (ReadHalf<Self>, WriteHalf<Self>)
    where
        Self: AsyncRead + AsyncWrite + Unpin,
    {
        io::split(self)
    } 

    /// Provides mutable access to the underlying stream for write operations.
    ///
    /// # Returns
    ///
    /// A mutable reference to a type that implements `AsyncWrite`. This can be used to perform
    /// write operations on the connection.
    pub fn writer_mut(&mut self) -> &mut (dyn AsyncWrite + Unpin) {
        match self {
            TcpConnectionStream::Tcp(stream) => stream,
            TcpConnectionStream::Tls(stream) => stream,
        }
    } 

    /// Gracefully shuts down the connection by closing the write half.
    ///
    /// This sends a FIN packet (TCP) or TLS close_notify alert to notify the peer
    /// that no more data will be sent. Reads can still be performed after shutdown.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use hotaru_core::connection::stream::TcpConnectionStream;
    /// # async fn example(mut conn: TcpConnectionStream) {
    /// let _ = conn.shutdown().await;
    /// # }
    /// ```
    pub async fn shutdown(&mut self) -> std::io::Result<()> {
        // Use pattern matching to call the appropriate shutdown method
        match self {
            TcpConnectionStream::Tcp(stream) => stream.shutdown().await,
            TcpConnectionStream::Tls(stream) => stream.shutdown().await,
        }
    }

    /// Returns the remote peer's socket address.
    ///
    /// # Errors
    ///
    /// Returns any underlying I/O error if the peer address cannot be retrieved.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use hotaru_core::connection::TcpConnectionStream;
    /// use tokio::net::TcpListener;
    ///
    /// # async fn example() -> std::io::Result<()> {
    /// let listener = TcpListener::bind("127.0.0.1:0").await?;
    /// let addr = listener.local_addr()?;
    /// let _client = tokio::net::TcpStream::connect(addr).await?;
    /// let (server_stream, _client_addr) = listener.accept().await?;
    /// let conn = TcpConnectionStream::new_tcp(server_stream);
    /// let _peer = conn.peer_addr()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn peer_addr(&self) -> std::io::Result<SocketAddr> {
        match self {
            TcpConnectionStream::Tcp(stream) => stream.peer_addr(),
            // tokio_rustls 0.26: get_ref() returns (&TcpStream, &ClientConnection)
            TcpConnectionStream::Tls(stream) => stream.get_ref().0.peer_addr(),
        }
    }

    /// Returns the local socket address.
    ///
    /// # Errors
    ///
    /// Returns any underlying I/O error if the local address cannot be retrieved.
    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        match self {
            TcpConnectionStream::Tcp(stream) => stream.local_addr(),
            TcpConnectionStream::Tls(stream) => stream.get_ref().0.local_addr(),
        }
    }
}

impl AsyncRead for TcpConnectionStream {
    /// Polls the `Connection` for reading data asynchronously.
    ///
    /// This implementation delegates the read operation to the u nderlying stream, whether
    /// it is a plain TCP or TLS stream.
    ///
    /// # Arguments
    ///
    /// * `cx` - The asynchronous task context.
    /// * `buf` - The buffer for storing the read data.
    ///
    /// # Returns
    ///
    /// A `Poll` indicating if the operation is ready or pending.
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>, 
    ) -> Poll<std::io::Result<()>> {
        // Convert the pinned reference of self into a mutable reference to the enum, then match on it.
        match self.get_mut() {
            TcpConnectionStream::Tcp(stream) => Pin::new(stream).poll_read(cx, buf),
            TcpConnectionStream::Tls(stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for TcpConnectionStream {
    /// Polls the `Connection` for writing data asynchronously.
    ///
    /// This implementation delegates the write operation to the underlying stream, whether
    /// it is a plain TCP or TLS stream.
    ///
    /// # Arguments
    ///
    /// * `cx` - The asynchronous task context.
    /// * `buf` - The buffer containing data to write.
    ///
    /// # Returns
    ///
    /// A `Poll` indicating the result of the write operation.
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        match self.get_mut() {
            TcpConnectionStream::Tcp(stream) => Pin::new(stream).poll_write(cx, buf),
            TcpConnectionStream::Tls(stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    /// Polls the `Connection` for flushing written data asynchronously.
    ///
    /// This operation ensures that all buffered data in the underlying stream is pushed out.
    ///
    /// # Arguments
    ///
    /// * `cx` - The asynchronous task context.
    ///
    /// # Returns
    ///
    /// A `Poll` indicating the result of the flush operation.
    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            TcpConnectionStream::Tcp(stream) => Pin::new(stream).poll_flush(cx),
            TcpConnectionStream::Tls(stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    /// Polls the `Connection` to shutdown the write half of the connection asynchronously.
    ///
    /// This operation signals that no more data will be written to the connection.
    ///
    /// # Arguments
    ///
    /// * `cx` - The asynchronous task context.
    ///
    /// # Returns
    ///
    /// A `Poll` indicating the result of the shutdown operation.
    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            TcpConnectionStream::Tcp(stream) => Pin::new(stream).poll_shutdown(cx),
            TcpConnectionStream::Tls(stream) => Pin::new(stream).poll_shutdown(cx),
        }
    }
} 

#[cfg(test)]
mod tests {
    use super::{split_connection, TcpConnectionStream};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};

    #[tokio::test]
    async fn peer_and_local_addr_are_available_for_tcp() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let listener_addr = listener.local_addr().unwrap();

        let client = TcpStream::connect(listener_addr).await.unwrap();
        let (server_stream, client_addr) = listener.accept().await.unwrap();

        let server_conn = TcpConnectionStream::new_tcp(server_stream);
        let client_conn = TcpConnectionStream::new_tcp(client);

        assert_eq!(server_conn.peer_addr().unwrap(), client_addr);
        assert_eq!(server_conn.local_addr().unwrap(), listener_addr);

        assert_eq!(client_conn.peer_addr().unwrap(), listener_addr);
        assert_eq!(client_conn.local_addr().unwrap(), client_addr);
    }

    #[tokio::test]
    async fn tcp_reader_writer_split_preserves_addrs_and_io() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let listener_addr = listener.local_addr().unwrap();

        let mut client = TcpStream::connect(listener_addr).await.unwrap();
        let (server_stream, client_addr) = listener.accept().await.unwrap();

        let server_conn = TcpConnectionStream::new_tcp(server_stream);
        let (mut reader, mut writer) = split_connection(server_conn);

        assert_eq!(reader.local_addr(), Some(listener_addr));
        assert_eq!(reader.remote_addr(), Some(client_addr));

        client.write_all(b"ping").await.unwrap();
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"ping");

        writer.write_all(b"pong").await.unwrap();
        writer.flush().await.unwrap();
        let mut reply = [0u8; 4];
        client.read_exact(&mut reply).await.unwrap();
        assert_eq!(&reply, b"pong");
    }

    #[tokio::test]
    async fn tcp_reader_fill_buf_and_consume() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let listener_addr = listener.local_addr().unwrap();

        let mut client = TcpStream::connect(listener_addr).await.unwrap();
        let (server_stream, _client_addr) = listener.accept().await.unwrap();

        let server_conn = TcpConnectionStream::new_tcp(server_stream);
        let (mut reader, _writer) = split_connection(server_conn);

        client.write_all(b"h").await.unwrap();
        let buf = reader.fill_buf().await.unwrap();
        assert_eq!(buf, b"h");
        let len = buf.len();
        reader.consume(len);
        assert!(reader.buffer().is_empty());
    }
}

// ============================================================================
// TcpReader / TcpWriter - Buffered wrappers with connection metadata
// ============================================================================

/// Buffered TCP reader with connection metadata.
pub struct TcpReader {
    inner: BufReader<ReadHalf<TcpConnectionStream>>,
    local_addr: Option<SocketAddr>,
    remote_addr: Option<SocketAddr>,
}

/// Buffered TCP writer.
pub struct TcpWriter {
    inner: BufWriter<WriteHalf<TcpConnectionStream>>,
}

impl TcpReader {
    /// Creates a new TcpReader.
    pub fn new(
        inner: BufReader<ReadHalf<TcpConnectionStream>>,
        local_addr: Option<SocketAddr>,
        remote_addr: Option<SocketAddr>,
    ) -> Self {
        Self { inner, local_addr, remote_addr }
    }

    /// Returns the local (server) socket address.
    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.local_addr
    }

    /// Returns the remote (client) socket address.
    pub fn remote_addr(&self) -> Option<SocketAddr> {
        self.remote_addr
    }

    /// Fills the internal buffer, returning a slice to the buffered data.
    pub async fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        tokio::io::AsyncBufReadExt::fill_buf(&mut self.inner).await
    }

    /// Tells this buffer that `amt` bytes have been consumed.
    pub fn consume(&mut self, amt: usize) {
        tokio::io::AsyncBufReadExt::consume(&mut self.inner, amt)
    }

    /// Returns a reference to the internally buffered data.
    pub fn buffer(&self) -> &[u8] {
        self.inner.buffer()
    }
}

impl TcpWriter {
    /// Creates a new TcpWriter.
    pub fn new(inner: BufWriter<WriteHalf<TcpConnectionStream>>) -> Self {
        Self { inner }
    }

    /// Flushes the internal buffer.
    pub async fn flush(&mut self) -> std::io::Result<()> {
        tokio::io::AsyncWriteExt::flush(&mut self.inner).await
    }

    /// Shuts down the write half of the connection.
    pub async fn shutdown(&mut self) -> std::io::Result<()> {
        tokio::io::AsyncWriteExt::shutdown(&mut self.inner).await
    }
}

/// Splits a connection into TcpReader and TcpWriter, capturing socket addresses.
pub fn split_connection(conn: TcpConnectionStream) -> (TcpReader, TcpWriter) {
    let local_addr = conn.local_addr().ok();
    let remote_addr = conn.peer_addr().ok();
    let (read_half, write_half) = conn.split();
    (
        TcpReader::new(BufReader::new(read_half), local_addr, remote_addr),
        TcpWriter::new(BufWriter::new(write_half)),
    )
}

// --- AsyncRead for TcpReader ---
impl AsyncRead for TcpReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

// --- AsyncBufRead for TcpReader ---
impl AsyncBufRead for TcpReader {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<&[u8]>> {
        Pin::new(&mut self.get_mut().inner).poll_fill_buf(cx)
    }

    fn consume(mut self: Pin<&mut Self>, amt: usize) {
        Pin::new(&mut self.inner).consume(amt)
    }
}

// --- AsyncWrite for TcpWriter ---
impl AsyncWrite for TcpWriter {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}
