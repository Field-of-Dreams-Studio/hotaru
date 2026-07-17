/// Named access points for registered handlers (runtime lookup handles).
pub mod access;

/// Pre-registration route definitions (`AccessPointDef<P, T>`).
pub mod def;

/// Protocol entry builder and runtime entry types.
pub mod entry;
/// Executable handler bindings and execution chains.
pub mod executable;
/// Async middleware traits and middleware chains.
pub mod middleware;
/// Protocol registry builder and registry storage.
pub mod registry;

pub use def::{
    AccessPointDef, BindError, Endpoint, EndpointHandler, FinalHandlerDef, MWChain, MWSlot,
    Outpoint, OutpointHandler, RouteAddress, UrlMode,
};
pub use entry::ProtocolEntryBuilder;
pub use executable::{ExecutableBinding, ExecutionChain, run_chain};
pub use registry::ProtocolRegistryBuilder;
