//! TCP primitive accepter and connector implementations.

use core::net::SocketAddr;

use hotaru_core::connection::{Accepter, Connector};
use tokio::net::TcpStream as TokioTcpStream;

use super::stream::TcpStream;

/// Plain TCP accepter that wraps accepted Tokio streams.
#[derive(Debug, Clone, Copy, Default)]
pub struct TcpAccepter;

impl Accepter for TcpAccepter {
    type Raw = TokioTcpStream;
    type Stream = TcpStream;

    async fn upgrade(&self, raw: Self::Raw) -> std::io::Result<Self::Stream> {
        Ok(TcpStream::new(raw))
    }
}

/// Plain TCP outbound connector.
#[derive(Debug, Clone, Copy, Default)]
pub struct TcpConnector;

impl Connector for TcpConnector {
    type Stream = TcpStream;
    type Target = String;

    async fn connect(&self, target: Self::Target) -> std::io::Result<Self::Stream> {
        TcpStream::connect(target).await
    }
}

/// TCP connector that accepts a resolved socket address.
#[derive(Debug, Clone, Copy, Default)]
pub struct TcpConnectorAddr;

impl Connector for TcpConnectorAddr {
    type Stream = TcpStream;
    type Target = SocketAddr;

    async fn connect(&self, target: Self::Target) -> std::io::Result<Self::Stream> {
        TcpStream::connect(target).await
    }
}
