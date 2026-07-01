//! TCP primitive accepter and connector implementations.

use core::convert::Infallible;
use core::net::SocketAddr;

use hotaru_core::connection::{Accepter, Connector};
use tokio::net::TcpStream as TokioTcpStream;

use super::stream::TcpStream;

/// Plain TCP accepter that wraps accepted Tokio streams.
///
/// Cannot fail — the "upgrade" is a zero-cost newtype wrap. `type Error =
/// Infallible` so callers can unwrap without runtime cost via `?` or
/// `.into_ok()` (once stable).
#[derive(Debug, Clone, Copy, Default)]
pub struct TcpAccepter;

impl Accepter for TcpAccepter {
    type Raw = TokioTcpStream;
    type Stream = TcpStream;
    type Error = Infallible;

    async fn upgrade(&self, raw: Self::Raw) -> Result<Self::Stream, Self::Error> {
        Ok(TcpStream::new(raw))
    }
}

/// Plain TCP outbound connector.
///
/// Errors are the tokio-native `std::io::Error` — no wrapping.
#[derive(Debug, Clone, Copy, Default)]
pub struct TcpConnector;

impl Connector for TcpConnector {
    type Stream = TcpStream;
    type Target = String;
    type Error = std::io::Error;

    async fn connect(&self, target: Self::Target) -> Result<Self::Stream, Self::Error> {
        TcpStream::connect(target).await
    }
}

/// TCP connector that accepts a resolved socket address.
#[derive(Debug, Clone, Copy, Default)]
pub struct TcpConnectorAddr;

impl Connector for TcpConnectorAddr {
    type Stream = TcpStream;
    type Target = SocketAddr;
    type Error = std::io::Error;

    async fn connect(&self, target: Self::Target) -> Result<Self::Stream, Self::Error> {
        TcpStream::connect(target).await
    }
}
