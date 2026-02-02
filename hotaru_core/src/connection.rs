pub mod connection; 
pub mod stream; 
pub mod error; 
pub mod builder; 
pub mod protocol; 
pub mod compat;
pub mod test; 

pub use self::builder::ConnectionBuilder;  
#[allow(deprecated)]
pub use self::compat::{LegacyConnectionBuilder, LegacyProtocol};
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
