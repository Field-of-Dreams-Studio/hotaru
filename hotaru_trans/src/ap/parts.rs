use crate::{
    config::{Cloneable, Config},
    middleware::MWChain,
};

use super::RouteAddress;

/// Address, middleware, and config components shared by every AP flavour.
pub(crate) struct APParts {
    address: RouteAddress,
    middlewares: MWChain,
    config: Config,
}

impl APParts {
    /// Construct with the same default middleware chain as core: `[Inherit]`.
    pub(crate) fn new(address: RouteAddress) -> Self {
        Self {
            address,
            middlewares: MWChain::inheriting(),
            config: Config::new(Vec::new(), Cloneable::Yes),
        }
    }

    pub(crate) fn with_middlewares(mut self, middlewares: MWChain) -> Self {
        self.middlewares = middlewares;
        self
    }

    pub(crate) fn with_config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    pub(crate) fn address(&self) -> &RouteAddress {
        &self.address
    }

    pub(crate) fn middlewares(&self) -> &MWChain {
        &self.middlewares
    }

    pub(crate) fn config(&self) -> &Config {
        &self.config
    }

    pub(crate) fn into_parts(self) -> (RouteAddress, MWChain, Config) {
        (self.address, self.middlewares, self.config)
    }
}
