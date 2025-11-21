use crate::http::http_value::{ContentDisposition, StatusCode};
use crate::http::safety::HttpSafety;

use super::body::HttpBody;
use super::cookie::Cookie;
use super::http_value::HttpContentType;
use super::meta::HttpMeta;
use super::net;
use super::start_line::{HttpStartLine, ResponseStartLine};
use std::collections::HashMap;
use tokio::io::{AsyncRead, AsyncWrite, BufReader, BufWriter};

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub meta: HttpMeta,
    pub body: HttpBody,
}

impl HttpResponse {
    pub fn new(meta: HttpMeta, body: HttpBody) -> Self {
        Self { meta, body }
    }

    pub async fn parse_lazy<R: AsyncRead + Unpin>(
        stream: &mut BufReader<R>,
        config: &HttpSafety,
        print_raw: bool,
    ) -> Self {
        match net::parse_lazy(stream, config, false, print_raw).await {
            Ok((meta, body)) => Self::new(meta, body),
            Err(_) => Self::default(),
        }
    }

    /// Parses the HTTP Body from buffer
    pub async fn parse_body(&mut self, safety_setting: &HttpSafety) {
        let body = std::mem::take(&mut self.body);
        self.body = body.parse_buffer(safety_setting);
    } 

    /// Get the parsed HTTP Body 
    pub async fn get_parsed_body(&mut self, safety: HttpSafety) -> HttpBody {
        self.parse_body(&safety).await; 
        self.body.clone() 
    } 

    /// Add a cookie into the response metadata.
    /// Insert an empty cookie to delete the cookie.
    pub fn add_cookie<T: Into<String>>(mut self, key: T, cookie: Cookie) -> Self {
        self.meta.add_cookie(key, cookie);
        self
    }

    /// Set content type for Http Response
    pub fn content_type(mut self, content_type: HttpContentType) -> Self {
        self.meta.set_content_type(content_type);
        self
    }

    /// Add a header for Http Response
    pub fn add_header<T: Into<String>, U: Into<String>>(mut self, key: T, value: U) -> Self {
        self.meta.set_attribute(key, value.into());
        self
    }

    /// Set the content disposition for the request.
    pub fn content_disposition(mut self, disposition: ContentDisposition) -> Self {
        self.meta.set_content_disposition(disposition);
        self
    }

    /// Send a status
    pub fn status<T: Into<StatusCode>>(mut self, status: T) -> Self {
        self.meta.start_line.set_status_code(status);
        self
    }

    /// Send the response
    /// When this method is changed, please also check Request::send()
    pub async fn send<W: AsyncWrite + Unpin>(
        self,
        writer: &mut BufWriter<W>,
    ) -> std::io::Result<()> {
        net::send(self.meta, self.body, writer).await
    }

    // /// Converts this response into a Future that resolves to itself.
    // /// Useful for middleware functions that need to return a Future<Output = HttpResponse>.
    // pub fn future(self) -> impl Future<Output = HttpResponse> + Send {
    //     ready(self)
    // }

    // /// Creates a boxed future from this response (useful for trait objects).
    // pub fn boxed_future(self) -> Pin<Box<dyn Future<Output = HttpResponse> + Send>> {
    //     Box::pin(self.future())
    // }
}

impl Default for HttpResponse {
    fn default() -> Self {
        let meta = HttpMeta::new(
            HttpStartLine::Response(ResponseStartLine::default()),
            HashMap::new(),
        );
        let body = HttpBody::default(); // Default body is empty string
        HttpResponse::new(meta, body)
    }
}

/// Collection of helper functions to easily create common HTTP responses.
///
/// This module provides convenient functions to create standardized HTTP responses
/// such as text, HTML, JSON, redirects, status codes, and template-based responses.
/// All functions return an `HttpResponse` that can be further customized if needed.
pub mod response_templates {
    use std::collections::HashMap;
    use std::path::Path;

    use akari::TemplateManager;
    use akari::Value;

    use super::HttpResponse;
    use crate::http::body::HttpBody;
    use crate::http::http_value::{HttpContentType, HttpVersion, StatusCode};
    use crate::http::meta::HttpMeta;
    use crate::http::start_line::HttpStartLine;

