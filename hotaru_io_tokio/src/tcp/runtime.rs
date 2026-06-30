//! TCP inbound and outbound runtime objects.

use hotaru_core::connection::{Accepter, Inbound, Outbound};
use tokio::net::TcpListener;

use super::{primitive::TcpAccepter, stream::TcpStream};

/// Bound plain TCP inbound runtime.
pub struct TcpInbound {
    listener: TcpListener,
    accepter: TcpAccepter,
}

impl Inbound for TcpInbound {
    type Wire = TcpStream;
    type BindTarget = String;
    type Error = std::io::Error;

    async fn bind(target: Self::BindTarget) -> Result<Self, Self::Error> {
        Ok(Self {
            listener: TcpListener::bind(target).await?,
            accepter: TcpAccepter,
        })
    }

    async fn accept(&self) -> Result<Self::Wire, Self::Error> {
        let (tcp, _) = self.listener.accept().await?;
        self.accepter.upgrade(tcp).await
    }
}

/// TCP outbound runtime using normal `TcpStream::connect`.
pub struct TcpOutbound {
    target: String,
}

impl TcpOutbound {
    /// Returns the remote target this outbound is bound to.
    pub fn target(&self) -> &str {
        &self.target
    }
}

impl Outbound for TcpOutbound {
    type Wire = TcpStream;
    type ConnectTarget = String;
    type Error = std::io::Error;

    async fn build(target: Self::ConnectTarget) -> Result<Self, Self::Error> {
        Ok(Self { target })
    }

    async fn connect(&self) -> Result<Self::Wire, Self::Error> {
        TcpStream::connect(&self.target).await
    }
}
