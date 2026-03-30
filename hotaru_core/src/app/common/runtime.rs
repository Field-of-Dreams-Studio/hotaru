use crate::extensions::{Locals, Params};

use super::RunMode;

/// Shared runtime-wide configuration used by both server and client runtimes.
#[derive(Default)]
pub struct RuntimeConfig {
    mode: RunMode,
    config: Params,
    statics: Locals,
}

impl RuntimeConfig {
    /// Creates an empty runtime config with default mode and stores.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a runtime config from prebuilt parts.
    pub fn from_parts(mode: RunMode, config: Params, statics: Locals) -> Self {
        Self {
            mode,
            config,
            statics,
        }
    }

    /// Consumes the runtime config and returns its owned parts.
    pub fn into_parts(self) -> (RunMode, Params, Locals) {
        (self.mode, self.config, self.statics)
    }

    /// Returns the configured run mode.
    pub fn mode(&self) -> RunMode {
        self.mode.clone()
    }

    /// Returns the runtime config storage.
    pub fn config(&self) -> &Params {
        &self.config
    }

    /// Returns the runtime statics storage.
    pub fn statics(&self) -> &Locals {
        &self.statics
    }

    /// Returns a typed config value if present.
    pub fn get_config<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
        self.config.get::<T>().cloned()
    }

    /// Returns a typed static value if present.
    pub fn get_static<T: Clone + Send + Sync + 'static>(&self, key: &str) -> Option<T> {
        self.statics.get::<T>(key).cloned()
    }
}