    /// Creates a plain text HTTP response with status 200 OK.
    ///
    /// # Arguments
    ///
    /// * `body` - The text content to be sent in the response.
    ///
    /// # Returns
    ///
    /// An `HttpResponse` with Content-Type set to text/plain.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::response::response_templates;
    ///
    /// let response = response_templates::text_response("Hello, world!");
    /// ```
    pub fn text_response(body: impl Into<String>) -> HttpResponse {
        let start_line = HttpStartLine::new_response(HttpVersion::Http11, StatusCode::OK);
        let mut meta = HttpMeta::new(start_line, HashMap::new());
        meta.set_content_type(HttpContentType::TextPlain());
        HttpResponse::new(meta, HttpBody::Text(body.into()))
    }

    /// Creates an HTML HTTP response with status 200 OK.
    ///
    /// # Arguments
    ///
    /// * `body` - The HTML content to be sent in the response.
    ///
    /// # Returns
    ///
    /// An `HttpResponse` with Content-Type set to text/html.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::response::response_templates;
    ///
    /// let html = "<html><body><h1>Hello, world!</h1></body></html>";
    /// let response = response_templates::html_response(html);
    /// ```
    pub fn html_response(body: impl Into<Vec<u8>>) -> HttpResponse {
        let start_line = HttpStartLine::new_response(HttpVersion::Http11, StatusCode::OK);
        let mut meta = HttpMeta::new(start_line, HashMap::new());
        meta.set_content_type(HttpContentType::TextHtml());
        HttpResponse::new(meta, HttpBody::Binary(body.into()))
    }

    /// Creates a redirect response (302 Found).
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to redirect to.
    ///
    /// # Returns
    ///
    /// An `HttpResponse` with the Location header set and an empty body.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::response::response_templates;
    ///
    /// let response = response_templates::redirect_response("/login");
    /// ```
    pub fn redirect_response(url: &str) -> HttpResponse {
        let start_line = HttpStartLine::new_response(HttpVersion::Http11, StatusCode::FOUND);
        let mut meta = HttpMeta::new(start_line, HashMap::new());
        meta.set_location(Some(url.to_string()));
        HttpResponse::new(meta, HttpBody::Empty)
    }

    /// Creates an HTML response from a template file without any data binding.
    ///
    /// # Arguments
    ///
    /// * `file` - The filename of the template within the templates directory.
    /// Never use absolute path for file argument
    ///
    /// # Returns
    ///
    /// An `HttpResponse` with the template content or a 404 error if the file is not found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::response::response_templates;
    ///
    /// let response = response_templates::plain_template_response("index.html");
    /// ```
    pub fn plain_template_response(file: &str) -> HttpResponse {
        let start_line = HttpStartLine::new_response(HttpVersion::Http11, StatusCode::OK);
        let mut meta = HttpMeta::new(start_line, HashMap::new());
        let file_path = Path::new("templates").join(file);
        // println!("[Response] Loading template: {}", file_path.display());
        let body = match std::fs::read(file_path) {
            Ok(content) => content,
            Err(_) => return return_status(StatusCode::NOT_FOUND),
        };
        meta.set_content_type(HttpContentType::TextHtml());
        HttpResponse::new(meta, HttpBody::Binary(body))
    }

    pub fn serve_static_file(file: &str) -> HttpResponse {
        let start_line = HttpStartLine::new_response(HttpVersion::Http11, StatusCode::OK);
        let mut meta = HttpMeta::new(start_line, HashMap::new());
        let file_path = Path::new("templates").join(file);

        // Set the response content type based on the file extension
        meta.set_content_type(HttpContentType::from_file_name(
            file_path
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or(""),
        ));

        let body = match std::fs::read(file_path) {
            Ok(content) => content,
            Err(_) => return return_status(StatusCode::NOT_FOUND),
        };
        HttpResponse::new(meta, HttpBody::Binary(body))
    }

    /// Creates an HTTP response with a specified status code and binary body.
    ///
    /// # Arguments
    ///
    /// * `status_code` - The HTTP status code for the response.
    /// * `body` - The binary content to be sent in the response.
    ///
    /// # Returns
    ///
    /// An `HttpResponse` with the specified status code and body.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::response::response_templates;
    /// use hotaru_core::http::http_value::StatusCode;
    ///
    /// let response = response_templates::normal_response(StatusCode::CREATED, "Resource created");
    /// ```
    pub fn normal_response<S: Into<StatusCode>, B: Into<Vec<u8>>>(
        status_code: S,
        body: B,
    ) -> HttpResponse {
        let start_line = HttpStartLine::new_response(HttpVersion::Http11, status_code.into());
        let meta = HttpMeta::new(start_line, HashMap::new());
        HttpResponse::new(meta, HttpBody::Binary(body.into())).content_type(HttpContentType::Text {
            subtype: "plain".to_string(),
            charset: Some("utf-8".to_string()),
        })
    }

