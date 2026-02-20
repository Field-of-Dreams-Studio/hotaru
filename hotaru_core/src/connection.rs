pub mod connection;
pub mod error;
pub mod test;
pub mod stream;
pub mod accepter;
pub mod connector;
pub mod transport_spec;
pub mod tcp;

pub use self::accepter::Accepter;
pub use self::connector::Connector;
pub use self::transport_spec::TransportSpec;
pub use self::stream::{ConnMeta, ConnStream};
pub use self::tcp::{TcpAccepter, TcpConnector, TcpConnectorAddr, TcpMeta, TcpTransport};
pub use self::error::Result;

// New protocol traits
pub use crate::protocol::{
    Protocol,
    Transport,
    Stream,
    Message,
    RequestContext,
    ProtocolRole,
    ProtocolIndex,
    ProtocolError,
    ProtocolErrorKind,
    ProtocolErrorBox,
    ProtocolResult,
    IoProtocolError,
    StaticProtocolError,
};
