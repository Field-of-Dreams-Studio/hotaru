//! Low-level transport primitives used by runtime transport objects.
//! App code should usually depend on `runtime`, not these traits directly.

pub mod accepter;
pub mod connector;

pub use accepter::Accepter;
pub use connector::Connector;
