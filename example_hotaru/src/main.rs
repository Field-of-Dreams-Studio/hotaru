use hotaru::http::*;
use hotaru::prelude::*;

#[tokio::main]
async fn main() {
    APP.clone().run().await;
}

LServer!(
    APP = Server::new()
        .binding("127.0.0.1:3003")
        .single_protocol(ProtocolBuilder::new(HTTP::server(HttpSafety::default())))
        .build()
);

// HTTPS-targeted client for example.com:443. `TlsClientConfig::default()`
// uses the bundled webpki root store for cert verification.
//
// `SClient` defaults TS to TcpTransport, so we bypass `LClient!` and declare
// the static directly to pin TS=TlsTransport here.
pub static CLIENT: Lazy<Arc<Client<TlsTransport>>> = Lazy::new(|| {
    Client::<TlsTransport>::new()
        .target(TlsOutboundTarget::new(
            "example.com",
            443,
            TlsClientConfig::default(),
        ))
        .single_protocol(ProtocolBuilder::new(HTTPS::client(HttpSafety::default())))
        .build()
});


// Trans

endpoint! {
    APP.url("/"),
    middleware = [LoggerMiddleware],

    index <HTTP> {
        akari_render!(
            "home.html",
            title = "Hotaru Example",
            page_title = "Welcome to Hotaru 0.8!",
            show_message = true,
            message = "Framework successfully running!",
            items = [
                "Protocol Abstraction Layer",
                "Async/await support",
                "Middleware system",
                "Template rendering"
            ]
        )
    }
}

// Proxy endpoint: fires an HTTPS outpoint to example.com, then renders the
// fetched body inline as the response. The body is extracted from whichever
// HttpBody variant the response landed in.
endpoint! {
    APP.url("/example_outpoint_fetch"),
    middleware = [LoggerMiddleware], 

    example_outpoint_fetch <HTTP> {
        let mut outbound = HttpRequest::default();
        outbound.meta.set_host(Some("example.com".to_string()));

        match run!(CLIENT<HTTPS>::ping_example, outbound).await {
            Ok(Ok(resp)) => {
                let body_bytes: Vec<u8> = match resp.body {
                    HttpBody::Text(s) => s.into_bytes(),
                    HttpBody::Binary(b) => b,
                    HttpBody::Buffer { data, .. } => data,
                    _ => Vec::new(),
                };
                let body_str = String::from_utf8_lossy(&body_bytes);
                let html = format!(
                    "<!doctype html><html><body>\
                     <h1>Fetched from https://example.com/</h1>\
                     <hr/>{body_str}</body></html>"
                );
                response_templates::html_response(html.into_bytes())
            }
            Ok(Err(e)) => response_templates::normal_response(
                502u16,
                format!("upstream error: {e}"),
            ),
            Err(e) => response_templates::normal_response(
                500u16,
                format!("lookup error: {e}"),
            ),
        }
    }
}

// Client-side HTTPS outpoint. Host header drives SNI / certificate verify and
// virtual hosting; user middleware fills it in if the caller didn't.
outpoint! {
    CLIENT.url("/"),

    ping_example <HTTPS> {
        if req.request.meta.get_host().is_none() {
            req.request.meta.set_host(Some("example.com".to_string()));
        }
        send;
        Ok(req)
    }
}

middleware! {
    LoggerMiddleware <HTTP> {
        println!("Request received: {} {}", req.method(), req.path());
        next(req).await
    }
}

// Semi-trans
// #[endpoint]
// #[url("/")]
// #[middleware([logger_middleware])]
// fn index <HTTP>() {
//     akari_render!(
//         "home.html",
//         title = "Hotaru Example",
//         page_title = "Welcome to Hotaru 0.8",
//         show_message = true,
//         message = "Framework successfully running!",
//         items = [
//             "Protocol Abstraction Layer",
//             "Async/await support",
//             "Middleware system",
//             "Template rendering"
//         ]
//     )
// }

// #[middleware]
// fn logger_middleware <HTTP>(context: CustomParam) {
//     println!("Request received: {} {}", context.method(), context.path());
//     next(context).await
// }

// Attr

// #[endpoint("/", middleware = [logger_middleware])]
// fn index<HTTP>() {
//     akari_render!(
//         "home.html",
//         title = "Hotaru Example",
//         page_title = "Welcome to Hotaru 0.8",
//         show_message = true,
//         message = "Framework successfully running!",
//         items = [
//             "Protocol Abstraction Layer",
//             "Async/await support",
//             "Middleware system",
//             "Template rendering"
//         ]
//     )
// }

// // #[endpoint("/<i:>")]
// // fn index_alt <HTTP>() {
// //     akari_json!({
// //         message: "This endpoint have an url formatting error! Trans crate will report an error!"
// //     })
// // }

// #[middleware]
// fn logger_middleware<HTTP>(context: CustomParam) {
//     println!("Request received: {} {}", context.method(), context.path());
//     next(context).await
// }

// mod resource;
