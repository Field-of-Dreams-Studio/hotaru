use hotaru::prelude::*;
use hotaru::http::*; 

use crate::APP;

// ============================================================================
// Form handling examples
// ============================================================================

endpoint! {
    APP.url("/form/url_encoded"),
    
    /// Form URL-encoded data handling
    pub form_url_encoded <HTTP> {
        if req.method() == POST {
            match req.form().await {
                Some(form) => {
                    let mut response = String::from("Form data:\n");
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
            // Using akari_render for template response
            akari_render!("forms/url_encoded.html",
                title = "URL Encoded Form Test",
                action = "/form/url_encoded"
            )
        }
    }
}

endpoint! {
    APP.url("/form/multipart"),
    
    /// Multipart form data handling (file uploads)
    pub form_multipart <HTTP> {
        if req.method() == POST {
            let files = req.files_or_default().await;
            let mut response = String::from("Files received:\n");
            for (name, field) in files.get_all().iter() {
                if let MultiFormField::File(file_vec) = field {
                    for file in file_vec {
                        response.push_str(&format!("Field: {}, Size: {} bytes\n", name, file.data().len()));
                    }
                } else if let MultiFormField::Text(text) = field {
                    response.push_str(&format!("Field: {}, Text: {}\n", name, text));
                }
            }
            text_response(response)
        } else {
            akari_render!("forms/multipart.html",
                title = "Multipart Form Test",
                action = "/form/multipart"
            )
        }
    }
}

endpoint! {
    APP.url("/form/cookie"),
    
    /// Cookie handling example
    pub form_cookie <HTTP> {
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
            let mut cookie_list = Vec::new();
            for (name, cookie) in cookies.0.iter() {
                cookie_list.push(object!({
                    name: name.clone(),
                    value: cookie.get_value()
                }));
            }
            
            akari_render!("forms/cookie.html",
                title = "Cookie Test",
                cookies = cookie_list
            )
        }
    }
}

endpoint! {
    APP.url("/form/json"),
    
    /// JSON data handling
    pub form_json <HTTP> {
        if req.method() == POST {
            match req.json().await {
                Some(json) => {
                    let received_json = json.clone();
                    let message = "JSON data received successfully";
                    json_response(object!({
                        received: received_json,
                        message: message
                    }))
                }
                None => {
                    let error = "Failed to parse JSON";
                    json_response(object!({
                        error: error
                    }))
                }
            }
        } else {
            akari_render!("forms/json.html",
                title = "JSON Test",
                endpoint = "/form/json"
            )
        }
    }
}