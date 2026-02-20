use crate::connection::error::ConnectionError;
use crate::http::encoding::HttpEncoding;
use crate::http::safety::HttpSafety;

use super::cookie::{Cookie, CookieMap}; 

use super::http_value::*; 
use super::start_line::HttpStartLine; 
use std::collections::{HashMap, HashSet}; 
use tokio::io::{AsyncBufRead, AsyncBufReadExt}; 
use std::str; 

/// RequestHeader is a struct that represents the headers of an HTTP request. 
#[derive(Debug, Clone)]
pub struct HttpMeta { 
    pub start_line: HttpStartLine, 
    pub header: HashMap<String, HeaderValue>,  

    // Content-type header, overrides the content type from the hashmap if present 
    content_type: Option<HttpContentType>, 

    // Content-length header, overrides the content length from the hashmap if present 
    content_length: Option<usize>, 

    // Cookies header in request, Set-Cookie header in response 
    cookies: Option<CookieMap>, 

    // Content-Disposition header, used for file downloads in responses 
    content_disposition: Option<ContentDisposition>, 

    /// Transfer-Encoding header, used for chunked transfer encoding in responses 
    encoding: Option<HttpEncoding>, 

    // Host header, overrides the content length from the hashmap if present  
    host: Option<String>, 

    // Accept-Language header in request and Content-Language header in response 
    // Overrides the content length from the hashmap if present   
    lang: Option<AcceptLang>, 

    /// Location header, used for redirects in responses 
    location: Option<String> 
} 

/// Represents a value for an HTTP header, which can be either a single string or multiple values.
/// 
/// HTTP headers can sometimes have multiple values, which are typically combined with commas,
/// but some special headers like Set-Cookie maintain separate values.
#[derive(Debug, Clone)]
pub enum HeaderValue {
    /// A single header value
    Single(String),
    /// Multiple header values
    Multiple(Vec<String>),
}

impl HeaderValue { 
    /// Create a new HeaderValue from a single string.
    /// 
    /// # Arguments
    /// 
    /// * `value` - A string that represents the header value.
    /// 
    /// # Returns
    /// 
    /// A new HeaderValue containing a single value.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use hotaru_core::http::meta::HeaderValue;
    /// let header = HeaderValue::new("application/json");
    /// ```
    pub fn new<T: Into<String>>(value: T) -> Self {
        HeaderValue::Single(value.into())
    }

    /// Append a new value to the HeaderValue.
    /// 
    /// If the HeaderValue is a single value, it will convert it to a multiple value.
    /// Values are typically combined with comma separators for standard HTTP headers.
    /// 
    /// # Arguments 
    /// 
    /// * `value` - A string that represents the header value to append.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use hotaru_core::http::meta::HeaderValue;
    /// let mut header_value = HeaderValue::new("text/html");
    /// header_value.append("charset=UTF-8");
    /// assert_eq!(header_value.as_str(), "text/html, charset=UTF-8");
    /// ```
    pub fn append<T: Into<String>>(&mut self, value: T) {
        match self {
            HeaderValue::Single(s) => {
                let mut values = vec![s.clone()];
                values.push(value.into());
                *self = HeaderValue::Multiple(values);
            }
            HeaderValue::Multiple(v) => v.push(value.into()),
        }
    }

    /// Convert the HeaderValue to a string representation.
    /// 
    /// Multiple values are joined with a comma and space, following HTTP header conventions.
    /// 
    /// # Returns
    /// 
    /// A string representation of the header value(s).
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use hotaru_core::http::meta::HeaderValue;
    /// let mut header_value = HeaderValue::new("text/html");
    /// header_value.append("application/xhtml+xml");
    /// assert_eq!(header_value.as_str(), "text/html, application/xhtml+xml");
    /// ```
    pub fn as_str(&self) -> String {
        match self {
            HeaderValue::Single(s) => s.clone(),
            HeaderValue::Multiple(v) => v.join(", "),
        }
    }

