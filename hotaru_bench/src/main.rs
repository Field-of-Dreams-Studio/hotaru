use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Message {
    message: String,
}

// Hotaru implementation
#[cfg(feature = "hotaru_server")]
mod hotaru_impl {
    use hotaru::prelude::*;
    use hotaru::http::*;

    pub static APP: SApp = Lazy::new(|| {
        App::new()
            .binding("0.0.0.0:8080")
            .build()
    });

    endpoint! {
        APP.url("/json"),
        pub json_endpoint<HTTP> {
            json_response(object!({
                message: "Hello, World!"
            }))
        }
    }

    endpoint! {
        APP.url("/plaintext"),
        pub plaintext_endpoint<HTTP> {
            text_response("Hello, World!")
        }
    }

    pub async fn run() {
        println!("🔥 Hotaru server running on http://0.0.0.0:8080");
        let _ = APP.clone().run().await;
    }
}

// Actix-web implementation
#[cfg(feature = "actix_server")]
mod actix_impl {
    use super::*;
    use actix_web::{web, App, HttpResponse, HttpServer, Responder};

    async fn json_handler() -> impl Responder {
        let msg = Message {
            message: "Hello, World!".to_string(),
        };
        HttpResponse::Ok()
            .content_type("application/json")
            .json(msg)
    }

    async fn plaintext_handler() -> impl Responder {
        HttpResponse::Ok()
            .content_type("text/plain")
            .body("Hello, World!")
    }

    pub async fn run() -> std::io::Result<()> {
        println!("⚡ Actix-web server running on http://0.0.0.0:8080");
        HttpServer::new(|| {
            App::new()
                .route("/json", web::get().to(json_handler))
                .route("/plaintext", web::get().to(plaintext_handler))
        })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
    }
}

// Axum implementation
#[cfg(feature = "axum_server")]
mod axum_impl {
    use super::*;
    use axum::{
        response::{IntoResponse, Response},
        routing::get,
        Json, Router,
    };

    async fn json_handler() -> Json<Message> {
        Json(Message {
            message: "Hello, World!".to_string(),
        })
    }

    async fn plaintext_handler() -> Response {
        ([("content-type", "text/plain")], "Hello, World!").into_response()
    }

    pub async fn run() {
        println!("🚀 Axum server running on http://0.0.0.0:8080");
        let app = Router::new()
            .route("/json", get(json_handler))
            .route("/plaintext", get(plaintext_handler));

        let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
            .await
            .unwrap();
        axum::serve(listener, app).await.unwrap();
    }
}

// Rocket implementation
#[cfg(feature = "rocket_server")]
mod rocket_impl {
    use super::*;
    use rocket::{get, routes, serde::json::Json, Config};
    use std::net::Ipv4Addr;

    #[get("/json")]
    fn json_handler() -> Json<Message> {
        Json(Message {
            message: "Hello, World!".to_string(),
        })
    }

    #[get("/plaintext")]
    fn plaintext_handler() -> (rocket::http::ContentType, &'static str) {
        (rocket::http::ContentType::Plain, "Hello, World!")
    }

    pub async fn run() -> Result<(), rocket::Error> {
        println!("🚀 Rocket server running on http://0.0.0.0:8080");
        let config = Config {
            port: 8080,
            address: Ipv4Addr::new(0, 0, 0, 0).into(),
            ..Config::default()
        };

        let _ = rocket::custom(&config)
            .mount("/", routes![json_handler, plaintext_handler])
            .launch()
            .await?;

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    #[cfg(feature = "hotaru_server")]
    {
        hotaru_impl::run().await;
    }

    #[cfg(feature = "actix_server")]
    {
        actix_impl::run().await.unwrap();
    }

    #[cfg(feature = "axum_server")]
    {
        axum_impl::run().await;
    }

    #[cfg(feature = "rocket_server")]
    {
        rocket_impl::run().await.unwrap();
    }

    #[cfg(not(any(
        feature = "hotaru_server",
        feature = "actix_server",
        feature = "axum_server",
        feature = "rocket_server"
    )))]
    {
        eprintln!("❌ No server feature enabled!");
        eprintln!("Please run with one of:");
        eprintln!("  cargo run --features hotaru_server");
        eprintln!("  cargo run --features actix_server");
        eprintln!("  cargo run --features axum_server");
        eprintln!("  cargo run --features rocket_server");
        std::process::exit(1);
    }
}
