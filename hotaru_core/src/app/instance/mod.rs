pub mod app;
pub mod client;
pub mod server;
pub mod target;

pub use app::App;
pub use client::Client;
pub use server::Server;
pub use target::{
    AppTarget, Both, InboundOnly, InboundState, InboundTarget, OutboundOnly, OutboundState,
    OutboundTarget,
};

/// Dual-role application with both inbound and outbound capabilities.
pub type Gateway<TS, Rt> = App<TS, Rt, Both>;
