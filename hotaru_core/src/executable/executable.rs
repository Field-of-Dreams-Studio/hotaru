use std::sync::Arc;

use crate::protocol::RequestContext;

use super::middleware::{AsyncFinalHandler, AsyncMiddleware, AsyncMiddlewareChain, BoxFuture};

/// A route- or node-level executable binding.
///
/// This stores the source-of-truth execution definition: the final handler and
/// the middleware list that should wrap it.
pub struct ExecutableBinding<C: RequestContext> {
    handler: Option<Arc<dyn AsyncFinalHandler<C>>>,
    middlewares: AsyncMiddlewareChain<C>,
}

impl<C: RequestContext> Clone for ExecutableBinding<C> {
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
            middlewares: self.middlewares.clone(),
        }
    }
}

impl<C: RequestContext> Default for ExecutableBinding<C> {
    fn default() -> Self {
        Self {
            handler: None,
            middlewares: Vec::new(),
        }
    }
}

impl<C: RequestContext> ExecutableBinding<C> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a cloned binding with a different final handler.
    pub fn with_handler(mut self, handler: Arc<dyn AsyncFinalHandler<C>>) -> Self {
        self.handler = Some(handler);
        self
    }

    /// Returns a cloned binding without a final handler.
    pub fn without_handler(mut self) -> Self {
        self.handler = None;
        self
    }

    /// Returns a cloned binding with a different middleware list.
    pub fn with_middlewares(mut self, middlewares: AsyncMiddlewareChain<C>) -> Self {
        self.middlewares = middlewares;
        self
    }

    /// Returns a cloned binding with one extra middleware appended.
    pub fn with_middleware(mut self, middleware: Arc<dyn AsyncMiddleware<C>>) -> Self {
        self.middlewares.push(middleware);
        self
    }

    pub fn handler(&self) -> Option<Arc<dyn AsyncFinalHandler<C>>> {
        self.handler.clone()
    }

    /// Returns the configured middleware list.
    pub fn middlewares(&self) -> &AsyncMiddlewareChain<C> {
        &self.middlewares
    }

    /// Returns whether no middleware is configured.
    pub fn has_no_middlewares(&self) -> bool {
        self.middlewares.is_empty()
    }

    /// Returns the configured middleware list mutably.
    pub fn middlewares_mut(&mut self) -> &mut AsyncMiddlewareChain<C> {
        &mut self.middlewares
    }

    pub fn has_handler(&self) -> bool {
        self.handler.is_some()
    }

    pub fn set_handler(&mut self, handler: Arc<dyn AsyncFinalHandler<C>>) {
        self.handler = Some(handler);
    }

    pub fn clear_handler(&mut self) {
        self.handler = None;
    }

    /// Replaces the configured middleware list.
    pub fn set_middlewares(&mut self, middlewares: AsyncMiddlewareChain<C>) {
        self.middlewares = middlewares;
    }

    /// Appends a middleware to the configured middleware list.
    pub fn append_middleware(&mut self, middleware: Arc<dyn AsyncMiddleware<C>>) {
        self.middlewares.push(middleware);
    }

    /// Compiles this binding into an executable chain if a handler exists.
    pub fn compile(&self) -> Option<ExecutionChain<C>>
    where
        C: Send + 'static,
    {
        self.execution_chain()
    }

    /// Consumes this binding and compiles it into an executable chain.
    pub fn into_chain(self) -> Result<ExecutionChain<C>, &'static str>
    where
        C: Send + 'static,
    {
        self.try_into()
    }

    pub fn execution_chain(&self) -> Option<ExecutionChain<C>> {
        self.handler
            .as_ref()
            .cloned()
            .map(|handler| ExecutionChain::new(self.middlewares.clone(), handler))
    }
}

/// The execution-chain builder and executor.
pub struct ExecutionChain<C> 
where 
    C: RequestContext + Send + 'static {
    inner: Arc<dyn Fn(C) -> BoxFuture<C> + Send + Sync + 'static>,
}

impl<C> Clone for ExecutionChain<C> 
where 
    C: RequestContext + Send + 'static { 
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<C> ExecutionChain<C>
where
    C: RequestContext + Send + 'static,
{
    /// Build a chain from middleware definitions and a final handler.
    pub fn new(
        middlewares: Vec<Arc<dyn AsyncMiddleware<C>>>,
        final_handler: Arc<dyn AsyncFinalHandler<C>>,
    ) -> Self {
        let final_fn: Arc<dyn Fn(C) -> BoxFuture<C> + Send + Sync + 'static> =
            Arc::new(move |ctx| final_handler.handle(ctx));

        let chain = middlewares.into_iter().rev().fold(final_fn, |next, mw| {
            let next_clone = next.clone();
            Arc::new(move |ctx: C| {
                let next_fn = next_clone.clone();
                mw.handle(ctx, Box::new(move |r| next_fn(r)))
            }) as Arc<dyn Fn(C) -> BoxFuture<C> + Send + Sync + 'static>
        });

        Self { inner: chain }
    }

    /// Drive the chain to completion, returning the final context.
    pub async fn run(&self, ctx: C) -> Result<C, <C as RequestContext>::Error> {
        (self.inner)(ctx).await
    } 
}

impl<C> TryFrom<ExecutableBinding<C>> for ExecutionChain<C>
where
    C: RequestContext + Send + 'static,
{
    type Error = &'static str;

    fn try_from(binding: ExecutableBinding<C>) -> Result<Self, Self::Error> {
        binding
            .handler
            .map(|handler| Self::new(binding.middlewares, handler))
            .ok_or("ExecutableBinding has no final handler")
    }
}

/// A helper that builds and runs an execution chain in one call.
pub async fn run_chain<C: RequestContext + 'static>(
    middlewares: AsyncMiddlewareChain<C>,
    final_handler: Arc<dyn AsyncFinalHandler<C>>,
    ctx: C,
) -> Result<C, <C as RequestContext>::Error> {
    let chain = ExecutionChain::new(middlewares, final_handler);
    chain.run(ctx).await
} 
