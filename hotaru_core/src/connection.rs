pub mod accepter;
pub mod connection;
pub mod connector;
pub mod error;
pub mod stream;
pub mod tcp;
pub mod test;
pub mod transport_spec;

pub use self::accepter::Accepter;
pub use self::connector::Connector;
pub use self::error::Result;
pub use self::stream::{ConnMeta, ConnStream};
pub use self::tcp::{TcpAccepter, TcpConnector, TcpConnectorAddr, TcpMeta, TcpTransport};
pub use self::transport_spec::TransportSpec;

// New protocol traits
pub use crate::protocol::{
    IoProtocolError, Message, Protocol, ProtocolError, ProtocolErrorBox, ProtocolErrorKind,
    ProtocolIndex, ProtocolResult, ProtocolRole, RequestContext, StaticProtocolError, Stream,
    Transport,
};
