use hotaru::prelude::*;
use hotaru::http::*;

#[tokio::main]
async fn main() {
    APP.clone().run().await;
}

LApp!(APP = App::new().build()); 

endpoint! { 
    APP.url("/"), 
    middleware = [logger_middleware] 
    
    index <HTTP> {
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
} 

middleware! { 
    logger_middleware <HTTP> {
        println!("Request received: {} {}", req.method(), req.path());
        next(req).await
    }  
}

// #[endpoint] 
// #[url(APP.url("/"))]
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

mod resource;
