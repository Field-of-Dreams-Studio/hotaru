//! Plain TCP transport implementation.

pub mod primitive;
pub mod runtime;
pub mod stream;
pub mod transport;

pub use primitive::{TcpAccepter, TcpConnector, TcpConnectorAddr};
pub use runtime::{TcpInbound, TcpOutbound};
pub use stream::TcpMeta;
pub use transport::TcpTransport;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::{Accepter, ConnStream};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};

    #[tokio::test]
    async fn test_tcp_stream_split() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            socket.write_all(b"pong").await.unwrap();
        });

        let stream = TcpStream::connect(addr).await.unwrap();
        let (mut read, _write, _meta) = ConnStream::split(stream);

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

        assert_eq!(ConnStream::peer_addr(&stream).unwrap(), server_addr);

        let local = ConnStream::local_addr(&stream).unwrap();
        assert!(local.port() > 0);
    }

    #[tokio::test]
    async fn test_tcp_accepter() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let server_addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            let (tcp, _) = listener.accept().await.unwrap();
            let accepter = TcpAccepter;
            let mut stream = accepter.upgrade(tcp).await.unwrap();

            let mut buf = [0u8; 4];
            stream.read_exact(&mut buf).await.unwrap();
            stream.write_all(&buf).await.unwrap();
        });

        let mut client = TcpStream::connect(server_addr).await.unwrap();
        client.write_all(b"ping").await.unwrap();

        let mut response = [0u8; 4];
        client.read_exact(&mut response).await.unwrap();
        assert_eq!(&response, b"ping");
    }
}
