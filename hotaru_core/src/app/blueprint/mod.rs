//! Reusable protocol and access-point definitions.
//!
//! Skeleton stage: this module currently declares the storage data model
//! and its re-exports only. Behavioral `impl` blocks land one at a time in
//! the following segments.

mod access_points;
mod blueprint;
mod configured;
mod error;
mod homo;
mod protocol_def;
mod target;

pub use access_points::AccessPoints;
pub use blueprint::Blueprint;
pub use configured::ConfiguredBlueprint;
pub use error::BlueprintError;
pub use homo::HomoBlueprint;
pub use protocol_def::ProtocolDef;

// Crate-private plumbing. Re-exported at the `blueprint` path so external
// code naming `HomoBluePrintTrait` receives E0603 (private), not E0433
// (unresolved). Never make these `pub`.
#[allow(unused_imports)]
pub(crate) use homo::{ErasedHomoBlueprint, HomoBluePrintTrait};
#[allow(unused_imports)]
pub(crate) use target::TargetGroups;
