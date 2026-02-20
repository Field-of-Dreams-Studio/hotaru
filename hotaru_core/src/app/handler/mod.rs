//! App-level handler submodule.
//!
//! This module is reserved for protocol dispatch and handler wiring code.
//! Existing implementation still lives in `app/protocol.rs` and will be moved
//! here incrementally.

pub mod handler;
pub mod registry;
pub mod builder;

pub use handler::{ProtocolHandler, ProtocolHandlerTrait};
pub use registry::{ProtocolRegistry, ProtocolRegistryKind};
pub use builder::{ProtocolHandlerBuilder, ProtocolRegistryBuilder};
