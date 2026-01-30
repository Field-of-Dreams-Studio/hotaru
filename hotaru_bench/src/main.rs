use serde::{Deserialize, Serialize};

mod models;
mod utils;
#[cfg(any(
    feature = "hotaru_server",
    feature = "actix_server",
    feature = "axum_server",
    feature = "rocket_server"
))]
mod database;

#[derive(Serialize, Deserialize)]
struct Message {
    message: String,
}

// Hotaru implementation
#[cfg(feature = "hotaru_server")]
mod hotaru_impl {
    use super::{database, models::Fortune, models::World, utils, Message};
    use akari::Value;
    use hotaru::http::*;
    use hotaru::hotaru_core::http::start_line::HttpStartLine;
    use hotaru::prelude::*;
    use serde::Serialize;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::SystemTime;
    use database::{DbPool, WorldCache};

    #[derive(Clone)]
    struct FortuneView {
        id: i32,
        message: String,
    }

    impl From<World> for Value {
        fn from(world: World) -> Self {
            object!({
                id: world.id,
                randomNumber: world.random_number,
            })
        }
    }

    impl From<FortuneView> for Value {
        fn from(fortune: FortuneView) -> Self {
            object!({
                id: fortune.id,
                message: fortune.message,
            })
        }
    }

    pub static APP: SApp = Lazy::new(|| {
        let pool = database::create_pool();
        let cache = database::create_cache();
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        if let Err(err) = runtime.block_on(database::warm_cache(&pool, &cache)) {
            eprintln!("Failed to warm cache at startup: {err}");
        }

        App::new()
            .binding("0.0.0.0:8080")
            .set_statics("db_pool", pool)
            .set_statics("world_cache", cache)
            .build()
    });

    fn pool_from(req: &HttpContext) -> DbPool {
        req.app()
            .expect("Hotaru app not available")
            .statics()
            .get::<DbPool>("db_pool")
            .expect("Database pool missing")
            .clone()
    }

    fn cache_from(req: &HttpContext) -> WorldCache {
        req.app()
            .expect("Hotaru app not available")
            .statics()
            .get::<WorldCache>("world_cache")
            .expect("World cache missing")
            .clone()
    }

    fn append_fortune(mut fortunes: Vec<Fortune>) -> Vec<Fortune> {
        fortunes.push(Fortune {
            id: 0,
            message: "Additional fortune added at request time.".to_string(),
        });
        fortunes.sort_by(|a, b| a.message.cmp(&b.message));
        fortunes
    }

    fn with_standard_headers(response: HttpResponse) -> HttpResponse {
        let date = httpdate::fmt_http_date(SystemTime::now());
        response
            .add_header("Server", "hotaru")
            .add_header("Date", date)
    }

    fn json_response_direct<T: Serialize>(data: &T) -> HttpResponse {
        let json_bytes = serde_json::to_vec(data).expect("JSON serialization failed");
        let start_line = HttpStartLine::new_response(HttpVersion::Http11, StatusCode::OK);
        let mut meta = HttpMeta::new(start_line, HashMap::new());
        meta.set_content_type(HttpContentType::ApplicationJson());
        with_standard_headers(HttpResponse::new(meta, HttpBody::Binary(json_bytes)))
    }

    endpoint! {
        APP.url("/json"),
        pub json_endpoint<HTTP> {
            json_response_direct(&Message {
                message: "Hello, World!".to_string(),
            })
        }
    }

    endpoint! {
        APP.url("/plaintext"),
        pub plaintext_endpoint<HTTP> {
            with_standard_headers(text_response("Hello, World!"))
        }
    }

    endpoint! {
        APP.url("/db"),
        pub db_endpoint<HTTP> {
            let pool = pool_from(&req);
            let client = pool.get().await.expect("DB pool error");
            let id = utils::random_id();
            let world = database::fetch_world_by_id(&client, id)
                .await
                .expect("DB query failed");

            json_response_direct(&world)
        }
    }

