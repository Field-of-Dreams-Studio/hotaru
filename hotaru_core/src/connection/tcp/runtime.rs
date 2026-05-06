//! TCP inbound and outbound runtime objects.

use async_trait::async_trait;
use tokio::net::{TcpListener, TcpStream};

use crate::connection::{Accepter, Inbound, Outbound};

use super::primitive::TcpAccepter;

/// Bound plain TCP inbound runtime.
pub struct TcpInbound {
    listener: TcpListener,
    accepter: TcpAccepter,
}

#[async_trait]
impl Inbound for TcpInbound {
    type Wire = TcpStream;
    type BindTarget = String;

    async fn bind(target: Self::BindTarget) -> std::io::Result<Self> {
        Ok(Self {
            listener: TcpListener::bind(target).await?,
            accepter: TcpAccepter,
        })
    }

    async fn accept(&self) -> std::io::Result<Self::Wire> {
        let (tcp, _) = self.listener.accept().await?;
        self.accepter.upgrade(tcp).await
    }
}

/// TCP outbound runtime using normal `TcpStream::connect`.
///
/// The OS chooses the local address and port.
pub struct TcpOutbound;

#[async_trait]
impl Outbound for TcpOutbound {
    type Wire = TcpStream;
    type ConnectTarget = String;

    async fn connect(target: Self::ConnectTarget) -> std::io::Result<Self::Wire> {
        TcpStream::connect(target).await
    }
}
