//! Reusable protocol and access-point definitions.
//!
//! A Blueprint retains typed access-point definitions grouped by concrete
//! protocol and AP flavour. It can be cloned cheaply and materialized
//! repeatedly into independent protocol registries.

mod access_points;
mod blueprint;
mod configured;
mod erased;
mod error;
mod homo;
mod target;

#[cfg(test)]
mod test;

pub use crate::executable::def::ProtocolDef;
pub use access_points::AccessPoints;
pub use blueprint::Blueprint;
pub use configured::ConfiguredBlueprint;
pub use error::BlueprintError;
pub use homo::HomoBlueprint;

// Crate-private plumbing. Re-exported at the `blueprint` path so external
// code naming `HomoBluePrintTrait` receives E0603 (private), not E0433
// (unresolved). Never make these `pub`.
#[allow(unused_imports)]
pub(crate) use erased::{ErasedHomoBlueprint, HomoBluePrintTrait};
#[allow(unused_imports)]
pub(crate) use target::TargetGroups;
