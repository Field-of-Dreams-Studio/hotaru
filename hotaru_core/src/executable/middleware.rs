use core::future::Future;
use core::pin::Pin;
use alloc::sync::Arc;

// use crate::debug_log;

use crate::protocol::RequestContext;
use core::any::Any;

/// A boxed future returning `C`.
pub type BoxFuture<C> = Pin<Box<
    dyn Future<Output = Result<C, <C as RequestContext>::Error>> + Send + 'static 
>>;

pub type AsyncMiddlewareChain<C> = Vec<Arc<dyn AsyncMiddleware<C>>>;

pub trait AsyncMiddleware<C: RequestContext>: Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;

    /// Used when creating the mddleware
    fn return_self() -> Self
    where
        Self: Sized;

    fn handle<'a>(
        &self,
        rc: C,
        next: Box<
            dyn Fn(C) -> Pin<Box<
                dyn Future<Output = Result<C, <C as RequestContext>::Error>> + Send
            >> + Send + Sync + 'static,
        >,
    ) -> Pin<Box<
        dyn Future<Output = Result<C, <C as RequestContext>::Error>> + Send + 'static
    >>; 
}

/// The “final handler” trait that sits at the end of a middleware chain.
pub trait AsyncFinalHandler<C: RequestContext>: Send + Sync + 'static {
    /// Consume the request‐context and return a future yielding the (possibly modified) context.
    fn handle(&self, ctx: C) -> BoxFuture<C>;
}

/// Blanket impl: any async fn or closure `Fn(R) -> impl Future<Output=R>` becomes an AsyncFinalHandler<R>.
impl<F, Fut, C> AsyncFinalHandler<C> for F
where
    C: RequestContext,
    F: Fn(C) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<C, <C as RequestContext>::Error>> + Send + 'static,
{
    fn handle(&self, ctx: C) -> BoxFuture<C> {
        Box::pin((self)(ctx))
    }
} 

// HTTP Implementation example (to be moved to hotaru_http crate later)
// pub struct LoggingMiddleware;

// impl AsyncMiddleware<HttpReqCtx> for LoggingMiddleware {
//     fn handle<'a>(
//         &'a self,
//         mut req: HttpReqCtx,
//         next: Box<dyn Fn(HttpReqCtx) -> Pin<Box<dyn Future<Output = HttpReqCtx> + Send>> + Send + Sync + 'static>,
//     ) -> Pin<Box<dyn Future<Output = HttpReqCtx> + Send + 'static>> {
//         Box::pin(async move {
//             print!("[Request Received] Method: ");
//             print!("{}, ", req.method());
//             print!("Path: ");
//             debug_log!("{}, ", req.path());
//             if req.meta().get_host() == None {
//                 req.response = crate::http::response::response_templates::normal_response(400, "").content_type(crate::http::http_value::HttpContentType::TextPlain());
//                 debug_log!("[Bad Request] Missing Host Header");
//                 return req;
//             }
//             req = next(req).await;
//             print!("[Request Processed] Method: ");
//             print!("{}, ", req.method());
//             print!("Path: ");
//             print!("{}, ", req.path());
//             print!("Status Code: ");
//             debug_log!("{}, ", req.response.meta.start_line.status_code());
//             req
//         })
//     }

//     fn as_any(&self) -> &dyn Any {
//         self
//     }

//     fn return_self() -> Self {
//         LoggingMiddleware
//     }
// }
