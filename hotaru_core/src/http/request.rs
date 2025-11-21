use crate::http::cookie::Cookie;
use crate::http::safety::HttpSafety; 

use super::{http_value::*, net}; 
use super::body::HttpBody;
use super::meta::HttpMeta;
use super::start_line::{HttpStartLine};
use super::form::UrlEncodedForm;
use std::collections::HashMap;  
use tokio::io::{AsyncRead, AsyncWrite, BufReader, BufWriter}; 

/// Represents an HTTP request with metadata and body.
/// 
/// This struct contains all information about an incoming HTTP request, 
/// including headers, method, URL, and body content.
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub meta: HttpMeta,
    pub body: HttpBody
}

impl HttpRequest { 
    pub fn new(meta: HttpMeta, body: HttpBody) -> Self { 
        HttpRequest { meta, body } 
    } 
    
    pub fn meta(&self) -> &HttpMeta { 
        &self.meta 
    } 

    /// Parses the HTTP request from a stream, returning an `HttpRequest` instance. 
    /// The stream is expected to be a `BufReader` wrapping a `TcpStream`. 
    /// Body will not be parsed 
    pub async fn parse_lazy<R: AsyncRead + Unpin>(stream: &mut BufReader<R>, config: &HttpSafety, print_raw: bool) -> Self {
        match net::parse_lazy(stream, config, true, print_raw).await { 
            Ok((meta, body)) => Self::new(meta, body), 
            Err(_) => Self::default() 
        }
    } 

    /// Parses the HTTP Body from buffer 
    pub async fn parse_body(&mut self, safety_setting: &HttpSafety) {
        let body = std::mem::take(&mut self.body);
        self.body = body.parse_buffer(safety_setting);  
    } 

    /// Add a cookie into the response metadata. 
    pub fn add_cookie<T: Into<String>>(mut self, key: T, cookie: Cookie) -> Self { 
        self.meta.add_cookie(key, cookie); 
        self 
    } 

    /// Set content type for Http Response 
    pub fn content_type(mut self, content_type: HttpContentType) -> Self { 
        self.meta.set_content_type(content_type); 
        self 
    } 

    /// Add a header for Http Request 
    pub fn add_header<T: Into<String>, U: Into<String>>(mut self, key: T, value: U) -> Self { 
        self.meta.set_attribute(key, value.into()); 
        self 
    } 

    /// Set the content disposition for the request. 
    pub fn content_disposition(mut self, disposition: ContentDisposition) -> Self { 
        self.meta.set_content_disposition(disposition); 
        self 
    } 
    
    pub async fn send<W: AsyncWrite + Unpin>(self, writer: &mut BufWriter<W>) -> std::io::Result<()> { 
        net::send(self.meta, self.body, writer).await 
    } 
}

impl Default for HttpRequest {
    fn default() -> Self {
        let meta = HttpMeta::new(
            HttpStartLine::new_request(
                HttpVersion::Http11,
                HttpMethod::GET,
                "/".to_string()
            ),
            HashMap::new()
        );
        let body = HttpBody::default();
        HttpRequest::new(meta, body)
    } 
} 

/// Collection of helper functions to easily create common HTTP requests. 
pub mod request_templates {
    use std::collections::HashMap;

    use akari::Value;

    use crate::http::{body::HttpBody, http_value::{HttpContentType, HttpMethod, HttpVersion}, meta::HttpMeta, start_line::HttpStartLine, form::UrlEncodedForm};

    use super::HttpRequest;
 
    pub fn get_request<T: Into<String>>(url: T) -> HttpRequest { 
        let meta = HttpMeta::new(
            HttpStartLine::new_request(
                HttpVersion::Http11,
                HttpMethod::GET,
                url.into(), 
            ),
            HashMap::new(),
        );
        let body = HttpBody::Unparsed;
        HttpRequest::new(meta, body) 
    } 