    endpoint! {
        APP.url("/queries"),
        pub queries_endpoint<HTTP> {
            let count = utils::parse_query_count(req.query("queries").as_deref());
            let pool = pool_from(&req);
            let client = pool.get().await.expect("DB pool error");

            let mut worlds = Vec::with_capacity(count);
            for _ in 0..count {
                let id = utils::random_id();
                let world = database::fetch_world_by_id(&client, id)
                    .await
                    .expect("DB query failed");
                worlds.push(world);
            }

            json_response_direct(&worlds)
        }
    }

    endpoint! {
        APP.url("/updates"),
        pub updates_endpoint<HTTP> {
            let count = utils::parse_query_count(req.query("queries").as_deref());
            let pool = pool_from(&req);
            let client = pool.get().await.expect("DB pool error");

            let mut worlds = Vec::with_capacity(count);
            for _ in 0..count {
                let id = utils::random_id();
                let mut world = database::fetch_world_by_id(&client, id)
                    .await
                    .expect("DB query failed");
                world.random_number = utils::random_id();
                database::update_world(&client, &world)
                    .await
                    .expect("DB update failed");
                worlds.push(world);
            }

            json_response_direct(&worlds)
        }
    }

    endpoint! {
        APP.url("/cached-worlds"),
        pub cached_worlds_endpoint<HTTP> {
            let count = utils::parse_query_count(req.query("count").as_deref());
            let pool = pool_from(&req);
            let cache = cache_from(&req);
            let client = pool.get().await.expect("DB pool error");

            let mut worlds = Vec::with_capacity(count);
            for _ in 0..count {
                let id = utils::random_id();
                if let Some(world) = cache.get(&id) {
                    worlds.push(world.as_ref().clone());
                } else {
                    let world = database::fetch_cached_world_by_id(&client, id)
                        .await
                        .expect("DB query failed");
                    cache.insert(id, Arc::new(world.clone()));
                    worlds.push(world);
                }
            }

            json_response_direct(&worlds)
        }
    }

