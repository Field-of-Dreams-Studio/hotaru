//! App-facing transport runtime traits.
//! These bind local runtime state and produce final wire streams.

pub mod inbound;
pub mod outbound;

pub use inbound::Inbound;
pub use outbound::Outbound;
