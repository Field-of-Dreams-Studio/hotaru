use hotaru::prelude::*; 
use hotaru::http::*;  
use std::collections::HashMap;

use crate::APP;

// ============================================================================
// HTTP Client examples - demonstrating outbound HTTP requests
// ============================================================================

endpoint! {
    APP.url("/client/test_get"),
    
    /// Test making a GET request to an external API
    pub client_get <HTTP> {
        // Make a simple GET request
        let request = get_request("/");
        
        match HttpContext::send_request("http://httpbin.org", request, HttpSafety::default()).await {
            Ok(response) => {
                let status = response.meta.start_line.status_code(); 
                let raw = response.body.raw(); 
                
                text_response(format!(
                    "GET Request successful!\nStatus: {}\nBody preview: {}...",
                    status, String::from_utf8_lossy(&raw[..100.min(raw.len())]),
                ))
            }
            Err(e) => {
                text_response(format!("Request failed: {:?}", e))
            }
        }
    }
}

endpoint! {
    APP.url("/client/test_get_params"),
    
    /// Test GET request with query parameters
    pub client_get_params <HTTP> {
        // Create parameters
        let mut params = HashMap::new();
        params.insert("key1".to_string(), "value1".to_string());
        params.insert("key2".to_string(), "test value with spaces".to_string());
        params.insert("special".to_string(), "chars&=?".to_string());
        
        let request = get_with_params("/get", params);
        
        match HttpContext::send_request("http://httpbin.org", request, HttpSafety::default()).await {
            Ok(response) => {
                json_response(object!({
                    status: response.meta.start_line.status_code() as u16,
                    body: String::from_utf8_lossy(&response.body.raw()).to_string()
                }))
            }
            Err(e) => {
                json_response(object!({
                    error: format!("{:?}", e)
                }))
            }
        }
    }
}

endpoint! {
    APP.url("/client/test_form_post"),
    
    /// Test POST request with form data
    pub client_form_post <HTTP> {
        // Create form data
        let mut form_data = HashMap::new();
        form_data.insert("username".to_string(), "testuser".to_string());
        form_data.insert("email".to_string(), "test@example.com".to_string());
        form_data.insert("message".to_string(), "Hello from Hotaru!".to_string());
        
        // Convert to UrlEncodedForm
        let form = UrlEncodedForm::from(form_data);
        let request = form_post("/post", form);
        
        match HttpContext::send_request("http://httpbin.org", request, HttpSafety::default()).await {
            Ok(response) => {
                json_response(object!({
                    status: response.meta.start_line.status_code() as u16,
                    response_body: String::from_utf8_lossy(&response.body.raw()).to_string()
                }))
            }
            Err(e) => {
                json_response(object!({
                    error: format!("{:?}", e)
                }))
            }
        }
    }
}

endpoint! {
    APP.url("/client/test_json_post"),
    
    /// Test POST request with JSON data
    pub client_json_post <HTTP> {
        // Create JSON payload
        let json_data = object!({
            name: "Hotaru Client",
            version: "0.7.0",
            features: ["async", "client", "forms"],
            metadata: {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                source: "example_hotaru"
            }
        });
        
        let request = json_request("/post", json_data);
        
        match HttpContext::send_request("http://httpbin.org", request, HttpSafety::default()).await {
            Ok(response) => {
                // Parse the response
                json_response(object!({
                    status: response.meta.start_line.status_code() as u16,
                    sent_successfully: true,
                    echo_response: response.body.raw() 
                }))
            }
            Err(e) => {
                json_response(object!({
                    error: format!("{:?}", e),
                    sent_successfully: false
                }))
            }
        }
    }
}

endpoint! {
    APP.url("/client/test_put"),
    
    /// Test PUT request with JSON
    pub client_put <HTTP> {
        let update_data = object!({
            id: 123,
            updated_field: "new value",
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        });
        
        let request = json_put("/put", update_data);
        
        match HttpContext::send_request("http://httpbin.org", request, HttpSafety::default()).await {
            Ok(response) => {
                text_response(format!(
                    "PUT request successful! Status: {}", 
                    response.meta.start_line.status_code() as u16
                ))
            }
            Err(e) => {
                text_response(format!("PUT request failed: {:?}", e))
            }
        }
    }
}

endpoint! {
    APP.url("/client/test_delete"),
    
    /// Test DELETE request
    pub client_delete <HTTP> {
        let request = delete("/delete");
        
        match HttpContext::send_request("http://httpbin.org", request, HttpSafety::default()).await {
            Ok(response) => {
                text_response(format!(
                    "DELETE request successful! Status: {}", 
                    response.meta.start_line.status_code() as u16
                ))
            }
            Err(e) => {
                text_response(format!("DELETE request failed: {:?}", e))
            }
        }
    }
}

endpoint! {
    APP.url("/client/chain_requests"),
    
    /// Demonstrate chaining multiple requests
    pub client_chain <HTTP> {
        let mut results = Vec::new();
        
        // First request: GET
        let get_request = get_request("/get");
        match HttpContext::send_request("http://httpbin.org", get_request, HttpSafety::default()).await {
            Ok(response) => {
                results.push(format!("GET: {}", response.meta.start_line.status_code() as u16));
            }
            Err(e) => {
                results.push(format!("GET failed: {:?}", e));
            }
        }
        
        // Second request: POST with form
        let mut form_data = HashMap::new();
        form_data.insert("step".to_string(), "2".to_string());
        form_data.insert("previous".to_string(), "GET successful".to_string());
        
        let form = UrlEncodedForm::from(form_data);
        let post_request = form_post("/post", form);
        
        match HttpContext::send_request("http://httpbin.org", post_request, HttpSafety::default()).await {
            Ok(response) => {
                results.push(format!("POST: {}", response.meta.start_line.status_code() as u16));
            }
            Err(e) => {
                results.push(format!("POST failed: {:?}", e));
            }
        }
        
        // Third request: JSON
        let json_data = object!({
            step: 3,
            results_so_far: results.len()
        });
        
        let json_request = json_request("/post", json_data);
        
        match HttpContext::send_request("http://httpbin.org", json_request, HttpSafety::default()).await {
            Ok(response) => {
                results.push(format!("JSON POST: {}", response.meta.start_line.status_code() as u16));
            }
            Err(e) => {
                results.push(format!("JSON POST failed: {:?}", e));
            }
        }
        
        json_response(object!({
            chain_complete: true,
            steps_executed: results.len(),
            results: results
        }))
    }
}

endpoint! {
    APP.url("/client/proxy_request"),
    
    /// Act as a proxy - forward the client's request to another server
    pub client_proxy <HTTP> {
        // Get the query parameter for the target URL
        let target_path = req.query("path").unwrap_or("/".to_string());
        
        // Forward the same method and body
        let forwarded_request = if req.method() == POST {
            if let Some(form) = req.form().await.cloned() {
                form_post(target_path, form)
            } else if let Some(json) = req.json().await.cloned() {
                json_request(target_path, json)
            } else {
                text_post(target_path, "")
            }
        } else {
            get_request(target_path)
        };
        
        match HttpContext::send_request("http://httpbin.org", forwarded_request, HttpSafety::default()).await {
            Ok(response) => {
                // Forward the response back to the client
                HttpResponse::new(
                    response.meta.clone(),
                    response.body.clone()
                )
            }
            Err(e) => {
                text_response(format!("Proxy error: {:?}", e))
            }
        }
    }
}