    /// Returns the number of values in this HeaderValue.
    /// 
    /// # Returns
    /// 
    /// * `usize` - 1 for a single value, or the count of values for multiple values.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use hotaru_core::http::meta::HeaderValue;
    /// let mut header = HeaderValue::new("text/html");
    /// assert_eq!(header.len(), 1);
    /// 
    /// header.append("application/json");
    /// assert_eq!(header.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        match self {
            HeaderValue::Single(_) => 1,
            HeaderValue::Multiple(v) => v.len(),
        }
    }

    /// Checks if the HeaderValue is empty.
    /// 
    /// A HeaderValue is considered empty if it contains no values or only empty strings.
    /// 
    /// # Returns
    /// 
    /// `true` if the header value is empty, `false` otherwise.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use hotaru_core::http::meta::HeaderValue;
    /// let empty_header = HeaderValue::new("");
    /// assert!(empty_header.is_empty());
    /// 
    /// let header = HeaderValue::new("application/json");
    /// assert!(!header.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        match self {
            HeaderValue::Single(s) => s.is_empty(),
            HeaderValue::Multiple(v) => v.is_empty() || v.iter().all(|s| s.is_empty()),
        }
    }

    /// Attempts to get a value at the specified index.
    /// 
    /// For a single value, only index 0 is valid.
    /// For multiple values, any valid index within the range of values is accepted.
    /// 
    /// # Arguments
    /// 
    /// * `index` - The index of the value to retrieve.
    /// 
    /// # Returns
    /// 
    /// * `Option<&String>` - The value at the specified index, or None if the index is out of bounds.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use hotaru_core::http::meta::HeaderValue;
    /// let mut header = HeaderValue::new("text/html");
    /// assert_eq!(header.try_get(0), Some(&"text/html".to_string()));
    /// assert_eq!(header.try_get(1), None);
    /// 
    /// header.append("application/json");
    /// assert_eq!(header.try_get(1), Some(&"application/json".to_string()));
    /// ```
    pub fn try_get(&self, index: usize) -> Option<&String> {
        match self {
            HeaderValue::Single(s) if index == 0 => Some(s),
            HeaderValue::Single(_) => None,
            HeaderValue::Multiple(v) => v.get(index),
        }
    }

    /// Gets a value at the specified index, or returns an empty string if the index is out of bounds.
    /// 
    /// # Arguments
    /// 
    /// * `index` - The index of the value to retrieve.
    /// 
    /// # Returns
    /// 
    /// The string at the specified index, or an empty string if the index is out of bounds.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use hotaru_core::http::meta::HeaderValue;
    /// let header = HeaderValue::new("text/html");
    /// assert_eq!(header.get(0), "text/html");
    /// assert_eq!(header.get(1), ""); // Out of bounds returns empty string
    /// ```
    pub fn get(&self, index: usize) -> String {
        self.try_get(index).cloned().unwrap_or_default()
    }

    /// Gets a value at the specified index, or returns the provided default if the index is out of bounds.
    /// 
    /// # Arguments
    /// 
    /// * `index` - The index of the value to retrieve.
    /// * `default` - The default value to return if the index is out of bounds.
    /// 
    /// # Returns
    /// 
    /// The string at the specified index, or the default if the index is out of bounds.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use hotaru_core::http::meta::HeaderValue;
    /// let header = HeaderValue::new("text/html");
    /// assert_eq!(header.get_or(0, "default"), "text/html");
    /// assert_eq!(header.get_or(1, "default"), "default"); // Out of bounds returns default
    /// ```
    pub fn get_or<S: Into<String>>(&self, index: usize, default: S) -> String {
        self.try_get(index).cloned().unwrap_or_else(|| default.into())
    }

    /// Add a value to the header without combining it with existing values.
    /// 
    /// This is useful for headers like Set-Cookie where each value should be treated
    /// as a separate header instance rather than being combined with commas.
    /// 
    /// # Arguments
    /// 
    /// * `value` - The value to add.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use hotaru_core::http::meta::HeaderValue;
    /// let mut cookies = HeaderValue::new("sessionId=abc123; Path=/");
    /// cookies.add_without_combining("theme=dark; Path=/; Max-Age=3600");
    /// 
    /// // Each cookie is kept as a separate value
    /// assert_eq!(cookies.try_get(0), Some(&"sessionId=abc123; Path=/".to_string()));
    /// assert_eq!(cookies.try_get(1), Some(&"theme=dark; Path=/; Max-Age=3600".to_string()));
    /// 
    /// // When we use as_str() they'll still be combined with commas for API consistency
    /// // but should be treated separately when used with headers like Set-Cookie
    /// ```
    pub fn add_without_combining<T: Into<String>>(&mut self, value: T) {
        match self {
            HeaderValue::Single(_) => {
                let original = std::mem::replace(self, HeaderValue::Multiple(Vec::new()));
                if let HeaderValue::Single(s) = original {
                    *self = HeaderValue::Multiple(vec![s, value.into()]);
                }
            }
            HeaderValue::Multiple(v) => v.push(value.into()),
        }
    }

    /// Attempts to get the first value in this HeaderValue.
    /// 
    /// # Returns
    /// 
    /// * `Option<&String>` - The first value, or None if there are no values.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use hotaru_core::http::meta::HeaderValue;
    /// let mut header = HeaderValue::new("text/html");
    /// header.append("application/json");
    /// assert_eq!(header.try_first(), Some(&"text/html".to_string()));
    /// ```
    pub fn try_first(&self) -> Option<&String> {
        match self {
            HeaderValue::Single(value) => Some(value),
            HeaderValue::Multiple(values) if !values.is_empty() => Some(&values[0]),
            _ => None,
        }
    }

    /// Gets the first value in this HeaderValue, or an empty string if there are no values.
    /// 
    /// # Returns
    /// 
    /// The first value, or an empty string if there are no values.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use hotaru_core::http::meta::HeaderValue;
    /// let header = HeaderValue::new("text/html");
    /// assert_eq!(header.first(), "text/html");
    /// 
    /// let empty: HeaderValue = HeaderValue::Multiple(vec![]);
    /// assert_eq!(empty.first(), "");
    /// ```
    pub fn first(&self) -> String {
        self.try_first().cloned().unwrap_or_default()
    }

    /// Gets the first value in this HeaderValue, or the provided default if there are no values.
    /// 
    /// # Arguments
    /// 
    /// * `default` - The default value to return if there are no values.
    /// 
    /// # Returns
    /// 
    /// The first value, or the default if there are no values.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use hotaru_core::http::meta::HeaderValue;
    /// let header = HeaderValue::new("text/html");
    /// assert_eq!(header.first_or("default"), "text/html");
    /// 
    /// let empty: HeaderValue = HeaderValue::Multiple(vec![]);
    /// assert_eq!(empty.first_or("default"), "default");
    /// ```
    pub fn first_or<S: Into<String>>(&self, default: S) -> String {
        self.try_first().cloned().unwrap_or_else(|| default.into())
    }

    /// Gets all values as a vector of string references.
    /// 
    /// # Returns
    /// 
    /// A vector containing references to all values.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use hotaru_core::http::meta::HeaderValue;
    /// let mut header = HeaderValue::new("text/html");
    /// header.append("application/json");
    /// 
    /// let values = header.values();
    /// assert_eq!(values.len(), 2);
    /// assert_eq!(values[0], &"text/html".to_string());
    /// assert_eq!(values[1], &"application/json".to_string());
    /// ```
    pub fn values(&self) -> Vec<&String> {
        match self {
            HeaderValue::Single(value) => vec![value],
            HeaderValue::Multiple(values) => values.iter().collect(),
        }
    } 

    /// Converts the HeaderValue into a string suitable f or use in HTTP headers. 
    /// This method formats the header value according to HTTP standards, ensuring 
    /// that single values are represented as a single line and multiple values are 
    /// each represented on their own line. 
    /// 
    /// # Arguments 
    /// * `header_name` - The name of the header to use in the formatted string. 
    /// 
    /// # Returns 
    /// A string formatted as an HTTP header line or lines, ready to be sent in a request or response. 
    /// 
    /// # Examples 
    /// ```rust 
    /// use hotaru_core::http::meta::HeaderValue; 
    /// let header_value = HeaderValue::new("text/html"); 
    /// let header_string = header_value.into_header_string("Content-Type"); 
    /// assert_eq!(header_string, "Content-Type: text/html\r\n"); 
    /// let mut multi_header = HeaderValue::new("text/html"); 
    /// multi_header.append("application/json"); 
    /// let multi_header_string = multi_header.into_header_string("Accept"); 
    /// assert_eq!(multi_header_string, "Accept: text/html\r\nAccept: application/json\r\n"); 
    /// ``` 
    pub fn into_header_string(&self, header_name: &str) -> String {
        match self {
            HeaderValue::Single(v) => {
                // Single values get a single header line
                format!("{}: {}\r\n", header_name, v)
            },
            HeaderValue::Multiple(values) => {
                // Multiple values each get their own header line
                let mut result = String::new(); 
                for v in values {
                    result.push_str(&format!("{}: {}\r\n", header_name, v));
                } 
                result 
            }
        }
    }
}

/// Implements conversion from a string to HeaderValue.
///
/// This enables more ergonomic creation of HeaderValue instances.
///
/// # Examples
/// 
/// ```rust
/// use hotaru_core::http::meta::HeaderValue;
/// let header: HeaderValue = "text/html".to_string().into();
/// assert_eq!(header.first(), "text/html");
/// ```
impl From<String> for HeaderValue {
    fn from(value: String) -> Self {
        HeaderValue::new(value)
    }
}

/// Implements conversion from a string slice to HeaderValue.
///
/// This enables more ergonomic creation of HeaderValue instances.
///
/// # Examples
/// 
/// ```rust
/// use hotaru_core::http::meta::HeaderValue;
/// let header: HeaderValue = "text/html".into();
/// assert_eq!(header.first(), "text/html");
/// ```
impl From<&str> for HeaderValue {
    fn from(value: &str) -> Self {
        HeaderValue::new(value.to_string())
    }
}

/// Implements iterator for HeaderValue to easily iterate over all values.
///
/// # Examples
/// 
/// ```rust
/// use hotaru_core::http::meta::HeaderValue;
/// let mut header = HeaderValue::new("text/html");
/// header.append("application/json");
/// 
/// let mut values = Vec::new();
/// for value in header {
///     values.push(value);
/// }
/// assert_eq!(values, vec!["text/html", "application/json"]);
/// ```
impl IntoIterator for HeaderValue {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            HeaderValue::Single(s) => vec![s].into_iter(),
            HeaderValue::Multiple(v) => v.into_iter(),
        }
    }
}

/// Implements conversion from HeaderValue to a vector of strings.
///
/// # Examples
/// 
/// ```rust
/// use hotaru_core::http::meta::HeaderValue;
/// let mut header = HeaderValue::new("text/html");
/// header.append("application/json");
/// 
/// let values: Vec<String> = header.into();
/// assert_eq!(values, vec!["text/html", "application/json"]);
/// ```
impl From<HeaderValue> for Vec<String> {
    fn from(header_value: HeaderValue) -> Self {
        match header_value {
            HeaderValue::Single(s) => vec![s],
            HeaderValue::Multiple(v) => v,
        }
    }
}

/// Implements conversion from HeaderValue to a string.
///
/// Multiple values are joined with commas and spaces.
///
/// # Examples
/// 
/// ```rust
/// use hotaru_core::http::meta::HeaderValue;
/// let mut header = HeaderValue::new("text/html");
/// header.append("application/json");
/// 
/// let value: String = header.into();
/// assert_eq!(value, "text/html, application/json");
/// ```
impl From<HeaderValue> for String {
    fn from(header_value: HeaderValue) -> Self {
        match header_value {
            HeaderValue::Single(s) => s,
            HeaderValue::Multiple(v) => v.join(", "),
        }
    }
} 

impl HttpMeta { 
    /// It is used to create a new RequestHeader object.
    pub fn new(
        start_line: HttpStartLine, 
        headers: HashMap<String, HeaderValue> 
    ) -> Self {
        Self { 
            start_line, 
            header: headers,
            content_type: None,
            content_length: None,
            content_disposition: None, 
            cookies: None, 
            encoding: None, 
            host: None, 
            lang: None, 
            location: None, 
        }
    } 