    endpoint! {
        APP.url("/fortunes"),
        pub fortunes_endpoint<HTTP> {
            let pool = pool_from(&req);
            let client = pool.get().await.expect("DB pool error");
            let fortunes = database::fetch_all_fortunes(&client)
                .await
                .expect("DB query failed");
            let fortunes = append_fortune(fortunes)
                .into_iter()
                .map(|fortune| FortuneView {
                    id: fortune.id,
                    message: utils::escape_html(&fortune.message),
                })
                .collect::<Vec<_>>();

            with_standard_headers(akari_render!("fortunes_hotaru.html", fortunes = fortunes))
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
    use super::{database, models::CountParams, models::Fortune, models::QueryParams, utils, Message};
    use actix_web::{web, App, HttpResponse, HttpServer, Responder};
    use askama::Template;
    use std::sync::Arc;

    use database::{DbPool, WorldCache};

    #[derive(Template)]
    #[template(path = "fortunes_askama.html")]
    struct FortunesTemplate {
        fortunes: Vec<Fortune>,
    }

    fn append_fortune(mut fortunes: Vec<Fortune>) -> Vec<Fortune> {
        fortunes.push(Fortune {
            id: 0,
            message: "Additional fortune added at request time.".to_string(),
        });
        fortunes.sort_by(|a, b| a.message.cmp(&b.message));
        fortunes
    }

    async fn json_handler() -> impl Responder {
        let msg = Message {
            message: "Hello, World!".to_string(),
        };
        HttpResponse::Ok().json(msg)
    }

    async fn plaintext_handler() -> impl Responder {
        HttpResponse::Ok()
            .content_type("text/plain")
            .body("Hello, World!")
    }

    async fn db_handler(pool: web::Data<DbPool>) -> impl Responder {
        let client = pool.get().await.expect("DB pool error");
        let world = database::fetch_world_by_id(&client, utils::random_id())
            .await
            .expect("DB query failed");
        HttpResponse::Ok().json(world)
    }

    async fn queries_handler(
        pool: web::Data<DbPool>,
        params: web::Query<QueryParams>,
    ) -> impl Responder {
        let count = utils::clamp_query_count(params.queries);
        let client = pool.get().await.expect("DB pool error");
        let mut worlds = Vec::with_capacity(count);
        for _ in 0..count {
            let world = database::fetch_world_by_id(&client, utils::random_id())
                .await
                .expect("DB query failed");
            worlds.push(world);
        }
        HttpResponse::Ok().json(worlds)
    }

    async fn updates_handler(
        pool: web::Data<DbPool>,
        params: web::Query<QueryParams>,
    ) -> impl Responder {
        let count = utils::clamp_query_count(params.queries);
        let client = pool.get().await.expect("DB pool error");
        let mut worlds = Vec::with_capacity(count);
        for _ in 0..count {
            let mut world = database::fetch_world_by_id(&client, utils::random_id())
                .await
                .expect("DB query failed");
            world.random_number = utils::random_id();
            database::update_world(&client, &world)
                .await
                .expect("DB update failed");
            worlds.push(world);
        }
        HttpResponse::Ok().json(worlds)
    }

    async fn cached_worlds_handler(
        pool: web::Data<DbPool>,
        cache: web::Data<WorldCache>,
        params: web::Query<CountParams>,
    ) -> impl Responder {
        let count = utils::clamp_query_count(params.count);
        let client = pool.get().await.expect("DB pool error");
        let mut worlds = Vec::with_capacity(count);
        for _ in 0..count {
            let id = utils::random_id();
            if let Some(world) = cache.get(&id) {
                worlds.push(world.as_ref().clone());
            } else {
                let world = database::fetch_cached_world_by_id(&client, id)
                    .await
                    .expect("DB query failed");
                cache.insert(id, Arc::new(world.clone()));
                worlds.push(world);
            }
        }
        HttpResponse::Ok().json(worlds)
    }

    async fn fortunes_handler(pool: web::Data<DbPool>) -> impl Responder {
        let client = pool.get().await.expect("DB pool error");
        let fortunes = database::fetch_all_fortunes(&client)
            .await
            .expect("DB query failed");
        let fortunes = append_fortune(fortunes);
        let template = FortunesTemplate { fortunes };
        let body = template.render().expect("Template render failed");
        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(body)
    }

    pub async fn run() -> std::io::Result<()> {
        println!("⚡ Actix-web server running on http://0.0.0.0:8080");
        let pool = database::create_pool();
        let cache = database::create_cache();
        if let Err(err) = database::warm_cache(&pool, &cache).await {
            eprintln!("Failed to warm cache at startup: {err}");
        }

        let pool_data = web::Data::new(pool);
        let cache_data = web::Data::new(cache);

        HttpServer::new(move || {
            App::new()
                .app_data(pool_data.clone())
                .app_data(cache_data.clone())
                .route("/json", web::get().to(json_handler))
                .route("/plaintext", web::get().to(plaintext_handler))
                .route("/db", web::get().to(db_handler))
                .route("/queries", web::get().to(queries_handler))
                .route("/updates", web::get().to(updates_handler))
                .route("/cached-worlds", web::get().to(cached_worlds_handler))
                .route("/fortunes", web::get().to(fortunes_handler))
        })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
    }
}

// Axum implementation
#[cfg(feature = "axum_server")]
mod axum_impl {
    use super::{database, models::CountParams, models::Fortune, models::QueryParams, models::World, utils, Message};
    use askama::Template;
    use axum::{
        extract::{Query, State},
        http::StatusCode,
        response::{IntoResponse, Response},
        routing::get,
        Json, Router,
    };
    use std::sync::Arc;

    use database::{DbPool, WorldCache};

    #[derive(Clone)]
    struct AppState {
        pool: DbPool,
        cache: WorldCache,
    }

    #[derive(Template)]
    #[template(path = "fortunes_askama.html")]
    struct FortunesTemplate {
        fortunes: Vec<Fortune>,
    }

    struct HtmlTemplate<T>(T);

    impl<T> IntoResponse for HtmlTemplate<T>
    where
        T: Template,
    {
        fn into_response(self) -> Response {
            match self.0.render() {
                Ok(html) => (
                    [("content-type", "text/html; charset=utf-8")],
                    html,
                )
                    .into_response(),
                Err(err) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Template error: {err}"),
                )
                    .into_response(),
            }
        }
    }

    fn append_fortune(mut fortunes: Vec<Fortune>) -> Vec<Fortune> {
        fortunes.push(Fortune {
            id: 0,
            message: "Additional fortune added at request time.".to_string(),
        });
        fortunes.sort_by(|a, b| a.message.cmp(&b.message));
        fortunes
    }

    async fn json_handler() -> Json<Message> {
        Json(Message {
            message: "Hello, World!".to_string(),
        })
    }

    async fn plaintext_handler() -> Response {
        ([("content-type", "text/plain")], "Hello, World!").into_response()
    }

    async fn db_handler(State(state): State<AppState>) -> Json<World> {
        let client = state.pool.get().await.expect("DB pool error");
        let world = database::fetch_world_by_id(&client, utils::random_id())
            .await
            .expect("DB query failed");
        Json(world)
    }

    async fn queries_handler(
        State(state): State<AppState>,
        Query(params): Query<QueryParams>,
    ) -> Json<Vec<World>> {
        let count = utils::clamp_query_count(params.queries);
        let client = state.pool.get().await.expect("DB pool error");
        let mut worlds = Vec::with_capacity(count);
        for _ in 0..count {
            let world = database::fetch_world_by_id(&client, utils::random_id())
                .await
                .expect("DB query failed");
            worlds.push(world);
        }
        Json(worlds)
    }

    async fn updates_handler(
        State(state): State<AppState>,
        Query(params): Query<QueryParams>,
    ) -> Json<Vec<World>> {
        let count = utils::clamp_query_count(params.queries);
        let client = state.pool.get().await.expect("DB pool error");
        let mut worlds = Vec::with_capacity(count);
        for _ in 0..count {
            let mut world = database::fetch_world_by_id(&client, utils::random_id())
                .await
                .expect("DB query failed");
            world.random_number = utils::random_id();
            database::update_world(&client, &world)
                .await
                .expect("DB update failed");
            worlds.push(world);
        }
        Json(worlds)
    }

    async fn cached_worlds_handler(
        State(state): State<AppState>,
        Query(params): Query<CountParams>,
    ) -> Json<Vec<World>> {
        let count = utils::clamp_query_count(params.count);
        let client = state.pool.get().await.expect("DB pool error");
        let mut worlds = Vec::with_capacity(count);
        for _ in 0..count {
            let id = utils::random_id();
            if let Some(world) = state.cache.get(&id) {
                worlds.push(world.as_ref().clone());
            } else {
                let world = database::fetch_cached_world_by_id(&client, id)
                    .await
                    .expect("DB query failed");
                state.cache.insert(id, Arc::new(world.clone()));
                worlds.push(world);
            }
        }
        Json(worlds)
    }

    async fn fortunes_handler(State(state): State<AppState>) -> HtmlTemplate<FortunesTemplate> {
        let client = state.pool.get().await.expect("DB pool error");
        let fortunes = database::fetch_all_fortunes(&client)
            .await
            .expect("DB query failed");
        let fortunes = append_fortune(fortunes);
        HtmlTemplate(FortunesTemplate { fortunes })
    }

    pub async fn run() {
        println!("🚀 Axum server running on http://0.0.0.0:8080");
        let pool = database::create_pool();
        let cache = database::create_cache();
        if let Err(err) = database::warm_cache(&pool, &cache).await {
            eprintln!("Failed to warm cache at startup: {err}");
        }

        let state = AppState { pool, cache };
        let app = Router::new()
            .route("/json", get(json_handler))
            .route("/plaintext", get(plaintext_handler))
            .route("/db", get(db_handler))
            .route("/queries", get(queries_handler))
            .route("/updates", get(updates_handler))
            .route("/cached-worlds", get(cached_worlds_handler))
            .route("/fortunes", get(fortunes_handler))
            .with_state(state);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
            .await
            .unwrap();
        axum::serve(listener, app).await.unwrap();
    }
}

// Rocket implementation
#[cfg(feature = "rocket_server")]
mod rocket_impl {
    use super::{database, models::Fortune, models::World, utils, Message};
    use rocket::{get, routes, serde::json::Json, Config, State};
    use rocket_dyn_templates::Template;
    use serde::Serialize;
    use std::net::Ipv4Addr;
    use std::sync::Arc;

