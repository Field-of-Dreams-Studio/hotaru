//! Compile-check the `middleware!` and `mw_chain!` expansions together.

use hotaru_core::executable::def::MWChain;
use hotaru_http::{HTTP, context::HttpReqCtx};
use hotaru_trans::{middleware, mw_chain};

middleware! {
    ExampleMiddleware <HTTP> {
        next(req).await
    }
}

fn main() {
    let _empty: MWChain<HttpReqCtx> = mw_chain!([]);
    let _chain: MWChain<HttpReqCtx> = mw_chain!([ExampleMiddleware, ..,]);
}
