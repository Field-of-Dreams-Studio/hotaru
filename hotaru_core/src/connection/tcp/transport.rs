//! Plain TCP transport policy.

use tokio::net::TcpStream;

use crate::connection::{Inbound, Outbound, TransportSpec};

use super::runtime::{TcpInbound, TcpOutbound};

/// Plain TCP transport.
pub struct TcpTransport;

impl TransportSpec for TcpTransport {
    type Wire = TcpStream;
    type IoError = std::io::Error;
    type Inbound = TcpInbound;
    type Outbound = TcpOutbound;

    fn default_inbound() -> Option<<Self::Inbound as Inbound>::BindTarget> {
        Some(String::from("127.0.0.1:3003"))
    }

    fn default_outbound() -> Option<<Self::Outbound as Outbound>::ConnectTarget> {
        None
    }
}