    pub fn json_request<T: Into<String>>(url: T, body: Value) -> HttpRequest { 
        let start_line = HttpStartLine::new_request(
            HttpVersion::Http11, 
            HttpMethod::POST, 
            url.into() 
        ); 
        let mut meta = HttpMeta::new(start_line, HashMap::new()); 
        meta.set_content_type(HttpContentType::ApplicationJson()); 
        HttpRequest::new(meta, HttpBody::Json(body)) 
    }
    
    /// Creates a POST request with form data (application/x-www-form-urlencoded)
    /// User passes in a UrlEncodedForm directly
    pub fn form_post<T: Into<String>>(url: T, form: UrlEncodedForm) -> HttpRequest {
        let start_line = HttpStartLine::new_request(
            HttpVersion::Http11,
            HttpMethod::POST,
            url.into()
        );
        let meta = HttpMeta::new(start_line, HashMap::new());
        HttpRequest::new(meta, HttpBody::Form(form))
    }
    
    /// Creates a GET request with query parameters
    /// Builds the query string using UrlEncodedForm's encoding
    pub fn get_with_params<T: Into<String>>(url: T, params: HashMap<String, String>) -> HttpRequest {
        let base_url = url.into();
        
        // Build query string using UrlEncodedForm
        let query = if params.is_empty() {
            String::new()
        } else {
            let form = UrlEncodedForm::from(params);
            let encoded = form.to_string();
            
            if base_url.contains('?') {
                format!("&{}", encoded)
            } else {
                format!("?{}", encoded)
            }
        };
        
        let full_url = format!("{}{}", base_url, query);
        get_request(full_url)
    }
    
    /// Creates a POST request with plain text body
    pub fn text_post<T: Into<String>, B: Into<String>>(url: T, text: B) -> HttpRequest {
        let start_line = HttpStartLine::new_request(
            HttpVersion::Http11,
            HttpMethod::POST,
            url.into()
        );
        let mut meta = HttpMeta::new(start_line, HashMap::new());
        meta.set_content_type(HttpContentType::Text {
            subtype: "plain".to_string(),
            charset: Some("utf-8".to_string()),
        });
        HttpRequest::new(meta, HttpBody::Binary(text.into().into_bytes()))
    }
    
    /// Creates a PUT request with JSON body
    pub fn json_put<T: Into<String>>(url: T, body: Value) -> HttpRequest {
        let start_line = HttpStartLine::new_request(
            HttpVersion::Http11,
            HttpMethod::PUT,
            url.into()
        );
        let mut meta = HttpMeta::new(start_line, HashMap::new());
        meta.set_content_type(HttpContentType::ApplicationJson());
        HttpRequest::new(meta, HttpBody::Json(body))
    }
    
    /// Creates a PATCH request with JSON body
    pub fn json_patch<T: Into<String>>(url: T, body: Value) -> HttpRequest {
        let start_line = HttpStartLine::new_request(
            HttpVersion::Http11,
            HttpMethod::PATCH,
            url.into()
        );
        let mut meta = HttpMeta::new(start_line, HashMap::new());
        meta.set_content_type(HttpContentType::ApplicationJson());
        HttpRequest::new(meta, HttpBody::Json(body))
    }
    
    /// Creates a DELETE request
    pub fn delete<T: Into<String>>(url: T) -> HttpRequest {
        let start_line = HttpStartLine::new_request(
            HttpVersion::Http11,
            HttpMethod::DELETE,
            url.into()
        );
        let meta = HttpMeta::new(start_line, HashMap::new());
        HttpRequest::new(meta, HttpBody::Unparsed)
    }
    
    /// Creates a HEAD request
    pub fn head<T: Into<String>>(url: T) -> HttpRequest {
        let start_line = HttpStartLine::new_request(
            HttpVersion::Http11,
            HttpMethod::HEAD,
            url.into()
        );
        let meta = HttpMeta::new(start_line, HashMap::new());
        HttpRequest::new(meta, HttpBody::Unparsed)
    }
} 
