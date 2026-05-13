pub mod connection;
pub mod error;
pub mod primitive;
pub mod runtime;
pub mod stream;
pub mod tcp;
pub mod test;
pub mod transport_spec;

pub use self::error::Result;
pub use self::primitive::{Accepter, Connector};
pub use self::runtime::{Inbound, Outbound};
pub use self::stream::{ConnMeta, ConnStream};
pub use self::tcp::{
    TcpAccepter, TcpConnector, TcpConnectorAddr, TcpInbound, TcpMeta, TcpOutbound, TcpTransport,
};
pub use self::transport_spec::TransportSpec;

// New protocol traits
pub use crate::protocol::{
    Message, Protocol,
    ProtocolIndex, ProtocolRole, RequestContext, Stream,
    transport::Transport,
    ProtocolError, BoxProtocolError, DefaultProtocolError,
};
