//! TCP primitive accepter and connector implementations.

use core::net::SocketAddr;
use tokio::net::TcpStream;

use crate::connection::{Accepter, Connector};

/// Plain TCP accepter that returns the accepted stream unchanged.
#[derive(Debug, Clone, Copy, Default)]
pub struct TcpAccepter;

impl Accepter for TcpAccepter {
    type Raw = TcpStream;
    type Stream = TcpStream;

    async fn upgrade(&self, raw: Self::Raw) -> std::io::Result<Self::Stream> {
        Ok(raw)
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