    pub async fn from_stream<R: AsyncBufRead + Unpin>(
        buf_reader: &mut R,
        config: &HttpSafety,
        print_raw: bool,
        is_request: bool,
    ) -> Result<HttpMeta, ConnectionError> {
        let mut headers = Self::header_lines_raw_from_stream(
            buf_reader, 
            config, 
            print_raw).await.map_err(|_| ConnectionError::BadRequest(
                format!("Failed to read headers")
            ))?;     

        if headers.is_empty() {
            return Err(ConnectionError::BadRequest(
                format!("Empty {}", if is_request { "request" } else { "response" })
            ));
        }
        
        // Parse the start line according to whether it's a request or response
        let start_line = Self::parse_start_line(&headers.remove(0), is_request);
        
        // Parse headers with special handling for specific header names
        let header = Self::parse_headers(headers, is_request);
        
        if print_raw {
            println!("Parsed headers: {:?}", header);
            println!("Parsed start line: {:?}", start_line);
        }
        
        Ok(HttpMeta::new(start_line, header))
    } 

    async fn header_lines_raw_from_stream<R: AsyncBufRead + Unpin>(
        buf_reader: &mut R,
        config: &HttpSafety,
        print_raw: bool,
    ) -> Result<Vec<String>, ConnectionError> { 
        let mut headers = Vec::new();
        let mut total_header_size = 0;

        // Try to fill the buffer with a single read first
        let buffer = buf_reader.fill_buf().await.map_err(|_| ConnectionError::InternalServerError(
            format!("Failed to fill buffer")
        ))?;

        // Fast path: Check if we got all headers in one go
        // Extract result first, then drop buffer borrow before calling consume
        let fast_path_result = Self::extract_headers_from_buffer(buffer, config);

        if let Some((header_lines, headers_end)) = fast_path_result {
            // We found the complete headers in the buffer
            if print_raw {
                println!("Fast path: got all headers in single read");
            }
            
            // Process headers from buffer
            for line in header_lines {
                if !config.check_line_length(line.len()) {
                    return Err(ConnectionError::BadRequest(
                        format!("Header line too long")
                    ));
                }
                
                total_header_size += line.len() + 2; // +2 for CRLF 

                if !config.check_header_size(total_header_size) {
                    return Err(ConnectionError::BadRequest(
                        format!("Headers too large")
                    ));
                }
                
                if !config.check_headers_count(headers.len()) {
                    return Err(ConnectionError::BadRequest(
                        format!("Too many headers")
                    ));
                }
                
                // Strip CRLF injection and store
                let safe_line = line.replace("\r", "");
                headers.push(safe_line);
            }
            
            // Consume the processed data from the buffer
            buf_reader.consume(headers_end);
        } else {
            // Slow path: read headers line by line as before
            if print_raw {
                println!("Slow path: reading headers line by line");
            }
            
            loop {  
                let mut line = String::new();
                let bytes_read = buf_reader.read_line(&mut line).await.map_err(|_| ConnectionError::InternalServerError(
                    format!("Failed to read line")
                ))?;
                if print_raw {
                    println!("Read line: {}, buffer: {}", line, bytes_read);
                }
                
                if bytes_read == 0 || line.trim_end().is_empty() {
                    // println!("[End of headers] No more lines to read, 0 bytes read {}, empty line: {}", bytes_read, line.trim_end().is_empty()); 
                    break; // End of headers
                }
                
                // Reject with an extremely long header line
                if  !config.check_line_length(line.len()) {
                    // println!("[Header line too long] Rejecting line: {}", line); 
                    return Err(ConnectionError::PayloadTooLarge);
                } 
                
                total_header_size += line.len();
                
                // Enforce max header size limit
                if !config.check_header_size(total_header_size) {
                    // println!("[Headers too large] Total header size: {}, allowed: {}", total_header_size, config.effective_header_size()); 
                    return Err(ConnectionError::PayloadTooLarge);
                }
                
                // Enforce max number of headers
                if !config.check_headers_count(headers.len()) {
                    // println!("[Too many headers] Current header count: {}", headers.len()); 
                    return Err(ConnectionError::PayloadTooLarge);
                }
                
                // Strip CRLF injection and store the header
                let safe_line = line.trim_end().replace("\r", "");
                headers.push(safe_line);
            } 
        }
        
        Ok(headers) 
    }
    
    // Helper function to parse the start line
    fn parse_start_line(line: &str, is_request: bool) -> HttpStartLine {
        if is_request {
            HttpStartLine::parse_request(line)
        } else {
            HttpStartLine::parse_response(line)
        }
    }
    
    // Helper function to parse headers with special handling for specific header types
    fn parse_headers(header_lines: Vec<String>, _is_response: bool) -> HashMap<String, HeaderValue> {
        let mut headers: HashMap<String, HeaderValue> = HashMap::new();
        
        // // List of headers that should not be combined (kept as separate values)
        // // This is especially important for responses with multiple Set-Cookie headers
        // let non_combinable_headers: HashSet<&str> = [
        //     "set-cookie",
        //     // Add other headers that should not be combined if needed 
        // ].iter().cloned().collect();
        
        for line in header_lines {
            if let Some(colon_pos) = line.find(':') {
                let (key, value) = line.split_at(colon_pos);
                
                // Normalize the header name (case-insensitive in HTTP)
                let header_name = key.trim().to_lowercase();
                
                // Remove the colon and trim whitespace from the value
                let header_value = value[1..].trim().to_string();
                
                // Check if this is a special header that should not be combined
                // let is_non_combinable = is_response && non_combinable_headers.contains(header_name.as_str());
                
                match headers.get_mut(&header_name) {
                    Some(existing_value) => { 
                        existing_value.add_without_combining(header_value);  
                        // For special headers like Set-Cookie, add without combining
                        // if is_non_combinable {
                        //     existing_value.add_without_combining(header_value);
                        // } else {
                        //     // For regular headers, append (typically combined with commas)
                        //     existing_value.append(header_value);
                        // }
                    }
                    None => {
                        // First occurrence of this header
                        headers.insert(header_name, HeaderValue::new(header_value));
                    }
                }
            }
        }
        
        headers
    }
    
    // Expose the specific methods that call the shared implementation
    pub async fn from_request_stream<R: AsyncBufRead + Unpin>(
        buf_reader: &mut R,
        config: &HttpSafety,
        print_raw: bool,
    ) -> Result<HttpMeta, ConnectionError> {
        Self::from_stream(buf_reader, config, print_raw, true).await
    } 

    pub async fn append_from_request_stream<R: AsyncBufRead + Unpin>(
        &mut self,
        buf_reader: &mut R,
        config: &HttpSafety,
        print_raw: bool,
    ) -> Result<(), ConnectionError> {
        let mut headers = Self::header_lines_raw_from_stream(buf_reader, config, print_raw).await?;
        
        if headers.is_empty() {
            return Ok(()); 
        }
        
        // Parse the start line
        let start_line = Self::parse_start_line(&headers.remove(0), true);
        
        // Parse headers
        let header = Self::parse_headers(headers, true);
        
        if print_raw {
            println!("Parsed request headers: {:?}", header);
            println!("Parsed request start line: {:?}", start_line);
        }
        
        self.start_line = start_line;
        self.header.extend(header);
        
        Ok(()) 
    } 
    
    pub async fn from_response_stream<R: AsyncBufRead + Unpin>(
        buf_reader: &mut R,
        config: &HttpSafety,
        print_raw: bool,
    ) -> Result<HttpMeta, ConnectionError> {
        Self::from_stream(buf_reader, config, print_raw, false).await
    }  
    
