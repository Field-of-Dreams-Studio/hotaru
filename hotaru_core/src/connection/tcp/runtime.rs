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
/// The OS chooses the local address and port. The remote target is
/// captured at `build` time and used by every `connect` call.
pub struct TcpOutbound {
    target: String,
}

impl TcpOutbound {
    /// Returns the remote target this outbound is bound to.
    pub fn target(&self) -> &str {
        &self.target
    }
}

#[async_trait]
impl Outbound for TcpOutbound {
    type Wire = TcpStream;
    type ConnectTarget = String;

    async fn build(target: Self::ConnectTarget) -> std::io::Result<Self> {
        // No work at build time for plain TCP — DNS resolution and socket
        // creation happen per-`connect`. Future TLS / pooled variants can
        // do more here.
        Ok(Self { target })
    }

    async fn connect(&self) -> std::io::Result<Self::Wire> {
        TcpStream::connect(&self.target).await
    }
}
