/// Shared server/client builder implementation.
pub mod builder;
/// Runtime-facing operational configuration.
pub mod operational_config;
/// Development/production run-mode marker.
pub mod runmode;
/// Shared runtime configuration and extension storage.
pub mod runtime;

pub use builder::AppBuilder;
pub use operational_config::{OperationalConfig, TimeoutSetting};
pub use runmode::RunMode;
pub use runtime::RuntimeConfig;