    use database::{DbPool, WorldCache};

    #[derive(Serialize)]
    struct FortunesContext {
        fortunes: Vec<Fortune>,
    }

    fn append_fortune(mut fortunes: Vec<Fortune>) -> Vec<Fortune> {
        fortunes.push(Fortune {
            id: 0,
            message: "Additional fortune added at request time.".to_string(),
        });
        fortunes.sort_by(|a, b| a.message.cmp(&b.message));
        fortunes
    }

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

    #[get("/db")]
    async fn db_handler(pool: &State<DbPool>) -> Json<World> {
        let client = pool.get().await.expect("DB pool error");
        let world = database::fetch_world_by_id(&client, utils::random_id())
            .await
            .expect("DB query failed");
        Json(world)
    }

    #[get("/queries?<queries>")]
    async fn queries_handler(pool: &State<DbPool>, queries: Option<u16>) -> Json<Vec<World>> {
        let count = utils::clamp_query_count(queries);
        let client = pool.get().await.expect("DB pool error");
        let mut worlds = Vec::with_capacity(count);
        for _ in 0..count {
            let world = database::fetch_world_by_id(&client, utils::random_id())
                .await
                .expect("DB query failed");
            worlds.push(world);
        }
        Json(worlds)
    }

    #[get("/updates?<queries>")]
    async fn updates_handler(pool: &State<DbPool>, queries: Option<u16>) -> Json<Vec<World>> {
        let count = utils::clamp_query_count(queries);
        let client = pool.get().await.expect("DB pool error");
        let mut worlds = Vec::with_capacity(count);
        for _ in 0..count {
            let mut world = database::fetch_world_by_id(&client, utils::random_id())
                .await
                .expect("DB query failed");
            world.random_number = utils::random_id();
            database::update_world(&client, &world)
                .await
                .expect("DB update failed");
            worlds.push(world);
        }
        Json(worlds)
    }

