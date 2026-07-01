/// Protocol flow-control channel types.
pub mod channel;
/// Request context and endpoint outcome traits.
pub mod context;
/// Protocol error traits and default error types.
pub mod error;
/// Message buffer abstraction used by protocols.
pub mod message;
/// Main protocol trait.
pub mod protocol;
/// Logical stream abstraction for multiplexed protocols.
pub mod stream;
/// Protocol role and index helper types.
pub mod types;

pub use context::{EndpointOutcome, RequestContext};
pub use error::{BoxProtocolError, DefaultProtocolError, EmptyError, ProtocolError};
pub use message::Message;
pub use protocol::{Protocol, CtxError};
pub use stream::Stream;
pub use channel::{Channel, ProtocolFlow};
pub use types::{ProtocolIndex, ProtocolRole};
