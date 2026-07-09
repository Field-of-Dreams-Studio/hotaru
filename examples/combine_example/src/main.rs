//! App-level `combine()`: two independently assembled server blueprints are
//! merged into one app before serving. Left side wins every conflict.
//!
//! Combining is not possible for `LServer!`/`endpoint!` statics: the static
//! holds its own `Arc` handle forever, so `try_combine` (which needs sole
//! ownership) always refuses. Blueprints that are meant to be combined must
//! be built and routed with the plain function API, as done here.
//!
//! Run `cargo run -p combine_example`, then:
//!   curl http://127.0.0.1:3005/hello   -> served by blueprint A (collision: A wins)
//!   curl http://127.0.0.1:3005/world   -> served by blueprint B (adopted subtree)

use std::future::Future;

use hotaru::http::*;
use hotaru::hotaru_core::executable::ExecutableBinding;
use hotaru::hotaru_http::HttpError;
use hotaru::prelude::*;

fn main() {
    let a = blueprint_a();
    let b = blueprint_b();

    // While another handle to A exists, the merge is refused and both apps
    // come back untouched inside the error.
    let held = a.clone();
    let (a, b) = match a.try_combine(b) {
        Ok(_) => unreachable!("A is still shared"),
        Err(AppInUse { app, other }) => {
            println!("try_combine refused while a handle to A is held");
            (app, other)
        }
    };
    drop(held);

    // Both blueprints register `/hello`; A's handler and binding survive.
    let app = a.try_combine(b).expect("no extra handles held now");
    println!("blueprints combined; serving on 127.0.0.1:3005");
    run_server!(app);
}

/// Blueprint A — the "main" app: real binding, its own `/hello`.
fn blueprint_a() -> Arc<Server> {
    let app = Server::new()
        .binding("127.0.0.1:3005")
        .single_protocol(ProtocolBuilder::new(HTTP::server(HttpSafety::default())))
        .build();
    app.url::<HTTP, _, _>("/hello", "hello_a", handler(hello_a), ParamsClone::default())
        .unwrap();
    app
}

/// Blueprint B — a feature module: `/world` is adopted; its `/hello` and
/// binding lose to A's.
fn blueprint_b() -> Arc<Server> {
    let app = Server::new()
        .binding("127.0.0.1:9999")
        .single_protocol(ProtocolBuilder::new(HTTP::server(HttpSafety::default())))
        .build();
    app.url::<HTTP, _, _>("/hello", "hello_b", handler(hello_b), ParamsClone::default())
        .unwrap();
    app.url::<HTTP, _, _>("/world", "world", handler(world), ParamsClone::default())
        .unwrap();
    app
}

async fn hello_a(mut ctx: HttpReqCtx) -> Result<HttpReqCtx, HttpError> {
    normal_response(200u16, "hello from blueprint A (left wins)\n").apply_to(&mut ctx)?;
    Ok(ctx)
}

async fn hello_b(mut ctx: HttpReqCtx) -> Result<HttpReqCtx, HttpError> {
    normal_response(200u16, "hello from blueprint B (should never be served)\n")
        .apply_to(&mut ctx)?;
    Ok(ctx)
}

async fn world(mut ctx: HttpReqCtx) -> Result<HttpReqCtx, HttpError> {
    normal_response(200u16, "world from blueprint B (adopted subtree)\n").apply_to(&mut ctx)?;
    Ok(ctx)
}

/// Wraps a plain `async fn` into the binding shape `url` expects.
fn handler<F, Fut>(f: F) -> ExecutableBinding<HttpReqCtx>
where
    F: Fn(HttpReqCtx) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<HttpReqCtx, HttpError>> + Send + 'static,
{
    ExecutableBinding::new().with_handler(Arc::new(f))
}
