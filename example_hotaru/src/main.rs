use hotaru::prelude::*;
use hotaru::http::*;

// Global middleware for the app
middleware! {
    /// Global logger middleware
    pub GlobalLogger <HTTP> {
        println!("🌍 GlobalLogger: Processing {}", req.path());
        next(req).await
    }
}

middleware! {
    /// Global metrics middleware  
    pub GlobalMetrics <HTTP> {
        println!("📊 GlobalMetrics: Tracking {}", req.path());
        next(req).await
    }
}

// Define the app with global middleware
pub static APP: SApp = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:3003")
        .set_config("This is a String Config")
        .single_protocol(
            ProtocolBuilder::new(HTTP::server(HttpSafety::default()))
                .append_middleware::<GlobalLogger>()
                .append_middleware::<GlobalMetrics>()
        ) 
        .build()
});

// Import modules
mod async_endpoints;
mod forms;
mod middleware_examples;
mod middleware_inheritance;
mod sessions;
mod http_client;

#[tokio::main(worker_threads = 16)]
async fn main() {
    println!("Starting Hotaru example server on 127.0.0.1:3003");
    APP.clone().run().await;
}

// ============================================================================
// Basic endpoints
// ============================================================================

endpoint! {
    APP.url("/"),
    
    /// Basic index endpoint using home.html template
    pub index <HTTP> {
        akari_render!(
            "home.html",
            title = "Hotaru Example",
            page_title = "Welcome to Hotaru 0.7",
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

endpoint! {
    APP.url("/form"),
    
    /// Form page using form.html template
    pub form_page <HTTP> {
        if req.method() == POST {
            match req.form().await {
                Some(form) => {
                    let mut response = String::from("Form data received:\n");
                    for (key, value) in form.data.iter() {
                        response.push_str(&format!("{}: {}\n", key, value));
                    }
                    text_response(response)
                }
                None => {
                    text_response("Error parsing form")
                }
            }
        } else {
            plain_template_response("form.html")
        }
    }
}

endpoint! {
    APP.url("/cookie"),
    
    /// Cookie page using cookie.html template
    pub cookie_page <HTTP> {
        if req.method() == POST {
            match req.form().await {
                Some(form) => {
                    let name = form.data.get("name").map(|s| s.as_str()).unwrap_or("");
                    let value = form.data.get("value").map(|s| s.as_str()).unwrap_or("");
                    text_response(format!("Cookie set: {} = {}", name, value))
                        .add_cookie(name, Cookie::new(value.to_string()).path("/"))
                }
                None => {
                    text_response("Error parsing form")
                }
            }
        } else {
            let cookies = req.get_cookies();
            let mut cookie_str = String::from("Current cookies:\n");
            for (name, cookie) in cookies.0.iter() {
                cookie_str.push_str(&format!("{}: {}\n", name, cookie.get_value()));
            }
            
            akari_render!("cookie.html",
                current_cookie = cookie_str
            )
        }
    }
}

endpoint! {
    APP.url("/health"),
    
    /// Health check endpoint
    pub health_check <HTTP> {
        json_response(object!({
            status: "healthy",
            service: "hotaru-example",
            version: "0.7.0",
            uptime: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        }))
    }
}
mod resource;
