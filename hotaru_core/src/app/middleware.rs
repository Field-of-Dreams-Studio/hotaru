use std::pin::Pin; 
use std::future::Future;
use std::sync::Arc; 
use crate::http::context::HttpReqCtx;

use crate::{debug_log, connection::RequestContext}; 
use std::any::Any; 

/// A boxed future returning `C`.
pub type BoxFuture<C> = Pin<Box<dyn Future<Output = C> + Send + 'static>>; 

pub type AsyncMiddlewareChain<C> = Vec<Arc<dyn AsyncMiddleware<C>>>; 

pub trait AsyncMiddleware<C: RequestContext>: Send + Sync + 'static { 
    fn as_any(&self) -> &dyn Any; 

    /// Used when creating the mddleware 
    fn return_self() -> Self where Self: Sized; 

    fn handle<'a>( 
        &self,
        rc: C,
        next: Box<dyn Fn(C) -> Pin<Box<dyn Future<Output = C> + Send>> + Send + Sync + 'static>,
    ) -> Pin<Box<dyn Future<Output = C> + Send + 'static>>; 
} 

/// The “final handler” trait that sits at the end of a middleware chain.
pub trait AsyncFinalHandler<C>: Send + Sync + 'static {
    /// Consume the request‐context and return a future yielding the (possibly modified) context.
    fn handle(&self, ctx: C) -> BoxFuture<C>;
} 

/// Blanket impl: any async fn or closure `Fn(R) -> impl Future<Output=R>` becomes an AsyncFinalHandler<R>.
impl<F, Fut, C> AsyncFinalHandler<C> for F
where
    F: Fn(C) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = C> + Send + 'static,
{
    fn handle(&self, ctx: C) -> BoxFuture<C> {
        Box::pin((self)(ctx))
    }
} 

/// The middleware‐chain builder and executor.
pub struct MiddlewareChain<C> {
    inner: Arc<dyn Fn(C) -> BoxFuture<C> + Send + Sync + 'static>,
}

impl<C> MiddlewareChain<C>
where
    C: RequestContext + Send + 'static,
{
    /// Build a chain from:
    ///  - `middlewares`: the Vec of AsyncMiddleware<R> in the order you want them to run
    ///  - `final_handler`: the AsyncFinalHandler<R> that executes last
    pub fn new(
        middlewares: Vec<Arc<dyn AsyncMiddleware<C>>>,
        final_handler: Arc<dyn AsyncFinalHandler<C>>,
    ) -> Self {
        // Wrap the final handler in a Fn(R)->Future
        let final_fn: Arc<dyn Fn(C) -> BoxFuture<C> + Send + Sync + 'static> =
            Arc::new(move |ctx| final_handler.handle(ctx));

        // Fold the middlewares in reverse so that the first element runs first
        let chain = middlewares.into_iter().rev().fold(final_fn, |next, mw| {
            let next_clone = next.clone();
            Arc::new(move |ctx: C| {
                // Each middleware calls the `next_fn` when ready to proceed
                let next_fn = next_clone.clone();
                mw.handle(ctx, Box::new(move |r| next_fn(r)))
            }) as Arc<dyn Fn(C) -> BoxFuture<C> + Send + Sync + 'static>
        });

        MiddlewareChain { inner: chain }
    }

    /// Drive the chain to completion, returning the final context.
    pub async fn run(&self, ctx: C) -> C {
        (self.inner)(ctx).await
    }
} 

/// A helper that builds and runs a middleware chain in one call.
pub async fn run_chain<C: RequestContext + 'static>(
    middlewares: Vec<Arc<dyn AsyncMiddleware<C>>>,
    final_handler: Arc<dyn AsyncFinalHandler<C>>,
    ctx: C,
) -> C {
    let chain = MiddlewareChain::new(middlewares, final_handler);
    chain.run(ctx).await
} 

pub struct LoggingMiddleware;

impl AsyncMiddleware<HttpReqCtx> for LoggingMiddleware {
    fn handle<'a>(
        &'a self,
        mut req: HttpReqCtx, 
        next: Box<dyn Fn(HttpReqCtx) -> Pin<Box<dyn Future<Output = HttpReqCtx> + Send>> + Send + Sync + 'static>,
    ) -> Pin<Box<dyn Future<Output = HttpReqCtx> + Send + 'static>> {
        Box::pin(async move {
            print!("[Request Received] Method: "); 
            print!("{}, ", req.method()); 
            print!("Path: "); 
            debug_log!("{}, ", req.path()); 
            if req.meta().get_host() == None { 
                req.response = crate::http::response::response_templates::normal_response(400, "").content_type(crate::http::http_value::HttpContentType::TextPlain());  
                debug_log!("[Bad Request] Missing Host Header"); 
                return req; 
            }
            req = next(req).await; 
            print!("[Request Processed] Method: "); 
            print!("{}, ", req.method()); 
            print!("Path: "); 
            print!("{}, ", req.path()); 
            print!("Status Code: "); 
            debug_log!("{}, ", req.response.meta.start_line.status_code()); 
            req 
        }) 
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    } 

    fn return_self() -> Self {
        LoggingMiddleware
    } 
} 
