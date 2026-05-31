# hotaru_trans

Proc-macro crate for the [Hotaru](../hotaru) web framework. Provides the macros users write at registration sites and call sites — all hand-rolled `proc_macro::TokenStream` (no `syn`/`quote`).

## Macros

**Registration (annotate definitions):**
- `endpoint!` — register an inbound URL handler on a `Server`.
- `outpoint!` — register an outbound URL handler on a `Client`. The user body is rewritten so `send;` triggers the inner middleware chain; the chain terminates in `<P as Protocol>::send(ctx).await`.
- `middleware!` — define an `AsyncMiddleware<C>` impl from a body that uses `req` + `next(req).await`.

**Invocation (use at call sites):**
- `run!(APP<P>::name, request)` → `APP.request_fn::<P>("name", request)` — one-shot outpoint request.
- `call!(APP<P>::name)` → `APP.call_fn::<P>("name")` — spawn persistent outpoint loop. Also accepts `call!(APP<P>: "/path")` for the URL form.

**Lazy statics (one-line declarations):**
- `LServer!(APP = Server::new()...build())` — `pub static APP: SServer = Lazy::new(...);`.
- `LClient!(CLIENT = Client::new()...build())` — `pub static CLIENT: SClient = Lazy::new(...);`.
- `LUrl!`, `LPattern!` — same idea for URL/pattern statics.

**Constructor attribute:**
- `#[ctor]` — built-in equivalent of the `ctor` crate. Used internally by registration macros so URL bindings install at program start. Use the `external-ctor` feature on `hotaru` to swap in the external `ctor` crate instead.

## Feature flags

- `trans` (default) — function-style macros (`endpoint!`/`outpoint!`/`middleware!`).
- `attr` — attribute-style: `#[endpoint("/path")]` over a free `fn`.
- `semi-trans` — attribute-on-block hybrid.
- `external-ctor` — opt out of the built-in `#[ctor]` and use the external `ctor = "0.4"` crate.

## Layout

- `url/` — `endpoint!` / `outpoint!` parsing + codegen (`UrlArgs`, `UrlFunc`, `UrlExpr`, `send;` rewriter, `reg_func(UrlKind::{Endpoint, Outpoint})`).
- `middleware.rs` — `middleware!` codegen (`MWFunc`; emits a struct + `AsyncMiddleware<C>` impl wrapped in `const _: () = { ... };` for scope isolation).
- `call.rs` — `run!` and `call!` parsers + emitters.
- `ctor.rs` — built-in constructor attribute.
- `helper.rs`, `outer_attr.rs` — shared parser helpers.

## Version

`0.8.2`. Depends on `hotaru_core = 0.8.2`, `hotaru_lib = 0.8.2`.
