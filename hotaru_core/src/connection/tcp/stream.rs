//! TCP wire metadata and `ConnStream` implementation.

use std::net::SocketAddr;
use tokio::io::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;

use crate::connection::{ConnMeta, ConnStream};

/// Connection metadata for plain TCP.
pub struct TcpMeta {
    local: Option<SocketAddr>,
    remote: Option<SocketAddr>,
}

impl ConnMeta for TcpMeta {
    fn local_addr(&self) -> Option<SocketAddr> {
        self.local
    }

    fn remote_addr(&self) -> Option<SocketAddr> {
        self.remote
    }
}

impl ConnStream for TcpStream {
    type ReadHalf = ReadHalf<TcpStream>;
    type WriteHalf = WriteHalf<TcpStream>;
    type Meta = TcpMeta;

    fn split(self) -> (Self::ReadHalf, Self::WriteHalf, Self::Meta) {
        let meta = TcpMeta {
            local: self.local_addr().ok(),
            remote: self.peer_addr().ok(),
        };
        let (read, write) = tokio::io::split(self);
        (read, write, meta)
    }

    fn peer_addr(&self) -> std::io::Result<SocketAddr> {
        self.peer_addr()
    }

    fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.local_addr()
    }
}