    /// Helper function to extract complete headers from a buffer if possible
    fn extract_headers_from_buffer<'a>(buffer: &'a [u8], config: &HttpSafety) -> Option<(Vec<&'a str>, usize)> {
        // Look for the end of headers marker (double CRLF)
        let mut i = 0;
        while i + 3 < buffer.len() {
            if buffer[i] == b'\r' && buffer[i+1] == b'\n' && 
            buffer[i+2] == b'\r' && buffer[i+3] == b'\n' {
                
                // Found end of headers
                let headers_section = std::str::from_utf8(&buffer[..i+2]).ok()?;
                
                // Split into lines
                let lines: Vec<&str> = headers_section
                    .split("\r\n")
                    .filter(|s| !s.is_empty())
                    .collect();
                    
                if !config.check_headers_count(lines.len()) {
                    return None; // Too many headers, fall back to slow path
                }
                
                return Some((lines, i + 4)); // +4 to include the final \r\n\r\n
            }
            i += 1;
        }
        
        None // Didn't find complete headers
    }    

    pub fn set_header_hashmap(&mut self, header: HashMap<String, HeaderValue>) {
        self.header = header;
    } 

    /// Returns the hashed, unparsed header. 
    /// Note this reference is not intended for you to mutate. 
    /// If yo do want to mutate, please use .set_attribute() method 
    pub fn get_header_hashmap(&self) -> &HashMap<String, HeaderValue> { 
        &self.header 
    } 

    pub fn get_header<T: Into<String>>(&self, key: T) -> Option<String> { 
        self.header.get(&key.into().trim().to_lowercase()).and_then(|v| 
            Some(v.as_str()) 
        ) 
    } 

    /// 
    pub fn set_attribute<T: Into<String>, S: Into<HeaderValue>>(&mut self, key: T, value: S) { 
        self.header.insert(key.into().trim().to_lowercase(), value.into()); 
    } 

    pub fn get_path(&mut self, part: usize) -> String {
        self.start_line.get_url().url_part(part)
    }

    pub fn url(&self) -> String {
        self.start_line.path() 
    } 

    pub fn path(&self) -> String {
        // Return the path part of the URL, removing the query string if present 
        self.start_line.path().split('?').next().unwrap_or("").to_string() 
    } 

    pub fn get_url_args<T: Into<String>>(&mut self, key: T) -> Option<String> {
        self.start_line.get_url().get_url_args(&key.into())
    } 

    pub fn method(&self) -> HttpMethod {
        self.start_line.method() 
    } 

    /// Gets the content length from the HTTP meta data.
    ///
    /// Returns the cached content length if available, otherwise parses
    /// the content-length header from the headers map.
    ///
    /// # Returns
    ///
    /// * `Option<usize>` - The content length, or None if not available or invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("content-length".to_string(), HeaderValue::new("123"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// assert_eq!(meta.get_content_length(), Some(123));
    /// ``` 
    pub fn get_content_length(&mut self) -> Option<usize> {
        if let Some(length) = self.content_length {
            return Some(length);
        }
        self.parse_content_length()
    } 

    /// Parses the Content-Length header from the headers map and stores it in the content_length field.
    ///
    /// # Returns
    ///
    /// * `Option<usize>` - The parsed Content-Length value, or None if not present or invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::meta::HeaderValue;
    /// use std::collections::HashMap;
    /// 
    /// let mut headers = HashMap::new();
    /// headers.insert("content-length".to_string(), HeaderValue::new("123"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// let length = meta.parse_content_length();
    /// assert_eq!(length, Some(123));
    /// assert_eq!(meta.get_content_length(), Some(123));
    /// ``` 
    pub fn parse_content_length(&mut self) -> Option<usize> {
        let length = self
            .header
            .get("content-length")
            .and_then(|s| s.first().parse::<usize>().ok()); 
        self.content_length = length;
        length 
    }

    /// Sets the content_length field.
    ///
    /// # Arguments
    ///
    /// * `length` - The content length to set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// 
    /// let mut meta = HttpMeta::default();
    /// meta.set_content_length(456);
    /// 
    /// assert_eq!(meta.get_content_length(), Some(456));
    /// ```
    pub fn set_content_length(&mut self, length: usize) {
        self.content_length = Some(length);
    }  

    /// Clears the cached content_length field without modifying the header map.
    ///
    /// Note that it will **NOT** clear the value in the HashMap.
    /// To remove both the cached field and the header, use `delete_content_length()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::meta::HeaderValue;
    /// 
    /// let mut meta = HttpMeta::default();
    /// meta.set_content_length(123);
    /// meta.clear_content_length();
    /// 
    /// // The content-length header in the HashMap is still intact
    /// // but the cached value is cleared
    /// ```
    pub fn clear_content_length(&mut self) {
        self.content_length = None;
    } 

    /// Deletes the Content-Length header completely, clearing both the cached field
    /// and removing it from the header map.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::meta::HeaderValue;
    /// 
    /// let mut meta = HttpMeta::default();
    /// meta.set_header("content-length", "123");
    /// meta.delete_content_length();
    /// 
    /// // Both the cached field and the header are now removed
    /// assert!(meta.get_header("content-length").is_none());
    /// ```
    pub fn delete_content_length(&mut self) {
        self.content_length = None;
        self.header.remove("content-length");
    }

    /// Gets the content type from the HTTP meta data.
    ///
    /// Returns the cached content type if available, otherwise parses
    /// the content-type header from the headers map.
    ///
    /// # Returns
    ///
    /// * `Option<HttpContentType>` - The content type, or None if not available.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::meta::HeaderValue;
    /// use hotaru_core::http::http_value::HttpContentType;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("content-type".to_string(), HeaderValue::new("text/html"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// assert_eq!(meta.get_content_type(), Some(HttpContentType::TextHtml));
    /// ``` 
    pub fn get_content_type(&mut self) -> Option<HttpContentType> {
        if let Some(ref content_type) = self.content_type {
            return Some(content_type.clone());
        }
        self.parse_content_type()
    } 

    /// Parses the Content-Type header from the headers map and stores it in the content_type field.
    ///
    /// # Returns
    ///
    /// * `Option<HttpContentType>` - The parsed Content-Type value, or None if not present.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::meta::HeaderValue;
    /// use hotaru_core::http::http_value::HttpContentType;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("content-type".to_string(), HeaderValue::new("text/html"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// let content_type = meta.parse_content_type();
    /// assert_eq!(content_type, Some(HttpContentType::TextHtml));
    /// ``` 
    pub fn parse_content_type(&mut self) -> Option<HttpContentType> {
        // Try lowercase first, then uppercase for backward compatibility
        let content_type_str = self.header
            .get("content-type")
            .map(|value| value.first())?;

        let content_type = HttpContentType::from_str(&content_type_str);
        self.set_content_type(content_type.clone());
        Some(content_type)
    } 

    /// Sets the content_type field.
    ///
    /// # Arguments
    ///
    /// * `content_type` - The content type to set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::http_value::HttpContentType;
    /// 
    /// let mut meta = HttpMeta::default();
    /// meta.set_content_type(HttpContentType::ApplicationJson);
    /// 
    /// assert_eq!(meta.get_content_type(), Some(HttpContentType::ApplicationJson));
    /// ```
    pub fn set_content_type(&mut self, content_type: HttpContentType) {
        self.content_type = Some(content_type);
    } 

    /// Clears the cached content_type field without modifying the header map.
    ///
    /// This method invalidates the cached content_type value, which will cause
    /// subsequent calls to `get_content_type()` to re-parse the value from the
    /// headers map.
    ///
    /// Note that it will **NOT** clear the value in the headers map.
    /// To remove both the cached field and the header, use `delete_content_type()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::meta::HeaderValue;
    /// use hotaru_core::http::http_value::HttpContentType;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("content-type".to_string(), HeaderValue::new("text/html"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// // Parse the value into the cache
    /// let content_type = meta.get_content_type();
    /// assert_eq!(content_type, Some(HttpContentType::TextHtml));
    /// 
    /// // Clear the cache only
    /// meta.clear_content_type();
    /// 
    /// // The header is still intact and will be re-parsed
    /// assert_eq!(meta.get_content_type(), Some(HttpContentType::TextHtml));
    /// ```
    pub fn clear_content_type(&mut self) {
        self.content_type = None;
    }

    /// Deletes the Content-Type header completely, clearing both the cached field
    /// and removing it from the header map.
    ///
    /// This method removes the content-type header from the headers map and
    /// clears the cached content_type value. Subsequent calls to `get_content_type()`
    /// will return a default value unless a new content-type is set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::meta::HeaderValue;
    /// use hotaru_core::http::http_value::HttpContentType;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("content-type".to_string(), HeaderValue::new("text/html"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// // Delete both the cache and header
    /// meta.delete_content_type();
    /// 
    /// // The header is gone
    /// assert!(meta.get_header("content-type").is_none());
    /// 
    /// // And get_content_type will now return a default value
    /// assert_eq!(meta.get_content_type().unwrap(), HttpContentType::from_str(""));
    /// ```
    pub fn delete_content_type(&mut self) {
        self.content_type = None;
        self.header.remove("content-type");
    } 

    /// Gets the Content-Disposition header value from the HTTP metadata.
    ///
    /// This method returns the cached Content-Disposition value if available.
    /// If not cached, it parses the "Content-Disposition" header from the headers map.
    ///
    /// # Returns
    ///
    /// * `Option<ContentDisposition>` - The parsed Content-Disposition value, or None if not present.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::meta::HeaderValue;
    /// use hotaru_core::http::http_value::ContentDisposition;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert(
    ///     "content-disposition".to_string(),
    ///     HeaderValue::new("attachment; filename=\"report.pdf\"")
    /// );
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// let content_disp = meta.get_content_disposition();
    /// assert!(content_disp.is_some());
    /// assert_eq!(content_disp.unwrap().filename().unwrap(), "report.pdf");
    /// ``` 
    pub fn get_content_disposition(&mut self) -> Option<ContentDisposition> {
        if let Some(ref content_disposition) = self.content_disposition {
            return Some(content_disposition.clone());
        }
        self.parse_content_disposition()
    } 

    /// Parses the Content-Disposition header from the headers map and stores it in the content_disposition field.
    ///
    /// # Returns
    ///
    /// * `Option<ContentDisposition>` - The parsed Content-Disposition value, or None if not present.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::meta::HeaderValue;
    /// use hotaru_core::http::http_value::ContentDisposition;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert(
    ///     "content-disposition".to_string(),
    ///     HeaderValue::new("form-data; name=\"file\"; filename=\"data.txt\"")
    /// );
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// let content_disp = meta.parse_content_disposition();
    /// assert!(content_disp.is_some());
    /// assert_eq!(content_disp.unwrap().filename().unwrap(), "data.txt");
    /// ```
    pub fn parse_content_disposition(&mut self) -> Option<ContentDisposition> {
        let content_disposition = self.header
            .get("content-disposition")
            .and_then(|s| ContentDisposition::parse(&s.first()).ok());
        
        if let Some(ref cd) = content_disposition {
            self.content_disposition = Some(cd.clone()); 
           
        }
        content_disposition
    } 

    /// Sets the content_disposition field.
    ///
    /// # Arguments
    ///
    /// * `content_disposition` - The Content-Disposition value to set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::http_value::{ContentDisposition, ContentDispositionType};
    /// 
    /// let mut meta = HttpMeta::default();
    /// let cd = ContentDisposition::attachment("report.pdf");
    /// meta.set_content_disposition(cd.clone());
    /// 
    /// assert_eq!(meta.get_content_disposition(), Some(cd));
    /// ```
    pub fn set_content_disposition(&mut self, content_disposition: ContentDisposition) {
        self.content_disposition = Some(content_disposition);
    } 

    /// Clears the cached content_disposition field without modifying the header map.
    ///
    /// This method invalidates the cached Content-Disposition value, which will cause
    /// subsequent calls to `get_content_disposition()` to re-parse the value from the
    /// headers map.
    ///
    /// Note that it will **NOT** clear the value in the headers map.
    /// To remove both the cached field and the header, use `delete_content_disposition()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::meta::HeaderValue;
    /// use hotaru_core::http::http_value::ContentDisposition;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert(
    ///     "content-disposition".to_string(),
    ///     HeaderValue::new("inline; filename=\"image.jpg\"")
    /// );
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// // Parse the value into the cache
    /// let content_disp = meta.get_content_disposition();
    /// assert!(content_disp.is_some());
    /// 
    /// // Clear the cache only
    /// meta.clear_content_disposition();
    /// 
    /// // The header is still intact and will be re-parsed
    /// assert!(meta.get_content_disposition().is_some());
    /// ```
    pub fn clear_content_disposition(&mut self) {
        self.content_disposition = None;
    } 

    /// Deletes the Content-Disposition header completely, clearing both the cached field
    /// and removing it from the header map.
    ///
    /// This method removes the content-disposition header from the headers map and
    /// clears the cached content_disposition value. Subsequent calls to `get_content_disposition()`
    /// will return None unless a new header is set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::meta::HeaderValue;
    /// use hotaru_core::http::http_value::ContentDisposition;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert(
    ///     "content-disposition".to_string(),
    ///     HeaderValue::new("attachment; filename=\"data.zip\"")
    /// );
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// // Delete both the cache and header
    /// meta.delete_content_disposition();
    /// 
    /// // The header is gone
    /// assert!(meta.get_header("content-disposition").is_none());
    /// 
    /// // And get_content_disposition will now return None
    /// assert!(meta.get_content_disposition().is_none());
    /// ```
    pub fn delete_content_disposition(&mut self) {
        self.content_disposition = None;
        self.header.remove("content-disposition");
    } 

    /// Gets the cookies from the HTTP meta data.
    ///
    /// Returns the cached cookies if available, otherwise parses
    /// the cookie header from the headers map.
    ///
    /// # Returns
    ///
    /// A reference to the cookies map.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::meta::HeaderValue;
    /// use hotaru_core::http::cookie::{Cookie, CookieMap};
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("cookie".to_string(), HeaderValue::new("sessionId=abc123; theme=dark"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// let cookies = meta.get_cookies();
    /// assert_eq!(cookies.get("sessionId").unwrap().get_value(), "abc123");
    /// assert_eq!(cookies.get("theme").unwrap().get_value(), "dark");
    /// ```
    pub fn get_cookies(&mut self) -> &CookieMap {
        if self.cookies.is_none() {
            self.cookies = Some(self.parse_cookies());
        }
        // Safety: unwrap() is safe here because we ensure cookies is Some() in the lines above.
        // This pattern guarantees cookies.is_some() before calling unwrap().
        self.cookies.as_ref().unwrap()
    } 

    /// Gets a specific cookie by key.
    ///
    /// If the cookies haven't been parsed yet, parses them from the headers map.
    ///
    /// # Arguments
    ///
    /// * `key` - The cookie key to look up.
    ///
    /// # Returns
    ///
    /// * `Option<Cookie>` - The cookie if found, or None.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("cookie".to_string(), HeaderValue::new("sessionId=abc123; theme=dark"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// let session_cookie = meta.get_cookie("sessionId");
    /// let theme_cookie = meta.get_cookie("theme");
    /// assert_eq!(session_cookie.unwrap().get_value(), "abc123");
    /// assert_eq!(theme_cookie.unwrap().get_value(), "dark");
    /// ```
    pub fn get_cookie<T: AsRef<str>>(&mut self, key: T) -> Option<Cookie> {
        if self.cookies.is_none() {
            self.cookies = Some(self.parse_cookies());
        }
        // Safety: unwrap() is safe here because we ensure cookies is Some() in the lines above.
        // This pattern guarantees cookies.is_some() before calling unwrap().
        self.cookies.as_ref().unwrap().get(key).cloned()
    } 

    /// Gets a specific cookie by key, returning a default cookie if not found.
    ///
    /// # Arguments
    ///
    /// * `key` - The cookie key to look up.
    ///
    /// # Returns
    ///
    /// The cookie if found, or a default empty cookie.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("cookie".to_string(), HeaderValue::new("sessionId=abc123"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// // Existing cookie
    /// let session_cookie = meta.get_cookie_or_default("sessionId");
    /// assert_eq!(session_cookie.get_value(), "abc123");
    /// 
    /// // Non-existent cookie returns default
    /// let nonexistent = meta.get_cookie_or_default("nonexistent");
    /// assert_eq!(nonexistent.get_value(), "");
    /// ```
    pub fn get_cookie_or_default<T: AsRef<str>>(&mut self, key: T) -> Cookie {
        self.get_cookie(key).unwrap_or_else(|| Cookie::new(""))
    } 

    /// Parses cookies from either request Cookie header or response Set-Cookie headers,
    /// depending on the type of HTTP message (request or response).
    ///
    /// # Returns
    ///
    /// A CookieMap containing the parsed cookies.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // For a request with a Cookie header
    /// use hotaru_core::http::meta::{HttpMeta, HeaderValue};
    /// use hotaru_core::http::http_value::HttpStartLine;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("cookie".to_string(), HeaderValue::new("sessionId=abc123; theme=dark"));
    /// let mut meta = HttpMeta::new(HttpStartLine::parse_request("GET / HTTP/1.1"), headers);
    /// 
    /// let cookies = meta.parse_cookies();
    /// assert_eq!(cookies.get("sessionId").unwrap().value, "abc123");
    ///
    /// // For a response with Set-Cookie headers
    /// use hotaru_core::http::meta::{HttpMeta, HeaderValue};
    /// use hotaru_core::http::http_value::HttpStartLine;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("set-cookie".to_string(), HeaderValue::new("sessionId=abc123; Path=/; Secure"));
    /// let mut meta = HttpMeta::new(HttpStartLine::parse_response("HTTP/1.1 200 OK"), headers);
    /// 
    /// let cookies = meta.parse_cookies();
    /// assert_eq!(cookies.get("sessionId").unwrap().value, "abc123");
    /// assert_eq!(cookies.get("sessionId").unwrap().get_path(), Some("/".to_string()));
    /// assert_eq!(cookies.get("sessionId").unwrap().get_secure(), Some(true));
    /// ```
    pub fn parse_cookies(&self) -> CookieMap {
        // Check if this is a request or response
        if self.start_line.is_request() {
            self.parse_request_cookies()
        } else {
            self.parse_response_cookies()
        }
    }
    
    /// Parses cookies from the request Cookie header.
    ///
    /// # Returns
    ///
    /// A CookieMap containing the parsed cookies.
    fn parse_request_cookies(&self) -> CookieMap { 
        let cookie_header = self.header.get("cookie");
        
        match cookie_header {
            Some(header_value) => match header_value {
                HeaderValue::Single(cookie_str) => CookieMap::parse(cookie_str),
                HeaderValue::Multiple(cookie_strs) => {
                    // Combine multiple cookie headers into one map
                    let mut cookie_map = CookieMap::new();
                    for cookie_str in cookie_strs {
                        let parsed = CookieMap::parse(cookie_str);
                        for (k, v) in parsed.0.into_iter() {
                            cookie_map.set(k, v);
                        }
                    }
                    cookie_map
                }
            },
            None => CookieMap::default()
        }
    }

    /// Parses cookies from the response Set-Cookie headers.
    ///
    /// # Returns
    ///
    /// A CookieMap containing the parsed cookies with their attributes.
    fn parse_response_cookies(&self) -> CookieMap {
        let set_cookie_header = self.header.get("set-cookie");
        
        match set_cookie_header {
            Some(HeaderValue::Single(s)) => CookieMap::parse_set_cookies([s.as_str()]),
            Some(HeaderValue::Multiple(v)) => {
                CookieMap::parse_set_cookies(v.iter().map(|s| s.as_str()))
            },
            None => CookieMap::default()
        }
    } 

    pub fn set_cookies(&mut self, cookies: CookieMap) { 
        self.cookies = Some(cookies);
    } 

    /// Add a cookie to the HTTP meta data.
    ///
    /// # Arguments
    ///
    /// * `key` - The cookie key.
    /// * `cookie` - The cookie to add.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::cookie::Cookie;
    /// 
    /// let mut meta = HttpMeta::default();
    /// meta.add_cookie("sessionId", Cookie::new("abc123"));
    /// assert_eq!(meta.get_cookie("sessionId").unwrap().get_value(), "abc123"); 
    /// 
    /// meta.add_cookie("sessionCont", Cookie::new("123"));
    /// assert_eq!(meta.get_cookie("sessionId").unwrap().get_value(), "abc123"); 
    /// ```
    pub fn add_cookie<T: Into<String>>(&mut self, key: T, cookie: Cookie) { 
        if self.cookies.is_none() { 
            self.cookies = Some(CookieMap::new()); 
        }         if let Some(ref mut cookies) = self.cookies { 
            cookies.set(key, cookie); 
        } 
    } 

    /// Clears the cached cookies field without modifying the header map.
    ///
    /// This method invalidates the cached cookies value, which will cause
    /// subsequent calls to `get_cookies()` to re-parse the value from the
    /// headers map.
    ///
    /// Note that it will **NOT** clear the value in the headers map.
    /// To remove both the cached field and the header, use `delete_cookies()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("cookie".to_string(), HeaderValue::new("sessionId=abc123"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// // Parse the value into the cache
    /// let cookies = meta.get_cookies();
    /// assert_eq!(cookies.get("sessionId").unwrap().value(), "abc123");
    /// 
    /// // Clear the cache only
    /// meta.clear_cookies();
    /// 
    /// // The header is still intact and will be re-parsed
    /// assert_eq!(meta.get_cookies().get("sessionId").unwrap().value(), "abc123");
    /// ```
    pub fn clear_cookies(&mut self) {
        self.cookies = None;
    }

    /// Deletes the Cookie header completely, clearing both the cached field
    /// and removing it from the header map.
    ///
    /// This method removes the cookie header from the headers map and
    /// clears the cached cookies value. Subsequent calls to `get_cookies()`
    /// will return an empty cookie map unless new cookies are set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("cookie".to_string(), HeaderValue::new("sessionId=abc123"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// // Delete both the cache and header
    /// meta.delete_cookies();
    /// 
    /// // The header is gone
    /// assert!(meta.get_header("cookie").is_none());
    /// 
    /// // And get_cookies will now return an empty map
    /// assert!(meta.get_cookies().is_empty());
    /// ```
    pub fn delete_cookies(&mut self) {
        self.cookies = None;
        self.header.remove("cookie"); 
        self.header.remove("set-cookie"); 
    } 

    /// Gets the host from the HTTP meta data.
    ///
    /// Returns the cached host if available, otherwise parses
    /// the host header from the headers map.
    ///
    /// # Returns
    ///
    /// * `Option<String>` - The host, or None if not available.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("host".to_string(), HeaderValue::new("example.com"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// assert_eq!(meta.get_host(), Some("example.com".to_string()));
    /// ``` 
    pub fn get_host(&mut self) -> Option<String> {
        if let Some(ref host) = self.host {
            return Some(host.clone());
        }
        self.parse_host()
    } 

    /// Parses the Host header from the headers map and stores it in the host field.
    ///
    /// # Returns
    ///
    /// * `Option<String>` - The host value, or None if not present.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::meta::HeaderValue;
    /// use std::collections::HashMap;
    /// 
    /// let mut headers = HashMap::new();
    /// headers.insert("host".to_string(), HeaderValue::new("example.com"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    ///
    /// let host = meta.parse_host();
    /// assert_eq!(host, Some("example.com".to_string()));
    /// ``` 
    pub fn parse_host(&mut self) -> Option<String> {
        let host = self.header
            .get("host") 
            .map(|value| value.first());
        
        self.set_host(host.clone());
        host
    } 

    /// Sets the host field. 
    /// 
    /// # Arguments
    /// 
    /// * `host` - The host to set.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta; 
    /// 
    /// let mut meta = HttpMeta::default();
    /// meta.set_host(Some("example.com".to_string()));
    /// 
    /// assert_eq!(meta.get_host(), Some("example.com".to_string())); 
    /// ```
    pub fn set_host(&mut self, host: Option<String>) {
        self.host = host;
    } 

    /// Clears the cached host field without modifying the header map. 
    /// 
    /// This method invalidates the cached host value, which will cause
    /// subsequent calls to `get_host()` to re-parse the value from the 
    /// headers map. 
    /// 
    /// Note that it will **NOT** clear the value in the headers map.,
    /// To remove both the cached field and the header, use `delete_host()`.
    /// 
    /// # Examples
    /// 
    /// ```rust 
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::meta::HeaderValue;
    /// use std::collections::HashMap;
    /// 
    /// let mut headers = HashMap::new();
    /// headers.insert("host".to_string(), HeaderValue::new("example.com"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// // Parse the value into the cache
    /// let host = meta.get_host();
    /// assert_eq!(host, Some("example.com".to_string()));
    /// 
    /// // Clear the cache only
    /// meta.clear_host();
    /// 
    /// // The header is still intact and will be re-parsed
    /// assert_eq!(meta.get_host(), Some("example.com".to_string()));
    /// ``` 
    pub fn clear_host(&mut self) {
        self.host = None;
    } 

    /// Gets the language preference from the HTTP meta data.
    ///
    /// Returns the cached language if available, otherwise parses
    /// the appropriate language header from the headers map.
    ///
    /// # Returns
    ///
    /// * `Option<AcceptLang>` - The language preference, or None if not available.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::meta::HeaderValue;
    /// use hotaru_core::http::start_line::HttpStartLine;
    /// use hotaru_core::http::http_value::*;
    /// use std::collections::HashMap;
    /// let mut headers = HashMap::new();
    /// headers.insert("accept-language".to_string(), HeaderValue::new("en-US, en;q=0.9"));
    /// headers.insert("content-language".to_string(), HeaderValue::new("zh-TW"));
    /// let mut meta = HttpMeta::new(HttpStartLine::new_request(HttpVersion::Http11, HttpMethod::GET, "/".to_string()), headers.clone());
    /// 
    /// let lang = meta.get_lang().unwrap(); 
    /// assert_eq!(lang.most_preferred(), "en-US"); 
    /// 
    /// let mut meta = HttpMeta::new(HttpStartLine::new_response(HttpVersion::Http11, StatusCode::OK), headers);
    /// let lang = meta.get_lang().unwrap(); 
    /// assert_eq!(lang.most_preferred(), "zh-TW"); 
    /// ```
    pub fn get_lang(&mut self) -> Option<AcceptLang> {
        if let Some(ref lang) = self.lang {
            return Some(lang.clone());
        }
        self.parse_lang()
    }

    /// Parses the language header from the headers map and stores it in the lang field.
    ///
    /// For requests: Parses "accept-language" header
    /// For responses: Parses "content-language" header
    ///
    /// # Returns
    ///
    /// * `Option<AcceptLang>` - The parsed language value, or None if not present.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::meta::HeaderValue;
    /// use hotaru_core::http::start_line::HttpStartLine;
    /// use hotaru_core::http::http_value::*;
    /// use std::collections::HashMap;
    /// let mut headers = HashMap::new();
    /// headers.insert("accept-language".to_string(), HeaderValue::new("en-US, en;q=0.9"));
    /// headers.insert("content-language".to_string(), HeaderValue::new("zh-TW"));
    /// let mut meta = HttpMeta::new(HttpStartLine::new_request(HttpVersion::Http11, HttpMethod::GET, "/".to_string()), headers.clone());
    /// 
    /// let lang = meta.parse_lang().unwrap(); 
    /// assert_eq!(lang.most_preferred(), "en-US"); 
    /// 
    /// let mut meta = HttpMeta::new(HttpStartLine::new_response(HttpVersion::Http11, StatusCode::OK), headers);
    /// let lang = meta.parse_lang().unwrap(); 
    /// assert_eq!(lang.most_preferred(), "zh-TW"); 
    /// ```
    pub fn parse_lang(&mut self) -> Option<AcceptLang> {
        let header_name = if self.start_line.is_request() {
            "accept-language"
        } else {
            "content-language"
        };
        
        let lang_str = self.header
            .get(header_name)
            .map(|value| value.as_str()); 
            
        let lang = lang_str.as_ref().map(|s| AcceptLang::from_str(s));
        self.lang = lang.clone();
        lang
    }

    /// Sets the lang field.
    ///
    /// # Arguments
    ///
    /// * `lang` - The language preference to set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::http_value::AcceptLang;
    /// let mut meta = HttpMeta::default();
    /// meta.set_lang(Some(AcceptLang::from_str("en")));
    /// ```
    pub fn set_lang(&mut self, lang: Option<AcceptLang>) {
        self.lang = lang;
    }

    /// Clears the cached lang field without modifying the header map.
    ///
    /// This method invalidates the cached lang value but preserves
    /// the header in the map. Subsequent calls to `get_lang()` will
    /// re-parse the value from headers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// let mut meta = HttpMeta::default();
    /// meta.clear_lang();
    /// ```
    pub fn clear_lang(&mut self) {
        self.lang = None;
    }

    /// Deletes the language header completely, clearing both the cached field
    /// and removing it from the header map.
    ///
    /// For requests: Removes "accept-language" header
    /// For responses: Removes "content-language" header
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// let mut meta = HttpMeta::default();
    /// meta.delete_lang();
    /// ```
    pub fn delete_lang(&mut self) {
        self.lang = None;
        if self.start_line.is_request() {
            self.header.remove("accept-language");
        } else {
            self.header.remove("content-language");
        }
    } 

    /// Deletes the Host header completely, clearing both the cached field 
    /// and removing it from the header map.
    /// 
    /// This method removes the host header from the headers map and
    /// clears the cached host value. Subsequent calls to `get_host()`
    /// will return None unless a new host is set.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::meta::HeaderValue;
    /// use std::collections::HashMap;
    /// 
    /// let mut headers = HashMap::new();
    /// headers.insert("host".to_string(), HeaderValue::new("example.com"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// // Delete both the cache and header
    /// meta.delete_host();
    /// 
    /// // The header is gone
    /// assert!(meta.get_header("host").is_none());
    /// 
    /// // And get_host will now return None
    /// assert_eq!(meta.get_host(), None);
    /// ``` 
    pub fn delete_host(&mut self) {
        self.host = None;
        self.header.remove("host");
    } 

    /// Gets the location header from the HTTP meta data.
    ///
    /// Returns the cached location if available, otherwise parses
    /// the location header from the headers map.
    ///
    /// # Returns
    ///
    /// * `Option<String>` - The location, or None if not available.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("location".to_string(), HeaderValue::new("/redirect"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// assert_eq!(meta.get_location(), Some("/redirect".to_string()));
    /// ```
    pub fn get_location(&mut self) -> Option<String> {
        if let Some(ref loc) = self.location {
            return Some(loc.clone());
        }
        self.parse_location()
    } 

    /// Parses the Location header from the headers map and stores it in the location field.
    ///
    /// # Returns
    ///
    /// * `Option<String>` - The location value, or None if not present.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("location".to_string(), HeaderValue::new("/redirect"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// let location = meta.parse_location();
    /// assert_eq!(location, Some("/redirect".to_string()));
    /// ```
    pub fn parse_location(&mut self) -> Option<String> {
        // Try both lowercase and uppercase for backward compatibility
        let location = self.header
            .get("location") 
            .map(|value| value.first());
        
        self.set_location(location.clone());
        location
    } 

    /// Sets the location field.
    ///
    /// # Arguments
    ///
    /// * `location` - The location to set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// 
    /// let mut meta = HttpMeta::default();
    /// meta.set_location(Some("/redirect".to_string()));
    /// 
    /// assert_eq!(meta.get_location(), Some("/redirect".to_string()));
    /// ```
    pub fn set_location(&mut self, location: Option<String>) {
        self.location = location;
    } 

    /// Clears the cached location field without modifying the header map.
    ///
    /// This method invalidates the cached location value, which will cause
    /// subsequent calls to `get_location()` to re-parse the value from the
    /// headers map.
    ///
    /// Note that it will **NOT** clear the value in the headers map.
    /// To remove both the cached field and the header, use `delete_location()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("location".to_string(), HeaderValue::new("/redirect"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// // Parse the value into the cache
    /// let location = meta.get_location();
    /// assert_eq!(location, Some("/redirect".to_string()));
    /// 
    /// // Clear the cache only
    /// meta.clear_location();
    /// 
    /// // The header is still intact and will be re-parsed
    /// assert_eq!(meta.get_location(), Some("/redirect".to_string()));
    /// ```
    pub fn clear_location(&mut self) {
        self.location = None;
    }

    /// Deletes the Location header completely, clearing both the cached field
    /// and removing it from the header map.
    ///
    /// This method removes the location header from the headers map and
    /// clears the cached location value. Subsequent calls to `get_location()`
    /// will return None unless a new location is set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::meta::HeaderValue;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("location".to_string(), HeaderValue::new("/redirect"));
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// // Delete both the cache and header
    /// meta.delete_location();
    /// 
    /// // The header is gone
    /// assert!(meta.get_header("location").is_none());
    /// 
    /// // And get_location will now return None
    /// assert_eq!(meta.get_location(), None);
    /// ```
    pub fn delete_location(&mut self) {
        self.location = None;
        self.header.remove("location");
    } 

    /// Gets the HTTP encoding (both transfer and content encoding) from the HTTP meta data.
    ///
    /// Returns the cached encoding if available, otherwise parses
    /// the transfer-encoding and content-encoding headers from the headers map.
    ///
    /// # Returns
    ///
    /// * `Option<HttpEncoding>` - The HTTP encodings, or None if not available.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::{HttpMeta, HeaderValue};
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("transfer-encoding".to_string(), vec![HeaderValue::new("chunked")]);
    /// headers.insert("content-encoding".to_string(), vec![HeaderValue::new("gzip")]);
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// let encoding = meta.get_encoding();
    /// assert!(encoding.is_some());
    /// let encoding = encoding.unwrap();
    /// assert!(encoding.transfer().is_chunked());
    /// assert!(!encoding.content().is_identity());
    /// ```
    pub fn get_encoding(&mut self) -> Option<HttpEncoding> {
        if let Some(ref enc) = self.encoding {
            return Some(enc.clone());
        }
        self.parse_encoding()
    }

    /// Parses the Transfer-Encoding and Content-Encoding headers from the headers map 
    /// and stores them in the encoding field.
    ///
    /// # Returns
    ///
    /// * `Option<HttpEncoding>` - The parsed HTTP encodings
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::{HttpMeta, HeaderValue};
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("transfer-encoding".to_string(), vec![HeaderValue::new("chunked")]);
    /// headers.insert("content-encoding".to_string(), vec![HeaderValue::new("br")]);
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// let encoding = meta.parse_encoding();
    /// assert!(encoding.is_some());
    /// let encoding = encoding.unwrap();
    /// assert!(encoding.transfer().is_chunked());
    /// assert_eq!(encoding.content().to_header(), "br");
    /// ```
    pub fn parse_encoding(&mut self) -> Option<HttpEncoding> {
        // Get header values as comma-separated strings
        let transfer_header = self.header
            .get("transfer-encoding")
            .map(|values| 
                values.first() 
            );

        let content_header = self.header
            .get("content-encoding")
            .map(|values| 
                values.first() 
            );

        let encoding = HttpEncoding::from_headers(transfer_header, content_header);
        self.encoding = Some(encoding.clone());
        Some(encoding)
    }

    /// Sets the encoding field with both transfer and content encodings
    ///
    /// # Arguments
    ///
    /// * `encoding` - The HTTP encodings to cache
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::encoding::HttpEncoding;
    /// 
    /// let mut meta = HttpMeta::default();
    /// let encoding = HttpEncoding::from_headers(
    ///     Some("chunked".to_string()),
    ///     Some("gzip".to_string())
    /// );
    /// 
    /// meta.set_encoding(Some(encoding.clone()));
    /// 
    /// assert!(meta.get_encoding().unwrap().transfer().is_chunked());
    /// assert!(!meta.get_encoding().unwrap().content().is_identity());
    /// ```
    pub fn set_encoding(&mut self, encoding: Option<HttpEncoding>) {
        self.encoding = encoding;
    }

    /// Clears the cached encoding field without modifying the header map
    ///
    /// Subsequent calls to `get_encoding()` will re-parse the value from headers
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::{HttpMeta, HeaderValue};
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("transfer-encoding".to_string(), vec![HeaderValue::new("chunked")]);
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// // Parse the value into cache
    /// let _ = meta.get_encoding();
    /// 
    /// // Clear the cache only
    /// meta.clear_encoding();
    /// 
    /// // Header is still intact and will be re-parsed
    /// assert!(meta.get_encoding().is_some());
    /// ```
    pub fn clear_encoding(&mut self) {
        self.encoding = None;
    }

    /// Deletes both Transfer-Encoding and Content-Encoding headers
    ///
    /// Clears both the cached field and removes headers from the map
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::{HttpMeta, HeaderValue};
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("transfer-encoding".to_string(), vec![HeaderValue::new("gzip")]);
    /// headers.insert("content-encoding".to_string(), vec![HeaderValue::new("br")]);
    /// let mut meta = HttpMeta::new(Default::default(), headers);
    /// 
    /// // Delete both cache and headers
    /// meta.delete_encoding();
    /// 
    /// // Headers are gone
    /// assert!(meta.get_header("transfer-encoding").is_none());
    /// assert!(meta.get_header("content-encoding").is_none());
    /// 
    /// // Encoding is now identity
    /// let encoding = meta.get_encoding().unwrap();
    /// assert!(encoding.transfer().is_identity());
    /// assert!(encoding.content().is_identity());
    /// ```
    pub fn delete_encoding(&mut self) {
        self.encoding = None;
        self.header.remove("transfer-encoding");
        self.header.remove("content-encoding");
    }

    /// Serializes the HTTP meta data to a string representation.
    ///
    /// This method generates a properly formatted HTTP header section,
    /// including the start line and all headers.
    ///
    /// # Returns
    ///
    /// A string containing the start line and all headers, formatted
    /// according to the HTTP protocol.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::meta::HttpMeta;
    /// use hotaru_core::http::meta::HeaderValue;
    /// use hotaru_core::http::http_value::{HttpStartLine, HttpVersion, HttpMethod};
    /// use std::collections::HashMap;
    ///
    /// // Create a request meta
    /// let mut meta = HttpMeta::new(
    ///     HttpStartLine::new_request(
    ///         HttpVersion::Http11,
    ///         HttpMethod::GET,
    ///         "/index.html".to_string()
    ///     ),
    ///     HashMap::new()
    /// );
    /// meta.set_header("host", "example.com");
    ///
    /// let http_string = meta.represent();
    /// assert!(http_string.starts_with("GET /index.html HTTP/1.1\r\n"));
    /// assert!(http_string.contains("host: example.com\r\n"));
    /// assert!(http_string.ends_with("\r\n\r\n"));
    /// ```
    pub fn represent(&self) -> String {
        let mut result = String::new();
        let mut handled_headers = HashSet::new();
        
        // Add the start line (works for both request and response)
        result.push_str(&format!("{}\r\n", self.start_line));
        
        // Process field values first (they have priority)
        
        // Add content-type if present
        if let Some(ref content_type) = self.content_type {
            result.push_str(&format!("content-type: {}\r\n", content_type));
            handled_headers.insert("content-type".to_string());
        }
        
        // Add content-length if present
        if let Some(content_length) = self.content_length {
            result.push_str(&format!("content-length: {}\r\n", content_length));
            handled_headers.insert("content-length".to_string());
        } 

        // Add content-disposition if present 
        if let Some(ref content_disposition) = self.content_disposition {
            result.push_str(&format!("content-disposition: {}\r\n", content_disposition.to_string()));
            handled_headers.insert("content-disposition".to_string());
        } 

        // Add host if present 
        if let Some(ref host) = self.host {
            result.push_str(&format!("host: {}\r\n", host));
            handled_headers.insert("host".to_string());
        } 

        // Add language if present 
        if let Some(ref lang) = self.lang { 
            if self.start_line.is_request() { 
                result.push_str(&format!("accept-language: {}\r\n", lang.to_header_string()));
                handled_headers.insert("host".to_string());
            } else { 
                result.push_str(&format!("content-language: {}\r\n", lang.to_response_header()));
                handled_headers.insert("content-language".to_string()); 
            } 
        } 
        
        // Add location if present
        if let Some(ref location) = self.location {
            result.push_str(&format!("location: {}\r\n", location));
            handled_headers.insert("location".to_string());
        } 

        // Add transfer-encoding if present 
        if let Some(ref transfer_encoding) = self.encoding { 
            let (transfer, content)= transfer_encoding.to_headers(); 
            if let Some(transfer) = transfer {
                result.push_str(&format!("transfer-encoding: {}\r\n", transfer));
                handled_headers.insert("transfer-encoding".to_string());
            } 
            if let Some(content) = content {
                result.push_str(&format!("content-encoding: {}\r\n", content));
                handled_headers.insert("content-encoding".to_string());
            } 
        } 
        
        // Add cookies based on whether this is a request or response
        if let Some(ref cookies) = self.cookies {
            if self.start_line.is_request() {
                // For requests, we use the Cookie header
                let cookie_header = cookies.request();
                if !cookie_header.is_empty() {
                    result.push_str(&format!("{}\r\n", cookie_header));
                    handled_headers.insert("cookie".to_string());
                }
            } else {
                // For responses, we use Set-Cookie headers
                let cookie_header = cookies.response();
                if !cookie_header.is_empty() {
                    result.push_str(&format!("{}", cookie_header.into_header_string("set-cookie"))); 
                    handled_headers.insert("set-cookie".to_string());
                }
            }
        }
        
        // Now process any remaining headers from the hashmap
        for (key, value) in &self.header {
            if !handled_headers.contains(key) {
                result.push_str(&value.into_header_string(key));
            }
        }
        
        // End headers with an extra CRLF
        result.push_str("\r\n");
        
        result 
    } 
} 

impl Default for HttpMeta { 
    fn default() -> Self {
        Self { 
            start_line: HttpStartLine::new_request( 
                HttpVersion::Http11,
                HttpMethod::GET,
                "/".to_string(),
            ), 
            header: HashMap::new(),
            content_type: None, 
            content_length: None, 
            content_disposition: None, 
            cookies: None, 
            encoding: None, 
            host: None, 
            lang: None, 
            location: None, 
        }
    } 
}
