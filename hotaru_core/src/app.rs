/// Outbound client runtime.
pub mod client;
/// Shared app builder and runtime configuration types.
pub mod common;
/// Unified application storage, targets, and side-specific implementations.
pub mod instance;
/// Protocol registry wiring used by clients and servers.
pub mod registry;
/// Runtime abstraction traits and backend capabilities.
pub mod runtime;
/// Inbound server runtime.
pub mod server;

pub use instance::{
    App, AppTarget, Both, Client, Gateway, InboundOnly, InboundState, InboundTarget, OutboundOnly,
    OutboundState, OutboundTarget, Server,
};
