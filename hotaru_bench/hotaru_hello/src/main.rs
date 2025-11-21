use hotaru::prelude::*;
use hotaru::http::*;
use akari::object;

pub static APP: SApp = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:8001")
        .build()
});

#[tokio::main]
async fn main() {
    println!("Starting Hotaru server on 127.0.0.1:8001");
    APP.clone().run().await;
}

endpoint! {
    APP.url("/"),
    
    pub hello <HTTP> {
        text_response("Hello, World!")
    }
}

endpoint! {
    APP.url("/json"),
    
    pub hello_json <HTTP> {
        json_response(object!({
            message: "Hello, World!",
            framework: "Hotaru"
        }))
    }
}
