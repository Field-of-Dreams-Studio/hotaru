//! TLS inbound and outbound runtime objects.

use async_trait::async_trait;
use hotaru_core::connection::{Accepter, Connector, Inbound, Outbound};
use tokio::net::TcpListener;

use super::{TlsAccepter, TlsConnector, TlsStream};
use crate::config::{TlsClientConfig, TlsConfig};

/// Server-side TLS bind target.
///
/// Carries both the TCP bind address and the TLS server config needed to
/// create the bound runtime.
#[derive(Clone)]
pub struct TlsInboundTarget {
    pub addr: String,
    pub config: TlsConfig,
}

impl TlsInboundTarget {
    pub fn new(addr: impl Into<String>, config: TlsConfig) -> Self {
        Self {
            addr: addr.into(),
            config,
        }
    }
}

/// Bound TLS inbound runtime.
pub struct TlsInbound {
    listener: TcpListener,
    accepter: TlsAccepter,
}

#[async_trait]
impl Inbound for TlsInbound {
    type Wire = TlsStream;
    type BindTarget = TlsInboundTarget;

    async fn bind(target: Self::BindTarget) -> std::io::Result<Self> {
        let listener = TcpListener::bind(target.addr).await?;
        let accepter = TlsAccepter::new(target.config)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err))?;

        Ok(Self { listener, accepter })
    }

    async fn accept(&self) -> std::io::Result<Self::Wire> {
        let (tcp, _) = self.listener.accept().await?;
        self.accepter.upgrade(tcp).await
    }
}

/// Client-side TLS connect target.
///
/// Carries the remote host/port and TLS client config. The hostname is also
/// used for SNI and certificate verification.
#[derive(Clone)]
pub struct TlsOutboundTarget {
    pub host: String,
    pub port: u16,
    pub config: TlsClientConfig,
}

impl TlsOutboundTarget {
    pub fn new(host: impl Into<String>, port: u16, config: TlsClientConfig) -> Self {
        Self {
            host: host.into(),
            port,
            config,
        }
    }
}

/// TLS outbound runtime bound to one remote target.
pub struct TlsOutbound {
    target: (String, u16),
    connector: TlsConnector,
}

impl TlsOutbound {
    pub fn target(&self) -> (&str, u16) {
        (&self.target.0, self.target.1)
    }
}

#[async_trait]
impl Outbound for TlsOutbound {
    type Wire = TlsStream;
    type ConnectTarget = TlsOutboundTarget;

    async fn build(target: Self::ConnectTarget) -> std::io::Result<Self> {
        let connector = TlsConnector::new(target.config)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err))?;

        Ok(Self {
            target: (target.host, target.port),
            connector,
        })
    }

    async fn connect(&self) -> std::io::Result<Self::Wire> {
        self.connector.connect(self.target.clone()).await
    }
}
