use actix_web::{web, App, HttpResponse, HttpServer, Result};
use serde_json::json;

async fn hello() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().body("Hello, World!"))
}

async fn hello_json() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "message": "Hello, World!",
        "framework": "Actix-web"
    })))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting Actix-web server on 127.0.0.1:8002");
    
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(hello))
            .route("/json", web::get().to(hello_json))
    })
    .bind("127.0.0.1:8002")?
    .run()
    .await
}