    /// Creates a JSON HTTP response with status 200 OK.
    ///
    /// # Arguments
    ///
    /// * `body` - The JSON object to be sent in the response.
    ///
    /// # Returns
    ///
    /// An `HttpResponse` with Content-Type set to application/json.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::response::response_templates;
    /// use akari::{Value, object};
    ///
    /// let mut data = object!({});
    /// data.set("message", "Success");
    /// data.set("status", 200);
    ///
    /// let response = response_templates::json_response(data);
    /// ```
    pub fn json_response(body: Value) -> HttpResponse {
        let start_line = HttpStartLine::new_response(HttpVersion::Http11, StatusCode::OK);
        let mut meta = HttpMeta::new(start_line, HashMap::new());
        meta.set_content_type(HttpContentType::ApplicationJson());
        HttpResponse::new(meta, HttpBody::Json(body))
    }

    /// Creates an HTML response from a template with data binding.
    ///
    /// # Arguments
    ///
    /// * `file` - The filename of the template within the templates directory.
    /// * `data` - A hashmap of values to be bound to the template.
    ///
    /// # Returns
    ///
    /// An `HttpResponse` with the rendered template or an error message if rendering fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::response::response_templates;
    /// use akari::Object;
    /// use std::collections::HashMap;
    ///
    /// let mut data = HashMap::new();
    /// let mut user = Object::new();
    /// user.insert("name", "John Doe");
    /// data.insert("user", user);
    ///
    /// let response = response_templates::template_response("user_profile.html", data);
    /// ```
    pub fn template_response(file: &str, data: HashMap<String, Value>) -> HttpResponse {
        let template_manager = TemplateManager::new("templates");
        let result = match template_manager.render(file, &data) {
            Ok(content) => content,
            Err(err) => return text_response(err.to_string()),
        };

        let start_line = HttpStartLine::new_response(HttpVersion::Http11, StatusCode::OK);
        let mut meta = HttpMeta::new(start_line, HashMap::new());
        meta.set_content_type(HttpContentType::TextHtml());

        let body = result.into_bytes();
        HttpResponse::new(meta, HttpBody::Binary(body))
    }

    /// Creates an HTTP response with only a status code and an empty body.
    ///
    /// # Arguments
    ///
    /// * `status_code` - The HTTP status code for the response.
    ///
    /// # Returns
    ///
    /// An `HttpResponse` with the specified status code and an empty body.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::response::response_templates;
    /// use hotaru_core::http::http_value::StatusCode;
    ///
    /// // Return a 404 Not Found response
    /// let response = response_templates::return_status(StatusCode::NOT_FOUND);
    /// ```
    pub fn return_status(status_code: StatusCode) -> HttpResponse {
        normal_response(status_code, Vec::<u8>::new())
    }
}

// pub mod akari_templates {
//     /// This macro is used to create a template response with the given path and key-value pairs.
//     /// It renders a template within specified path
//     /// and inserts the key-value pairs into the template context.
//     /// It is a convenient way to generate dynamic HTML responses.
//     /// # Examples
//     /// ```rust
//     /// // akari_render!("/path/to/template", key1 = "value1", key2 = "value2");
//     /// // This will fail because the template does not exist.
//     /// ```
//     #[macro_export]
//     macro_rules! akari_render {
//         ($path:expr) => {{
//             template_response($path, std::collections::HashMap::new())
//         }};
//         ($path:expr, $($key:ident = $value:tt),* $(,)?) => {{
//             let mut map = std::collections::HashMap::new();
//             $(
//                 akari_render!(@insert map, $key = $value);
//             )*
//             template_response($path, map)
//         }};
//         (@insert $map:expr, $key:ident = $value:literal) => {
//             $map.insert(stringify!($key).to_string(), object!($value));
//         };
//         (@insert $map:expr, $key:ident = $value:expr) => {
//             $map.insert(stringify!($key).to_string(), $value);
//         };
//     }
// }

// pub mod akari_object {
//     /// This macro is used to create a JSON response with the given key-value pairs.
//     /// It is a convenient way to generate JSON responses.
//     #[macro_export]
//     macro_rules! akari_json {
//         // Forward any input to the object! macro and wrap the result in json_response
//         ($($tokens:tt)*) => {{
//             let obj = object!($($tokens)*);
//             json_response(obj)
//         }};
//     }
// }
