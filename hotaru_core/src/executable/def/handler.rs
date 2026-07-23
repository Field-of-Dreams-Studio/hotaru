//! Trait-based route flavour distinction.
//!
//! `FinalHandlerDef<P>` separates endpoints from outpoints at the type
//! level. `EndpointHandler<P>` supplies just a final handler (the
//! user's body); `OutpointHandler<P>` supplies a body-as-middleware
//! plus `<P as Protocol>::send` as the fixed final handler. 

use crate::executable::middleware::{AsyncFinalHandler, AsyncMiddleware};
use crate::marker::{MaybeSendBoxFuture, MaybeSendSync};
use crate::prelude::{Arc, Box};
use crate::protocol::{EndpointOutcome, Protocol, RequestContext};

/// Defines a route's final handler plus any body-middleware prefix
/// the flavour requires. Endpoints have no prefix; outpoints have
/// exactly one prefix entry (the user's outpoint body wrapped as
/// middleware).
pub trait FinalHandlerDef<P: Protocol>: 'static {
    // Final handler is in the center of the executable middleware chain. 
    fn final_handler(&self) -> Arc<dyn AsyncFinalHandler<P::Context>>;
    // Body middleware is in the most outside of the middleware chain 
    fn body_middleware(&self) -> Option<Arc<dyn AsyncMiddleware<P::Context>>>;
}

/// Endpoint flavour: user body is the final handler; no prefix.
pub struct EndpointHandler<P: Protocol> {
    body: Arc<dyn AsyncFinalHandler<P::Context>> 
}

impl<P: Protocol> EndpointHandler<P> {
    pub fn new(body: Arc<dyn AsyncFinalHandler<P::Context>>) -> Self {
        Self {
            body 
        }
    }

    /// Normalize a borrowed endpoint body into the owned final-handler
    /// contract used by the executable middleware chain.
    pub fn from_async_fn<R, H>(handler: H) -> Self
    where
        R: EndpointOutcome<P::Context> + 'static,
        H: for<'a> Fn(&'a mut P::Context) -> MaybeSendBoxFuture<'a, R> + MaybeSendSync + 'static,
    {
        let handler = Arc::new(handler);

        Self::new(Arc::new(
            move |mut context: P::Context| -> MaybeSendBoxFuture<
                'static,
                Result<P::Context, <P::Context as RequestContext>::Error>,
            > {
                let handler = handler.clone();
                Box::pin(async move {
                    let outcome = handler(&mut context).await;
                    outcome.apply_to(&mut context)?;
                    Ok(context)
                })
            },
        ))
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
    body: Arc<dyn AsyncMiddleware<P::Context>> 
}

impl<P: Protocol> OutpointHandler<P> {
    pub fn new(body: Arc<dyn AsyncMiddleware<P::Context>>) -> Self {
        Self {
            body 
        }
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
