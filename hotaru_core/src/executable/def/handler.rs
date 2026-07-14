//! Trait-based route flavour distinction.
//!
//! `FinalHandlerDef<P>` separates endpoints from outpoints at the type
//! level. `EndpointHandler<P>` supplies just a final handler (the
//! user's body); `OutpointHandler<P>` supplies a body-as-middleware
//! plus `<P as Protocol>::send` as the fixed final handler.

use core::marker::PhantomData;

use crate::executable::middleware::{AsyncFinalHandler, AsyncMiddleware};
use crate::prelude::Arc;
use crate::protocol::Protocol;

/// Defines a route's final handler plus any body-middleware prefix
/// the flavour requires. Endpoints have no prefix; outpoints have
/// exactly one prefix entry (the user's outpoint body wrapped as
/// middleware).
pub trait FinalHandlerDef<P: Protocol>: 'static {
    fn final_handler(&self) -> Arc<dyn AsyncFinalHandler<P::Context>>;
    fn body_middleware(&self) -> Option<Arc<dyn AsyncMiddleware<P::Context>>>;
}

/// Endpoint flavour: user body is the final handler; no prefix.
pub struct EndpointHandler<P: Protocol> {
    body: Arc<dyn AsyncFinalHandler<P::Context>>,
    _p: PhantomData<fn() -> P>,
}

impl<P: Protocol> EndpointHandler<P> {
    pub fn new(body: Arc<dyn AsyncFinalHandler<P::Context>>) -> Self {
        Self { body, _p: PhantomData }
    }
}

impl<P: Protocol> FinalHandlerDef<P> for EndpointHandler<P> {
    fn final_handler(&self) -> Arc<dyn AsyncFinalHandler<P::Context>> {
        self.body.clone()
    }
    fn body_middleware(&self) -> Option<Arc<dyn AsyncMiddleware<P::Context>>> {
        None
    }
}

/// Outpoint flavour: user body becomes a middleware prefix; final
/// handler is `<P as Protocol>::send` (fn-item ZST wrapped in `Arc`,
/// coerced through the `AsyncFinalHandler` blanket impl at
/// `hotaru_core/src/executable/middleware.rs:44`).
pub struct OutpointHandler<P: Protocol> {
    body: Arc<dyn AsyncMiddleware<P::Context>>,
    _p: PhantomData<fn() -> P>,
}

impl<P: Protocol> OutpointHandler<P> {
    pub fn new(body: Arc<dyn AsyncMiddleware<P::Context>>) -> Self {
        Self { body, _p: PhantomData }
    }
}

impl<P: Protocol> FinalHandlerDef<P> for OutpointHandler<P> {
    fn final_handler(&self) -> Arc<dyn AsyncFinalHandler<P::Context>> {
        // The blanket impl `AsyncFinalHandler<C> for F where F: Fn(C)
        // -> Fut, Fut: Future` at
        // `hotaru_core/src/executable/middleware.rs:44` accepts this
        // closure directly — no `Box::pin` wrapping needed. The
        // blanket impl's `handle()` boxes internally.
        Arc::new(|ctx: P::Context| <P as Protocol>::send(ctx))
    }
    fn body_middleware(&self) -> Option<Arc<dyn AsyncMiddleware<P::Context>>> {
        Some(self.body.clone())
    }
}