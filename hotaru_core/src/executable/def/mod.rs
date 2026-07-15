//! Pre-registration route definitions.
//!
//! Route flavour is encoded at the type level via `FinalHandlerDef<P>`:
//! `Endpoint<P>` = `AccessPointDef<P, EndpointHandler<P>>`, `Outpoint<P>`
//! = `AccessPointDef<P, OutpointHandler<P>>`. `App::bind` accepts only
//! the flavours its role permits (`Server` = endpoints, `Client` =
//! outpoints, `Gateway` = both). Mismatches fail to compile.

mod access_point;
mod error;
mod handler;
mod middleware;
mod route_address;
mod url_mode;

#[cfg(test)]
mod test;

pub use access_point::{AccessPointDef, Endpoint, Outpoint};
pub use error::BindError;
pub use handler::{EndpointHandler, FinalHandlerDef, OutpointHandler};
pub use middleware::MiddlewareSlot;
pub(crate) use middleware::MiddlewareSlots;
pub use route_address::RouteAddress;
pub use url_mode::UrlMode;

// Re-exported for convenience so downstream code can spell prepared
// bindings through the def module in migration examples.
pub use super::executable::ExecutableBinding;