    #[get("/cached-worlds?<count>")]
    async fn cached_worlds_handler(
        pool: &State<DbPool>,
        cache: &State<WorldCache>,
        count: Option<u16>,
    ) -> Json<Vec<World>> {
        let count = utils::clamp_query_count(count);
        let client = pool.get().await.expect("DB pool error");
        let mut worlds = Vec::with_capacity(count);
        for _ in 0..count {
            let id = utils::random_id();
            if let Some(world) = cache.get(&id) {
                worlds.push(world.as_ref().clone());
            } else {
                let world = database::fetch_cached_world_by_id(&client, id)
                    .await
                    .expect("DB query failed");
                cache.insert(id, Arc::new(world.clone()));
                worlds.push(world);
            }
        }
        Json(worlds)
    }

    #[get("/fortunes")]
    async fn fortunes_handler(pool: &State<DbPool>) -> Template {
        let client = pool.get().await.expect("DB pool error");
        let fortunes = database::fetch_all_fortunes(&client)
            .await
            .expect("DB query failed");
        let fortunes = append_fortune(fortunes);
        Template::render("rocket_fortunes", &FortunesContext { fortunes })
    }

    pub async fn run() -> Result<(), rocket::Error> {
        println!("🚀 Rocket server running on http://0.0.0.0:8080");
        let pool = database::create_pool();
        let cache = database::create_cache();
        if let Err(err) = database::warm_cache(&pool, &cache).await {
            eprintln!("Failed to warm cache at startup: {err}");
        }

        let config = Config {
            port: 8080,
            address: Ipv4Addr::new(0, 0, 0, 0).into(),
            ..Config::default()
        };

        let _ = rocket::custom(&config)
            .manage(pool)
            .manage(cache)
            .mount(
                "/",
                routes![
                    json_handler,
                    plaintext_handler,
                    db_handler,
                    queries_handler,
                    updates_handler,
                    cached_worlds_handler,
                    fortunes_handler,
                ],
            )
            .attach(Template::fairing())
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
