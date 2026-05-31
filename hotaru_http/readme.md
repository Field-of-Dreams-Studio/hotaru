# hotaru_http

HTTP/1.1 implementation for the [Hotaru](../hotaru) web framework — context type, channel, request/response model, error mapping, and the `Http1Protocol` impl that bridges Hotaru's `Protocol` trait to HTTP/1.1 over any `ConnStream` transport.

Most users should depend on the umbrella `hotaru` crate; this crate is the seam where HTTP-specific code lives so future protocols (HTTP/2, etc.) can sit beside it.

## Features

- `tls` — pulls in [`hotaru_tls`](../hotaru_tls) and exposes `HTTPS = Http1Protocol<TlsStream, TlsTransport>` plus the TLS transport/config re-exports.

## Layout

- `protocol/` — `Http1Protocol`, `HttpError`, helpers (keep-alive, error responses).
- `channel/` — `HttpChannel` trait + `Http1Channel<W>` (per-exchange wire wrapper).
- `context/` — `HttpContext<TS>` (the `RequestContext` impl).
- `message/` — `HttpRequest`, `HttpResponse`, `HttpBody`, `HttpMeta`, `HttpStartLine`, types.
- `security/` — `HttpSafety` (size/limit knobs).
- `util/` — cookies, encoding, form parsing.

## Version

`0.8.2`. Depends on `hotaru_core = 0.8.2`, `hotaru_lib = 0.8.2`.
