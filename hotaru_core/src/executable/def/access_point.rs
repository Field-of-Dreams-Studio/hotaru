use core::marker::PhantomData;

use akari::extensions::ParamsClone;

use crate::executable::middleware::{AsyncFinalHandler, AsyncMiddleware};
use crate::prelude::{Arc, String, Vec};
use crate::protocol::Protocol;
use crate::url::{PathPattern, StepName, tokens_to_patterns};

use super::error::BindError;
use super::handler::{EndpointHandler, FinalHandlerDef, OutpointHandler};
use super::middleware::{MWChain, MWSlot};
use super::route_address::RouteAddress;
use super::url_mode::UrlMode;

/// Pre-registration route definition.
///
/// Structure mirrors `hotaru_trans::UrlArgs` on the macro side:
/// `address` (analog to `UrlExpr`), a middleware chain, a flavour-
/// specific handler (analog to `UrlFunc`), and a `ParamsClone`
/// config. `T: FinalHandlerDef<P>` supplies the flavour. Use
/// `Endpoint<P>` / `Outpoint<P>` aliases in almost all call sites;
/// the generic form is for advanced hand-writers.
pub struct AccessPointDef<P: Protocol, T: FinalHandlerDef<P>> {
    address: RouteAddress,
    /// Defaults to `[Inherit]` for both endpoints and outpoints.
    middlewares: MWChain<P::Context>,
    handler: T,
    config: ParamsClone,
    _protocol: PhantomData<fn() -> P>,
}

/// Endpoint route: user body is the final handler.
pub type Endpoint<P> = AccessPointDef<P, EndpointHandler<P>>;
/// Outpoint route: user body wraps as middleware, `Protocol::send` is
/// the final handler.
pub type Outpoint<P> = AccessPointDef<P, OutpointHandler<P>>;

impl<P: Protocol, T: FinalHandlerDef<P>> AccessPointDef<P, T> {
    /// Generic constructor. Sets the normative `[Inherit]` default
    /// user chain. Prefer `Endpoint::endpoint` / `Outpoint::outpoint`
    /// at call sites; use this only when the flavour is generic.
    pub fn new(
        url: impl Into<String>,
        name: impl Into<String>,
        handler: T,
    ) -> Self {
        Self::with_address(RouteAddress::new(url, name), handler)
    }

    /// Constructor from an already-built `RouteAddress`. Useful in
    /// macro expansions and in tests that want to reuse an address
    /// across multiple defs.
    pub fn with_address(address: RouteAddress, handler: T) -> Self {
        Self {
            address,
            middlewares: MWChain::inheriting(),
            handler,
            config: ParamsClone::default(),
            _protocol: PhantomData,
        }
    }

    // ----- builders -----

    pub fn with_url_mode(mut self, mode: UrlMode) -> Self {
        self.address = self.address.with_url_mode(mode);
        self
    }

    pub fn with_middleware(
        mut self,
        middleware: Arc<dyn AsyncMiddleware<P::Context>>,
    ) -> Self {
        self.middlewares.push(MWSlot::Concrete(middleware));
        self
    }

    pub fn with_inherit(mut self) -> Self {
        self.middlewares.push(MWSlot::Inherit);
        self
    }

    /// Drop the default `[Inherit]` entry. Combined with
    /// `with_middleware`, this reproduces `middleware = [A, B]`
    /// semantics.
    pub fn no_inherit(mut self) -> Self {
        self.middlewares.remove_inherit();
        self
    }

    /// Replace the user chain wholesale. Outpoint bodies stay
    /// untouched — they live in `handler`, not `middlewares`.
    pub fn with_middlewares(mut self, middlewares: Vec<MWSlot<P::Context>>) -> Self {
        self.middlewares = MWChain::new(middlewares);
        self
    }

    pub fn with_config(mut self, config: ParamsClone) -> Self {
        self.config = config;
        self
    }

    // ----- read-only inspection (delegating to sub-structs) -----

    pub fn address(&self) -> &RouteAddress { &self.address }
    pub fn url(&self) -> &str { self.address.url() }
    pub fn name(&self) -> &str { self.address.name() }
    pub fn url_mode(&self) -> UrlMode { self.address.url_mode() }
    pub fn middlewares(&self) -> &[MWSlot<P::Context>] { self.middlewares.as_slice() }
    pub fn handler(&self) -> &T { &self.handler }
    pub fn config(&self) -> &ParamsClone { &self.config }

    // ----- crate-private accessors for preparation -----

    /// Parse this definition's URL into the representation consumed by
    /// the registry, retaining route identity in any parse error.
    pub(crate) fn parse_url_pattern(
        &self,
    ) -> Result<(Vec<PathPattern>, StepName), BindError> {
        match self.url_mode() {
            UrlMode::Pattern => {
                let tokens = P::tokenize_url(self.url())
                    .map_err(|error| BindError::new(self.name(), self.url(), error.into()))?;
                let (path, step_names) = tokens_to_patterns(&tokens)
                    .map_err(|error| BindError::new(self.name(), self.url(), error.into()))?;

                Ok((path, step_names.into()))
            }
            UrlMode::Literal => Ok((
                P::lit_parser(self.url())
                    .into_iter()
                    .map(PathPattern::literal_path)
                    .collect(),
                StepName::default(),
            )),
        }
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        RouteAddress,
        MWChain<P::Context>,
        T,
        ParamsClone,
    ) {
        (self.address, self.middlewares, self.handler, self.config)
    }
}

// Convenience constructors on the two aliases so users can write
// `Endpoint::endpoint(url, name, handler_arc)` instead of
// turbofishing the generic parent.
impl<P: Protocol> Endpoint<P> {
    pub fn endpoint(
        url: impl Into<String>,
        name: impl Into<String>,
        handler: Arc<dyn AsyncFinalHandler<P::Context>>,
    ) -> Self {
        Self::new(url, name, EndpointHandler::new(handler))
    }
}

impl<P: Protocol> Outpoint<P> {
    pub fn outpoint(
        url: impl Into<String>,
        name: impl Into<String>,
        body: Arc<dyn AsyncMiddleware<P::Context>>,
    ) -> Self {
        Self::new(url, name, OutpointHandler::new(body))
    }
} 
