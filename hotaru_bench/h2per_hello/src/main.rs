use hotaru::prelude::*;
use h2per::prelude::*;
use serde_json::json;

// Create the app with Hyper protocol (HTTP/1.1)
pub static APP: SApp = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:8003")
        .handle(
            HandlerBuilder::new()
                .protocol(ProtocolBuilder::new(HYPER1::new(ProtocolRole::Server)))
        )
        .build()
});

#[tokio::main]
async fn main() {
    println!("Starting H2per (Hyper-based) server on 127.0.0.1:8003");
    APP.clone().run().await;
}

endpoint! {
    APP.url("/"),
    
    pub hello <HYPER1> {
        text_response("Hello, World!")
    }
}

endpoint! {
    APP.url("/json"),
    
    pub hello_json <HYPER1> {
        json_response(json!({
            "message": "Hello, World!",
            "framework": "H2per"
        }))
    }
}