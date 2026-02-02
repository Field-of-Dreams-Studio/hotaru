use hotaru::prelude::*;
use hotaru::http::*;
use hotaru::{ConnectionBuilder, LegacyConnectionBuilder, LegacyProtocol};

#[tokio::main]
async fn main() {
    connection_builder_examples();

    if std::env::var("HOTARU_CLIENT_EXAMPLE").is_ok() {
        run_client_example().await;
        return;
    }

    APP.clone().run().await;
}

#[allow(deprecated)]
fn connection_builder_examples() {
    let _new_builder = ConnectionBuilder::<HTTP>::new("example.com")
        .tls(true)
        .port(443);

    let _legacy_builder = LegacyConnectionBuilder::new("example.com", 0)
        .protocol(LegacyProtocol::HTTP)
        .tls(true);
}

LApp!(APP = App::new().build()); 
LClient!(API_CLIENT = Client::new()
    .name("jsonplaceholder")
    .base_url("https://jsonplaceholder.typicode.com")
    .build()
);

// Trans 

// endpoint! { 
//     APP.url("/"), 
//     middleware = [LoggerMiddleware] 
    
//     index <HTTP> {
//         akari_render!(
//             "home.html",
//             title = "Hotaru Example",
//             page_title = "Welcome to Hotaru 0.8",
//             show_message = true,
//             message = "Framework successfully running!",
//             items = [
//                 "Protocol Abstraction Layer",
//                 "Async/await support", 
//                 "Middleware system",
//                 "Template rendering"
//             ]
//         )
//     } 
// } 

// middleware! { 
//     LoggerMiddleware <HTTP> {
//         println!("Request received: {} {}", req.method(), req.path());
//         next(req).await
//     }  
// } 

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

#[endpoint("/", middleware = [logger_middleware])]
fn index <HTTP>() { 
    akari_render!(
        "home.html",
        title = "Hotaru Example",
        page_title = "Welcome to Hotaru 0.8",
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

// #[endpoint("/<i:>")] 
// fn index_alt <HTTP>() { 
//     akari_json!({
//         message: "This endpoint have an url formatting error! Trans crate will report an error!" 
//     }) 
// } 

#[middleware] 
fn logger_middleware <HTTP>(context: CustomParam) {
    println!("Request received: {} {}", context.method(), context.path());
    next(context).await
} 

#[outpoint(API_CLIENT.url("/users/<id>"))]
fn get_user <HTTP>() {
    req.request.meta.start_line.set_method(HttpMethod::GET);
    let url = match req.build_url() {
        Ok(url) => url,
        Err(_) => return HttpResponse::default(),
    };
    if req.send(&url).await.is_err() {
        return HttpResponse::default();
    }
    req.response.clone()
}

async fn run_client_example() {
    let entry = ClientRegistry::global()
        .get::<HttpContext>("jsonplaceholder", "get_user")
        .expect("outpoint not registered");

    let mut ctx = HttpContext::new_client_with_context(API_CLIENT.clone())
        .with_url_patterns(entry.patterns.clone(), entry.names.clone())
        .with_param("id", "1");

    let response = get_user(&mut ctx).await;
    println!(
        "Status: {}",
        response.meta.start_line.status_code().as_u16()
    );
    println!("Body: {:?}", response.body);
}

// mod resource;
