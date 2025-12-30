pub mod connection; 
pub mod stream; 
pub mod error; 
pub mod builder; 
pub mod protocol; 
pub mod test; 

pub use self::builder::ConnectionBuilder;  
// TODO: Rename builder::Protocol to ConnectionProtocol or ClientProtocol
// pub use self::builder::Protocol; 
pub use self::stream::{TcpConnectionStream, TcpReader, TcpWriter, split_connection};
pub use self::error::Result; 

// New protocol traits
pub use self::protocol::{
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
};

// Alias for generic connection stream for compatibility
pub use self::stream::TcpConnectionStream as Connection; 
pub use self::connection::ConnectionStatus;


