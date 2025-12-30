use hotaru::prelude::*;
use hotaru::http::*;  
use tokio::time::sleep;

use crate::APP;

// ============================================================================
// Async endpoints examples
// ============================================================================

endpoint! {
    APP.url("/async/channel1"),
    
    /// Async test endpoint with delays
    pub async_test <HTTP> {
        sleep(Duration::from_secs(1)).await;
        println!("1");
        sleep(Duration::from_secs(1)).await;
        println!("2");
        sleep(Duration::from_secs(1)).await;
        println!("3");
        text_response("Async Test Page")
    }
}

endpoint! {
    APP.url("/async/channel2"),
    
    /// Another async test endpoint
    pub async_test2 <HTTP> {
        sleep(Duration::from_secs(1)).await;
        println!("Channel 2: 1");
        sleep(Duration::from_secs(1)).await;
        println!("Channel 2: 2");
        sleep(Duration::from_secs(1)).await;
        println!("Channel 2: 3");
        text_response("Async Test Page 2")
    }
} 

endpoint!{ 
    APP.url("/async/whoami"),
    
    /// Return the IP address of the client 
    pub whoami<HTTP> {
        // Get client's full socket address (IP + port)
        match req.client_ip() {
            Some(addr) => text_response(format!("Your address: {}", addr)),
            None => text_response("Unknown client"),
        }
    }
}

endpoint! {
    APP.url("/async/<int:number>"),
    
    /// Pattern matching for numbers
    pub number_page <HTTP> {
        let number = req.pattern("number").unwrap_or("unknown".to_string());
        text_response(format!("Number page: {}", number))
    }
}

endpoint! {
    APP.url("/async/get_secret_key"),
    
    /// Get a value from app config
    pub get_secret_key <HTTP> {
        let key = req.app()
            .and_then(|app| app.config.get::<String>().cloned())
            .unwrap_or("No key".to_string());
        text_response(key)
    }
}

endpoint! {
    APP.url("/async/file"),
    
    /// File download example
    pub file_download <HTTP> {
        let file_path = "Cargo.toml"; // Example file to serve
        match std::fs::read(file_path) {
            Ok(content) => {
                normal_response(StatusCode::OK, content)
                    .add_header("Content-Disposition", "attachment; filename=\"Cargo.toml\"")
            }
            Err(e) => text_response(format!("Error reading file: {}", e)),
        }
    }
}

endpoint! {
    APP.url("/async/get"),
    config = [HttpSafety::new().with_allowed_method(GET)],
    
    /// GET-only endpoint
    pub get_only <HTTP> {
        text_response("GET only endpoint")
    }
}

endpoint! {
    APP.url("/async/post"),
    config = [HttpSafety::new().with_allowed_methods(vec![POST])],
    
    /// POST-only endpoint
    pub post_only <HTTP> {
        text_response("POST only endpoint")
    }
}