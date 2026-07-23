//! Focused compile fixture for the Stage 8.2 middleware emitter.
//! It checks visibility, local expansion, and trait-object erasure.
//! The example validates generated code only; it does not run a server.

use hotaru_core::{executable::middleware::AsyncMiddleware, marker::Arc};
use hotaru_http::{context::HttpReqCtx, HTTP};
use hotaru_trans::middleware;

pub mod public_boundary {
    use super::*;

    middleware! {
        pub PublicMiddleware <HTTP> {
            next(req).await
        }
    }

}

mod public_consumer {
    use super::*;

    pub fn erased() -> Arc<dyn AsyncMiddleware<HttpReqCtx>> {
        Arc::new(public_boundary::PublicMiddleware)
    }
}

fn private_local() -> Arc<dyn AsyncMiddleware<HttpReqCtx>> {
    middleware! {
        LocalMiddleware <HTTP> {
            next(req).await
        }
    }

    Arc::new(LocalMiddleware)
}

fn forwarding_and_error_shape() -> Arc<dyn AsyncMiddleware<HttpReqCtx>> {
    middleware! {
        ForwardingMiddleware <HTTP> {
            next(req).await
        }
    }

    Arc::new(ForwardingMiddleware)
}

fn main() {
    // Public middleware remains nameable after crossing a module boundary.
    let _public = public_consumer::erased();

    // A private middleware can still be expanded and used in its local scope.
    let _private = private_local();

    // The generated forwarding body matches the erased middleware ABI.
    let _forwarding = forwarding_and_error_shape();
}

#[cfg(test)]
mod tests {
    #[test]
    fn generated_middleware_is_constructible_and_erasable() {
        let _ = super::public_consumer::erased();
        let _ = super::private_local();
        let _ = super::forwarding_and_error_shape();
    }
}
