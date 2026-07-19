use crate::{config::Config, middleware::MWChain};

use super::{APParts, OutpointMW, ParsedAP, RouteAddress};

/// Parsed `outpoint!` definition.
pub(crate) struct Outpoint {
    parts: APParts,
    body: OutpointMW,
}

impl Outpoint {
    pub(crate) fn new(address: RouteAddress, body: OutpointMW) -> Self {
        Self {
            parts: APParts::new(address),
            body,
        }
    }

    pub(crate) fn from_parsed(parsed: ParsedAP) -> Self {
        let (parts, handler) = parsed.into_parts();
        Self {
            parts,
            body: OutpointMW::new(handler),
        }
    }

    pub(crate) fn with_middlewares(mut self, middlewares: MWChain) -> Self {
        self.parts = self.parts.with_middlewares(middlewares);
        self
    }

    pub(crate) fn with_config(mut self, config: Config) -> Self {
        self.parts = self.parts.with_config(config);
        self
    }

    pub(crate) fn ap_parts(&self) -> &APParts {
        &self.parts
    }

    pub(crate) fn body(&self) -> &OutpointMW {
        &self.body
    }

    pub(crate) fn into_parts(self) -> (APParts, OutpointMW) {
        (self.parts, self.body)
    }
}
