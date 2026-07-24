//! Manual route registration using only Hotaru's ordinary Rust APIs.
//!
//! This crate does not depend on `hotaru_trans` and uses no Hotaru
//! procedural macros. Run it with:
//!
//! ```text
//! cargo run -p api_registration_example
//! curl http://127.0.0.1:3010/
//! curl http://127.0.0.1:3010/hello/Hotaru
//! curl http://127.0.0.1:3010/health
//! ```

use hotaru_core::{
    app::server::{Server, run_server},
    executable::{
        ProtocolEntryBuilder,
        def::{Endpoint, UrlMode},
    },
    marker::MaybeSendBoxFuture,
    prelude::Box,
    protocol::EndpointOutcome,
};
use hotaru_http::{
    DefaultHttpTransport, HTTP, context::HttpReqCtx, response::response_templates::normal_response,
    safety::HttpSafety,
};
use hotaru_rt_tokio::TokioRuntime;

fn main() {
    let app = Server::<DefaultHttpTransport, TokioRuntime>::new()
        .binding("127.0.0.1:3010")
        .single_protocol(ProtocolEntryBuilder::new(HTTP::server(
            HttpSafety::default(),
        )))
        .build();

    app.insert(index_route()).expect("index route must bind");
    app.extend([hello_route(), health_route()])
        .expect("API routes must bind");

    println!("API-only server listening on http://127.0.0.1:3010");
    run_server(app);
}

fn index_route() -> Endpoint<HTTP> {
    Endpoint::endpoint("/", "index", index_handler)
}

fn hello_route() -> Endpoint<HTTP> {
    Endpoint::endpoint("/hello/<str:name>", "hello", hello_handler)
}

fn health_route() -> Endpoint<HTTP> {
    Endpoint::endpoint("/health", "health", health_handler).with_url_mode(UrlMode::Literal)
}

fn index_handler(
    _context: &mut HttpReqCtx,
) -> MaybeSendBoxFuture<'_, impl EndpointOutcome<HttpReqCtx> + 'static + use<>> {
    Box::pin(async {
        normal_response(
            200u16,
            "Routes were registered through Endpoint::endpoint and App::insert.\n",
        )
    })
}

fn hello_handler(
    context: &mut HttpReqCtx,
) -> MaybeSendBoxFuture<'_, impl EndpointOutcome<HttpReqCtx> + 'static + use<>> {
    let name = context
        .pattern("name")
        .unwrap_or_else(|| "world".to_string());
    Box::pin(async move { normal_response(200u16, format!("Hello, {name}!\n")) })
}

fn health_handler(
    _context: &mut HttpReqCtx,
) -> MaybeSendBoxFuture<'_, impl EndpointOutcome<HttpReqCtx> + 'static + use<>> {
    Box::pin(async { normal_response(200u16, "ok\n") })
}
