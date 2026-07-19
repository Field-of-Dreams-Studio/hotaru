use crate::{config::Config, middleware::MWChain};

use super::{APParts, FinalHandler, ParsedAP, RouteAddress};

/// Parsed `endpoint!` definition.
pub(crate) struct Endpoint {
    parts: APParts,
    handler: FinalHandler,
}

impl Endpoint {
    pub(crate) fn new(address: RouteAddress, handler: FinalHandler) -> Self {
        Self {
            parts: APParts::new(address),
            handler,
        }
    }

    pub(crate) fn from_parsed(parsed: ParsedAP) -> Self {
        let (parts, handler) = parsed.into_parts();
        Self {
            parts,
            handler: FinalHandler::new(handler),
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

    pub(crate) fn handler(&self) -> &FinalHandler {
        &self.handler
    }

    pub(crate) fn into_parts(self) -> (APParts, FinalHandler) {
        (self.parts, self.handler)
    }
}
