#![allow(non_snake_case)] 
#![allow(non_camel_case_types)] 

use std::{collections::HashMap, hash::Hash}; 
use hotaru_lib::url_encoding::*; 

#[derive(Debug, Clone)]  
pub enum HttpVersion { 
    Http09,
    Http10,
    Http11,
    Http20,
    Http30, 
    Unknown, 
} 

impl HttpVersion { 
    pub fn to_string(&self) -> String { 
        match self { 
            HttpVersion::Http09 => "HTTP/0.9".to_string(), 
            HttpVersion::Http10 => "HTTP/1.0".to_string(), 
            HttpVersion::Http11 => "HTTP/1.1".to_string(), 
            HttpVersion::Http20 => "HTTP/2.0".to_string(), 
            HttpVersion::Http30 => "HTTP/3.0".to_string(), 
            _ => "UNKNOWN".to_string(), 
        } 
    } 

    pub fn from_string(version: &str) -> Self { 
        match version { 
            "HTTP/0.9" => HttpVersion::Http09, 
            "HTTP/1.0" => HttpVersion::Http10, 
            "HTTP/1.1" => HttpVersion::Http11, 
            "HTTP/2.0" => HttpVersion::Http20, 
            "HTTP/3.0" => HttpVersion::Http30, 
            _ => HttpVersion::Unknown,  
        }  
    }  
} 

impl std::fmt::Display for HttpVersion { 
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { 
        write!(f, "{}", self.to_string()) 
    } 
} 

#[derive(Debug, Clone, PartialEq)] 
pub enum HttpMethod { 
    GET, 
    POST, 
    PUT, 
    DELETE, 
    HEAD, 
    OPTIONS, 
    PATCH, 
    TRACE, 
    CONNECT, 
    UNKNOWN, 
} 

impl HttpMethod { 
    pub fn to_string(&self) -> String { 
        match self { 
            HttpMethod::GET => "GET".to_string(), 
            HttpMethod::POST => "POST".to_string(), 
            HttpMethod::PUT => "PUT".to_string(), 
            HttpMethod::DELETE => "DELETE".to_string(), 
            HttpMethod::HEAD => "HEAD".to_string(), 
            HttpMethod::OPTIONS => "OPTIONS".to_string(), 
            HttpMethod::PATCH => "PATCH".to_string(), 
            HttpMethod::TRACE => "TRACE".to_string(), 
            HttpMethod::CONNECT => "CONNECT".to_string(), 
            _ => "UNKNOWN".to_string(), 
        } 
    } 

    pub fn from_string(method: &str) -> Self { 
        match method { 
            "GET" => HttpMethod::GET, 
            "POST" => HttpMethod::POST, 
            "PUT" => HttpMethod::PUT, 
            "DELETE" => HttpMethod::DELETE, 
            "HEAD" => HttpMethod::HEAD, 
            "OPTIONS" => HttpMethod::OPTIONS, 
            "PATCH" => HttpMethod::PATCH, 
            "TRACE" => HttpMethod::TRACE, 
            "CONNECT" => HttpMethod::CONNECT, 
            _ => HttpMethod::UNKNOWN,  
        }  
    }  

    pub fn get_full_list() -> Vec<HttpMethod> { 
        vec![ 
            HttpMethod::GET, 
            HttpMethod::POST, 
            HttpMethod::PUT, 
            HttpMethod::DELETE, 
            HttpMethod::HEAD, 
            HttpMethod::OPTIONS, 
            HttpMethod::PATCH, 
            HttpMethod::TRACE, 
            HttpMethod::CONNECT, 
        ] 
    } 
} 

impl std::fmt::Display for HttpMethod { 
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { 
        write!(f, "{}", self.to_string()) 
    } 
} 

impl Default for HttpMethod { 
    fn default() -> Self { 
        HttpMethod::GET  
    } 
} 

impl PartialEq<&HttpMethod> for HttpMethod { 
    fn eq(&self, other: &&HttpMethod) -> bool { 
        self == *other 
    } 
} 

impl PartialEq<HttpMethod> for &HttpMethod {
    fn eq(&self, other: &HttpMethod) -> bool {
        **self == *other
    }
} 

/// Represents HTTP status codes.
///
/// This enum includes the most common HTTP status codes and provides methods
/// to check status code categories, convert between numeric and string representations,
/// and perform other common operations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StatusCode {
    // 1xx - Informational
    CONTINUE = 100,
    SWITCHING_PROTOCOLS = 101,
    PROCESSING = 102,
    EARLY_HINTS = 103,

    // 2xx - Success
    OK = 200,
    CREATED = 201,
    ACCEPTED = 202,
    NON_AUTHORITATIVE_INFORMATION = 203,
    NO_CONTENT = 204,
    RESET_CONTENT = 205,
    PARTIAL_CONTENT = 206,
    MULTI_STATUS = 207,
    ALREADY_REPORTED = 208,
    IM_USED = 226,

    // 3xx - Redirection
    MULTIPLE_CHOICES = 300,
    MOVED_PERMANENTLY = 301,
    FOUND = 302,
    SEE_OTHER = 303,
    NOT_MODIFIED = 304,
    USE_PROXY = 305,
    TEMPORARY_REDIRECT = 307,
    PERMANENT_REDIRECT = 308,

    // 4xx - Client Error
    BAD_REQUEST = 400,
    UNAUTHORIZED = 401,
    PAYMENT_REQUIRED = 402,
    FORBIDDEN = 403,
    NOT_FOUND = 404,
    METHOD_NOT_ALLOWED = 405,
    NOT_ACCEPTABLE = 406,
    PROXY_AUTHENTICATION_REQUIRED = 407,
    REQUEST_TIMEOUT = 408,
    CONFLICT = 409,
    GONE = 410,
    LENGTH_REQUIRED = 411,
    PRECONDITION_FAILED = 412,
    PAYLOAD_TOO_LARGE = 413,
    URI_TOO_LONG = 414,
    UNSUPPORTED_MEDIA_TYPE = 415,
    RANGE_NOT_SATISFIABLE = 416,
    EXPECTATION_FAILED = 417,
    IM_A_TEAPOT = 418,
    MISDIRECTED_REQUEST = 421,
    UNPROCESSABLE_ENTITY = 422,
    LOCKED = 423,
    FAILED_DEPENDENCY = 424,
    TOO_EARLY = 425,
    UPGRADE_REQUIRED = 426,
    PRECONDITION_REQUIRED = 428,
    TOO_MANY_REQUESTS = 429,
    REQUEST_HEADER_FIELDS_TOO_LARGE = 431,
    UNAVAILABLE_FOR_LEGAL_REASONS = 451,

    // 5xx - Server Error
    INTERNAL_SERVER_ERROR = 500,
    NOT_IMPLEMENTED = 501,
    BAD_GATEWAY = 502,
    SERVICE_UNAVAILABLE = 503,
    GATEWAY_TIMEOUT = 504,
    HTTP_VERSION_NOT_SUPPORTED = 505,
    VARIANT_ALSO_NEGOTIATES = 506,
    INSUFFICIENT_STORAGE = 507,
    LOOP_DETECTED = 508,
    NOT_EXTENDED = 510,
    NETWORK_AUTHENTICATION_REQUIRED = 511,

    // Unknown status code
    UNKNOWN = 0,
}

impl StatusCode {
    /// Returns the numeric value of the status code.
    ///
    /// # Returns
    ///
    /// The u16 value of this status code.
    ///
    /// # Examples
    ///
    /// ```
    /// use hotaru_core::http::http_value::StatusCode;
    /// let code = StatusCode::OK;
    /// assert_eq!(code.as_u16(), 200);
    /// ```
    pub fn as_u16(&self) -> u16 {
        self.clone() as u16
    }

    /// Returns a string representation of the status code.
    ///
    /// # Returns
    ///
    /// A string containing both the numeric code and its reason phrase.
    ///
    /// # Examples
    ///
    /// ```
    /// use hotaru_core::http::http_value::StatusCode;
    /// let code = StatusCode::OK;
    /// assert_eq!(code.to_string(), "200 OK");
    /// ```
    pub fn to_string(&self) -> String {
        match self {
            StatusCode::CONTINUE => "100 Continue",
            StatusCode::SWITCHING_PROTOCOLS => "101 Switching Protocols",
            StatusCode::PROCESSING => "102 Processing",
            StatusCode::EARLY_HINTS => "103 Early Hints",
            
            StatusCode::OK => "200 OK",
            StatusCode::CREATED => "201 Created",
            StatusCode::ACCEPTED => "202 Accepted",
            StatusCode::NON_AUTHORITATIVE_INFORMATION => "203 Non-Authoritative Information",
            StatusCode::NO_CONTENT => "204 No Content",
            StatusCode::RESET_CONTENT => "205 Reset Content",
            StatusCode::PARTIAL_CONTENT => "206 Partial Content",
            StatusCode::MULTI_STATUS => "207 Multi-Status",
            StatusCode::ALREADY_REPORTED => "208 Already Reported",
            StatusCode::IM_USED => "226 IM Used",
            
            StatusCode::MULTIPLE_CHOICES => "300 Multiple Choices",
            StatusCode::MOVED_PERMANENTLY => "301 Moved Permanently",
            StatusCode::FOUND => "302 Found",
            StatusCode::SEE_OTHER => "303 See Other",
            StatusCode::NOT_MODIFIED => "304 Not Modified",
            StatusCode::USE_PROXY => "305 Use Proxy",
            StatusCode::TEMPORARY_REDIRECT => "307 Temporary Redirect",
            StatusCode::PERMANENT_REDIRECT => "308 Permanent Redirect",
            
            StatusCode::BAD_REQUEST => "400 Bad Request",
            StatusCode::UNAUTHORIZED => "401 Unauthorized",
            StatusCode::PAYMENT_REQUIRED => "402 Payment Required",
            StatusCode::FORBIDDEN => "403 Forbidden",
            StatusCode::NOT_FOUND => "404 Not Found",
            StatusCode::METHOD_NOT_ALLOWED => "405 Method Not Allowed",
            StatusCode::NOT_ACCEPTABLE => "406 Not Acceptable",
            StatusCode::PROXY_AUTHENTICATION_REQUIRED => "407 Proxy Authentication Required",
            StatusCode::REQUEST_TIMEOUT => "408 Request Timeout",
            StatusCode::CONFLICT => "409 Conflict",
            StatusCode::GONE => "410 Gone",
            StatusCode::LENGTH_REQUIRED => "411 Length Required",
            StatusCode::PRECONDITION_FAILED => "412 Precondition Failed",
            StatusCode::PAYLOAD_TOO_LARGE => "413 Payload Too Large",
            StatusCode::URI_TOO_LONG => "414 URI Too Long",
            StatusCode::UNSUPPORTED_MEDIA_TYPE => "415 Unsupported Media Type",
            StatusCode::RANGE_NOT_SATISFIABLE => "416 Range Not Satisfiable",
            StatusCode::EXPECTATION_FAILED => "417 Expectation Failed",
            StatusCode::IM_A_TEAPOT => "418 I'm a teapot",
            StatusCode::MISDIRECTED_REQUEST => "421 Misdirected Request",
            StatusCode::UNPROCESSABLE_ENTITY => "422 Unprocessable Entity",
            StatusCode::LOCKED => "423 Locked",
            StatusCode::FAILED_DEPENDENCY => "424 Failed Dependency",
            StatusCode::TOO_EARLY => "425 Too Early",
            StatusCode::UPGRADE_REQUIRED => "426 Upgrade Required",
            StatusCode::PRECONDITION_REQUIRED => "428 Precondition Required",
            StatusCode::TOO_MANY_REQUESTS => "429 Too Many Requests",
            StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE => "431 Request Header Fields Too Large",
            StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS => "451 Unavailable For Legal Reasons",
            
            StatusCode::INTERNAL_SERVER_ERROR => "500 Internal Server Error",
            StatusCode::NOT_IMPLEMENTED => "501 Not Implemented",
            StatusCode::BAD_GATEWAY => "502 Bad Gateway",
            StatusCode::SERVICE_UNAVAILABLE => "503 Service Unavailable",
            StatusCode::GATEWAY_TIMEOUT => "504 Gateway Timeout",
            StatusCode::HTTP_VERSION_NOT_SUPPORTED => "505 HTTP Version Not Supported",
            StatusCode::VARIANT_ALSO_NEGOTIATES => "506 Variant Also Negotiates",
            StatusCode::INSUFFICIENT_STORAGE => "507 Insufficient Storage",
            StatusCode::LOOP_DETECTED => "508 Loop Detected",
            StatusCode::NOT_EXTENDED => "510 Not Extended",
            StatusCode::NETWORK_AUTHENTICATION_REQUIRED => "511 Network Authentication Required",
            
            StatusCode::UNKNOWN => "0 Unknown",
        }.to_string()
    }

    /// Gets only the reason phrase part of the status code.
    ///
    /// # Returns
    ///
    /// The reason phrase part of the status code.
    pub fn reason_phrase(&self) -> &'static str {
        match self {
            StatusCode::CONTINUE => "Continue",
            StatusCode::SWITCHING_PROTOCOLS => "Switching Protocols",
            StatusCode::PROCESSING => "Processing",
            StatusCode::EARLY_HINTS => "Early Hints",
            
            StatusCode::OK => "OK",
            StatusCode::CREATED => "Created",
            StatusCode::ACCEPTED => "Accepted",
            StatusCode::NON_AUTHORITATIVE_INFORMATION => "Non-Authoritative Information",
            StatusCode::NO_CONTENT => "No Content",
            StatusCode::RESET_CONTENT => "Reset Content",
            StatusCode::PARTIAL_CONTENT => "Partial Content",
            StatusCode::MULTI_STATUS => "Multi-Status",
            StatusCode::ALREADY_REPORTED => "Already Reported",
            StatusCode::IM_USED => "IM Used",
            
            StatusCode::MULTIPLE_CHOICES => "Multiple Choices",
            StatusCode::MOVED_PERMANENTLY => "Moved Permanently",
            StatusCode::FOUND => "Found",
            StatusCode::SEE_OTHER => "See Other",
            StatusCode::NOT_MODIFIED => "Not Modified",
            StatusCode::USE_PROXY => "Use Proxy",
            StatusCode::TEMPORARY_REDIRECT => "Temporary Redirect",
            StatusCode::PERMANENT_REDIRECT => "Permanent Redirect",
            
            StatusCode::BAD_REQUEST => "Bad Request",
            StatusCode::UNAUTHORIZED => "Unauthorized",
            StatusCode::PAYMENT_REQUIRED => "Payment Required",
            StatusCode::FORBIDDEN => "Forbidden",
            StatusCode::NOT_FOUND => "Not Found",
            StatusCode::METHOD_NOT_ALLOWED => "Method Not Allowed",
            StatusCode::NOT_ACCEPTABLE => "Not Acceptable",
            StatusCode::PROXY_AUTHENTICATION_REQUIRED => "Proxy Authentication Required",
            StatusCode::REQUEST_TIMEOUT => "Request Timeout",
            StatusCode::CONFLICT => "Conflict",
            StatusCode::GONE => "Gone",
            StatusCode::LENGTH_REQUIRED => "Length Required",
            StatusCode::PRECONDITION_FAILED => "Precondition Failed",
            StatusCode::PAYLOAD_TOO_LARGE => "Payload Too Large",
            StatusCode::URI_TOO_LONG => "URI Too Long",
            StatusCode::UNSUPPORTED_MEDIA_TYPE => "Unsupported Media Type",
            StatusCode::RANGE_NOT_SATISFIABLE => "Range Not Satisfiable",
            StatusCode::EXPECTATION_FAILED => "Expectation Failed",
            StatusCode::IM_A_TEAPOT => "I'm a teapot",
            StatusCode::MISDIRECTED_REQUEST => "Misdirected Request",
            StatusCode::UNPROCESSABLE_ENTITY => "Unprocessable Entity",
            StatusCode::LOCKED => "Locked",
            StatusCode::FAILED_DEPENDENCY => "Failed Dependency",
            StatusCode::TOO_EARLY => "Too Early",
            StatusCode::UPGRADE_REQUIRED => "Upgrade Required",
            StatusCode::PRECONDITION_REQUIRED => "Precondition Required",
            StatusCode::TOO_MANY_REQUESTS => "Too Many Requests",
            StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE => "Request Header Fields Too Large",
            StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS => "Unavailable For Legal Reasons",
            
            StatusCode::INTERNAL_SERVER_ERROR => "Internal Server Error",
            StatusCode::NOT_IMPLEMENTED => "Not Implemented",
            StatusCode::BAD_GATEWAY => "Bad Gateway",
            StatusCode::SERVICE_UNAVAILABLE => "Service Unavailable",
            StatusCode::GATEWAY_TIMEOUT => "Gateway Timeout",
            StatusCode::HTTP_VERSION_NOT_SUPPORTED => "HTTP Version Not Supported",
            StatusCode::VARIANT_ALSO_NEGOTIATES => "Variant Also Negotiates",
            StatusCode::INSUFFICIENT_STORAGE => "Insufficient Storage",
            StatusCode::LOOP_DETECTED => "Loop Detected",
            StatusCode::NOT_EXTENDED => "Not Extended",
            StatusCode::NETWORK_AUTHENTICATION_REQUIRED => "Network Authentication Required",
            
            StatusCode::UNKNOWN => "Unknown",
        }
    }

    /// Creates a status code from a u16 value.
    ///
    /// # Arguments
    ///
    /// * `code` - The numeric status code.
    ///
    /// # Returns
    ///
    /// The corresponding StatusCode enum value, or UNKNOWN for unsupported codes.
    pub fn from_u16(code: u16) -> Self {
        match code {
            100 => StatusCode::CONTINUE,
            101 => StatusCode::SWITCHING_PROTOCOLS,
            102 => StatusCode::PROCESSING,
            103 => StatusCode::EARLY_HINTS,
            
            200 => StatusCode::OK,
            201 => StatusCode::CREATED,
            202 => StatusCode::ACCEPTED,
            203 => StatusCode::NON_AUTHORITATIVE_INFORMATION,
            204 => StatusCode::NO_CONTENT,
            205 => StatusCode::RESET_CONTENT,
            206 => StatusCode::PARTIAL_CONTENT,
            207 => StatusCode::MULTI_STATUS,
            208 => StatusCode::ALREADY_REPORTED,
            226 => StatusCode::IM_USED,
            
            300 => StatusCode::MULTIPLE_CHOICES,
            301 => StatusCode::MOVED_PERMANENTLY,
            302 => StatusCode::FOUND,
            303 => StatusCode::SEE_OTHER,
            304 => StatusCode::NOT_MODIFIED,
            305 => StatusCode::USE_PROXY,
            307 => StatusCode::TEMPORARY_REDIRECT,
            308 => StatusCode::PERMANENT_REDIRECT,
            
            400 => StatusCode::BAD_REQUEST,
            401 => StatusCode::UNAUTHORIZED,
            402 => StatusCode::PAYMENT_REQUIRED,
            403 => StatusCode::FORBIDDEN,
            404 => StatusCode::NOT_FOUND,
            405 => StatusCode::METHOD_NOT_ALLOWED,
            406 => StatusCode::NOT_ACCEPTABLE,
            407 => StatusCode::PROXY_AUTHENTICATION_REQUIRED,
            408 => StatusCode::REQUEST_TIMEOUT,
            409 => StatusCode::CONFLICT,
            410 => StatusCode::GONE,
            411 => StatusCode::LENGTH_REQUIRED,
            412 => StatusCode::PRECONDITION_FAILED,
            413 => StatusCode::PAYLOAD_TOO_LARGE,
            414 => StatusCode::URI_TOO_LONG,
            415 => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            416 => StatusCode::RANGE_NOT_SATISFIABLE,
            417 => StatusCode::EXPECTATION_FAILED,
            418 => StatusCode::IM_A_TEAPOT,
            421 => StatusCode::MISDIRECTED_REQUEST,
            422 => StatusCode::UNPROCESSABLE_ENTITY,
            423 => StatusCode::LOCKED,
            424 => StatusCode::FAILED_DEPENDENCY,
            425 => StatusCode::TOO_EARLY,
            426 => StatusCode::UPGRADE_REQUIRED,
            428 => StatusCode::PRECONDITION_REQUIRED,
            429 => StatusCode::TOO_MANY_REQUESTS,
            431 => StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE,
            451 => StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS,
            
            500 => StatusCode::INTERNAL_SERVER_ERROR,
            501 => StatusCode::NOT_IMPLEMENTED,
            502 => StatusCode::BAD_GATEWAY,
            503 => StatusCode::SERVICE_UNAVAILABLE,
            504 => StatusCode::GATEWAY_TIMEOUT,
            505 => StatusCode::HTTP_VERSION_NOT_SUPPORTED,
            506 => StatusCode::VARIANT_ALSO_NEGOTIATES,
            507 => StatusCode::INSUFFICIENT_STORAGE,
            508 => StatusCode::LOOP_DETECTED,
            510 => StatusCode::NOT_EXTENDED,
            511 => StatusCode::NETWORK_AUTHENTICATION_REQUIRED,
            
            _ => StatusCode::UNKNOWN,
        }
    }

    /// Creates a status code from a string representation.
    ///
    /// # Arguments
    ///
    /// * `code` - The string representation of the status code.
    ///
    /// # Returns
    ///
    /// The corresponding StatusCode enum value, or UNKNOWN if not recognized.
    pub fn from_string(code: &str) -> Self {
        // Try parsing just the numeric part first
        if let Ok(num) = code.split_whitespace().next().unwrap_or("").parse::<u16>() {
            return StatusCode::from_u16(num);
        }
        
        // If that fails, try matching the full string
        match code {
            "100 Continue" => StatusCode::CONTINUE,
            "101 Switching Protocols" => StatusCode::SWITCHING_PROTOCOLS,
            "102 Processing" => StatusCode::PROCESSING,
            "103 Early Hints" => StatusCode::EARLY_HINTS,
            
            "200 OK" => StatusCode::OK,
            "201 Created" => StatusCode::CREATED,
            "202 Accepted" => StatusCode::ACCEPTED,
            "203 Non-Authoritative Information" => StatusCode::NON_AUTHORITATIVE_INFORMATION,
            "204 No Content" => StatusCode::NO_CONTENT,
            "205 Reset Content" => StatusCode::RESET_CONTENT,
            "206 Partial Content" => StatusCode::PARTIAL_CONTENT,
            "207 Multi-Status" => StatusCode::MULTI_STATUS,
            "208 Already Reported" => StatusCode::ALREADY_REPORTED,
            "226 IM Used" => StatusCode::IM_USED,
            
            "300 Multiple Choices" => StatusCode::MULTIPLE_CHOICES,
            "301 Moved Permanently" => StatusCode::MOVED_PERMANENTLY,
            "302 Found" => StatusCode::FOUND,
            "303 See Other" => StatusCode::SEE_OTHER,
            "304 Not Modified" => StatusCode::NOT_MODIFIED,
            "305 Use Proxy" => StatusCode::USE_PROXY,
            "307 Temporary Redirect" => StatusCode::TEMPORARY_REDIRECT,
            "308 Permanent Redirect" => StatusCode::PERMANENT_REDIRECT,
            
            "400 Bad Request" => StatusCode::BAD_REQUEST,
            "401 Unauthorized" => StatusCode::UNAUTHORIZED,
            "402 Payment Required" => StatusCode::PAYMENT_REQUIRED,
            "403 Forbidden" => StatusCode::FORBIDDEN,
            "404 Not Found" => StatusCode::NOT_FOUND,
            "405 Method Not Allowed" => StatusCode::METHOD_NOT_ALLOWED,
            "406 Not Acceptable" => StatusCode::NOT_ACCEPTABLE,
            "407 Proxy Authentication Required" => StatusCode::PROXY_AUTHENTICATION_REQUIRED,
            "408 Request Timeout" => StatusCode::REQUEST_TIMEOUT,
            "409 Conflict" => StatusCode::CONFLICT,
            "410 Gone" => StatusCode::GONE,
            "411 Length Required" => StatusCode::LENGTH_REQUIRED,
            "412 Precondition Failed" => StatusCode::PRECONDITION_FAILED,
            "413 Payload Too Large" => StatusCode::PAYLOAD_TOO_LARGE,
            "414 URI Too Long" => StatusCode::URI_TOO_LONG,
            "415 Unsupported Media Type" => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "416 Range Not Satisfiable" => StatusCode::RANGE_NOT_SATISFIABLE,
            "417 Expectation Failed" => StatusCode::EXPECTATION_FAILED,
            "418 I'm a teapot" => StatusCode::IM_A_TEAPOT,
            "421 Misdirected Request" => StatusCode::MISDIRECTED_REQUEST,
            "422 Unprocessable Entity" => StatusCode::UNPROCESSABLE_ENTITY,
            "423 Locked" => StatusCode::LOCKED,
            "424 Failed Dependency" => StatusCode::FAILED_DEPENDENCY,
            "425 Too Early" => StatusCode::TOO_EARLY,
            "426 Upgrade Required" => StatusCode::UPGRADE_REQUIRED,
            "428 Precondition Required" => StatusCode::PRECONDITION_REQUIRED,
            "429 Too Many Requests" => StatusCode::TOO_MANY_REQUESTS,
            "431 Request Header Fields Too Large" => StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE,
            "451 Unavailable For Legal Reasons" => StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS,
            
            "500 Internal Server Error" => StatusCode::INTERNAL_SERVER_ERROR,
            "501 Not Implemented" => StatusCode::NOT_IMPLEMENTED,
            "502 Bad Gateway" => StatusCode::BAD_GATEWAY,
            "503 Service Unavailable" => StatusCode::SERVICE_UNAVAILABLE,
            "504 Gateway Timeout" => StatusCode::GATEWAY_TIMEOUT,
            "505 HTTP Version Not Supported" => StatusCode::HTTP_VERSION_NOT_SUPPORTED,
            "506 Variant Also Negotiates" => StatusCode::VARIANT_ALSO_NEGOTIATES,
            "507 Insufficient Storage" => StatusCode::INSUFFICIENT_STORAGE,
            "508 Loop Detected" => StatusCode::LOOP_DETECTED,
            "510 Not Extended" => StatusCode::NOT_EXTENDED,
            "511 Network Authentication Required" => StatusCode::NETWORK_AUTHENTICATION_REQUIRED,
            
            _ => StatusCode::UNKNOWN,
        }
    }

    /// Checks if the status code is informational (1xx).
    ///
    /// # Returns
    ///
    /// `true` if the status code is in the 1xx range, `false` otherwise.
    pub fn is_informational(&self) -> bool {
        let code = self.as_u16();
        (100..=199).contains(&code)
    }

    /// Checks if the status code indicates success (2xx).
    ///
    /// # Returns
    ///
    /// `true` if the status code is in the 2xx range, `false` otherwise.
    pub fn is_success(&self) -> bool {
        let code = self.as_u16();
        (200..=299).contains(&code)
    }

    /// Checks if the status code indicates redirection (3xx).
    ///
    /// # Returns
    ///
    /// `true` if the status code is in the 3xx range, `false` otherwise.
    pub fn is_redirection(&self) -> bool {
        let code = self.as_u16();
        (300..=399).contains(&code)
    }

    /// Checks if the status code indicates a client error (4xx).
    ///
    /// # Returns
    ///
    /// `true` if the status code is in the 4xx range, `false` otherwise.
    pub fn is_client_error(&self) -> bool {
        let code = self.as_u16();
        (400..=499).contains(&code)
    }

    /// Checks if the status code indicates a server error (5xx).
    ///
    /// # Returns
    ///
    /// `true` if the status code is in the 5xx range, `false` otherwise.
    pub fn is_server_error(&self) -> bool {
        let code = self.as_u16();
        (500..=599).contains(&code)
    }

    /// Checks if the status code indicates an error (4xx or 5xx).
    ///
    /// # Returns
    ///
    /// `true` if the status code is in the 4xx or 5xx range, `false` otherwise.
    pub fn is_error(&self) -> bool {
        self.is_client_error() || self.is_server_error()
    }

    /// Checks if the status code indicates that the resource was not found (404).
    ///
    /// # Returns
    ///
    /// `true` if the status code is 404, `false` otherwise.
    pub fn is_not_found(&self) -> bool {
        *self == StatusCode::NOT_FOUND
    }

    /// Checks if the status code indicates a successful response (200 OK).
    ///
    /// # Returns
    ///
    /// `true` if the status code is 200, `false` otherwise.
    pub fn is_ok(&self) -> bool {
        *self == StatusCode::OK
    }

    /// Checks if the status code indicates that the content should be omitted (204 No Content).
    ///
    /// # Returns
    ///
    /// `true` if the status code is 204, `false` otherwise.
    pub fn is_no_content(&self) -> bool {
        *self == StatusCode::NO_CONTENT
    }

    /// Checks if the status code indicates that the requested resource was created (201 Created).
    ///
    /// # Returns
    ///
    /// `true` if the status code is 201, `false` otherwise.
    pub fn is_created(&self) -> bool {
        *self == StatusCode::CREATED
    }
    
    /// Checks if the status code indicates that the client is not authorized (401 Unauthorized).
    ///
    /// # Returns
    ///
    /// `true` if the status code is 401, `false` otherwise.
    pub fn is_unauthorized(&self) -> bool {
        *self == StatusCode::UNAUTHORIZED
    }
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl From<u16> for StatusCode {
    fn from(code: u16) -> Self {
        StatusCode::from_u16(code)
    }
}

impl From<&str> for StatusCode {
    fn from(code: &str) -> Self {
        StatusCode::from_string(code)
    }
}

impl From<String> for StatusCode {
    fn from(code: String) -> Self {
        StatusCode::from_string(&code)
    }
}

impl From<StatusCode> for u16 {
    fn from(code: StatusCode) -> Self {
        code.as_u16()
    }
}

impl From<StatusCode> for String {
    fn from(code: StatusCode) -> Self {
        code.to_string()
    }
} 

/// Represents the content type of an HTTP message. 
/// This enum is used to parse and construct HTTP headers related to content type. 
/// It includes well-known content types like text, application, image, audio, video, and multipart. 
/// It also includes a generic Other variant for any other content types. 
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpContentType {
    // Well-known content types
    Text { subtype: String, charset: Option<String> },
    Application { subtype: String, parameters: Option<Vec<(String, String)>> },
    Image { subtype: String },
    Audio { subtype: String },
    Video { subtype: String }, 
    Model { subtype: String }, 
    Multipart { subtype: String, boundary: Option<String> },
    Other { type_name: String, subtype: String, parameters: Option<Vec<(String, String)>> },
} 

impl HttpContentType {
    /// Converts a string into an HttpContentType enum variant 
    /// This function parses the content type string and extracts the main type, subtype, and any parameters.
    /// It supports well-known content types like text, application, image, audio, video, and multipart. ]
    /// 
    /// # Examples 
    /// 
    /// ```rust 
    /// use hotaru_core::http::http_value::HttpContentType; 
    /// let content_type = HttpContentType::from_str("text/html; charset=UTF-8"); 
    /// assert_eq!(content_type, HttpContentType::Text { subtype: "html".to_string(), charset: Some("UTF-8".to_string()) }); 
    /// ``` 
    pub fn from_str(content_type: &str) -> Self {
        let parts: Vec<&str> = content_type.split(';').collect();
        let main_part = parts[0].trim();
        let mut parameters = Vec::new();

        for part in &parts[1..] {
            let param_parts: Vec<&str> = part.split('=').collect();
            if param_parts.len() == 2 {
                parameters.push((param_parts[0].trim().to_string(), param_parts[1].trim().to_string()));
            }
        }

        let (type_name, subtype) = if let Some(pos) = main_part.find('/') {
            (&main_part[..pos], &main_part[pos + 1..])
        } else {
            ("unknown", "unknown")
        };

        match type_name {
            "text" => Self::Text { 
                subtype: subtype.to_string(), 
                charset: Self::find_value_from_vec(&parameters, "charset"),
            }, 
            "application" => Self::Application { 
                subtype: subtype.to_string(), 
                parameters: Some(parameters) 
            },
            "image" => Self::Image { 
                subtype: subtype.to_string() 
            },
            "audio" => Self::Audio { 
                subtype: subtype.to_string() 
            },
            "video" => Self::Video { 
                subtype: subtype.to_string() 
            }, 
            "model" => Self::Model { 
                subtype: subtype.to_string() 
            }, 
            "multipart" => Self::Multipart { 
                subtype: subtype.to_string(), 
                boundary: Self::find_value_from_vec(&parameters, "boundary"),  
            },
            _ => Self::Other {
                type_name: type_name.to_string(), 
                subtype: subtype.to_string(), 
                parameters: Some(parameters) 
            },
        } 
    } 

    /// Find value from Vec<(String, String)> 
    /// # Examples 
    /// ```rust 
    /// use hotaru_core::http::http_value::HttpContentType; 
    /// let vec = vec![("key1".to_string(), "value1".to_string()), ("key2".to_string(), "value2".to_string())]; 
    /// let value = HttpContentType::find_value_from_vec(&vec, "key1"); 
    /// assert_eq!(value, Some("value1".to_string())); 
    /// ``` 
    pub fn find_value_from_vec(vec: &Vec<(String, String)>, key: &str) -> Option<String> { 
        for (k, v) in vec { 
            if k == key { 
                return Some(v.clone()); 
            } 
        } 
        None 
    } 

    /// Converts an HttpContentType enum variant into its string representation
    pub fn to_string(&self) -> String {
        match self {
            HttpContentType::Text { subtype, charset } => match charset {
                Some(cs) => format!("text/{}; charset={}", subtype, cs),
                None => format!("text/{}", subtype)
            }, 
            HttpContentType::Application { subtype, parameters } => {
                let mut result = format!("application/{}", subtype);
                if let Some(params) = parameters {
                    for (key, value) in params {
                        result.push_str(&format!("; {}={}", key, value));
                    }
                }
                result
            },
            HttpContentType::Image { subtype } => format!("image/{}", subtype),
            HttpContentType::Audio { subtype } => format!("audio/{}", subtype),
            HttpContentType::Video { subtype } => format!("video/{}", subtype),
            HttpContentType::Model { subtype } => format!("model/{}", subtype), 
            HttpContentType::Multipart { subtype, boundary } => {
                if let Some(b) = boundary {
                    format!("multipart/{}; boundary={}", subtype, b)
                } else {
                    format!("multipart/{}", subtype)
                }
            },
            HttpContentType::Other { type_name, subtype, parameters } => {
                let mut result = format!("{}/{}", type_name, subtype);
                if let Some(params) = parameters {
                    for (key, value) in params {
                        result.push_str(&format!("; {}={}", key, value));
                    }
                }
                result
            } 
        }
    } 

    /// Get content type based on file extension
    /// # Examples
    /// ```
    /// use hotaru_core::http::http_value::HttpContentType;
    /// 
    /// let content_type = HttpContentType::from_extension("html");
    /// assert_eq!(content_type, HttpContentType::TextHtml());
    ///
    /// let content_type = HttpContentType::from_extension("svg");
    /// assert_eq!(content_type, HttpContentType::ImageSvg());
    /// ```
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            // Text types
            "html" | "htm" => Self::TextHtml(),
            "txt" => Self::TextPlain(),
            "css" => Self::TextCss(),
            "csv" => Self::TextCsv(),
            "xml" => Self::TextXml(),

            // Application types
            "js" | "mjs" | "cjs" => Self::ApplicationJavascript(),
            "json" => Self::ApplicationJson(),
            "pdf" => Self::ApplicationPdf(),
            "zip" | "jar" | "war" => Self::ApplicationZip(),
            "wasm" => Self::ApplicationWasm(),
            "bin" => Self::ApplicationOctetStream(),
            
            // Image types
            "png" => Self::ImagePng(),
            "jpg" | "jpeg" | "jpe" | "jif" | "jfif" => Self::ImageJpeg(),
            "gif" => Self::ImageGif(),
            "svg" | "svgz" => Self::ImageSvg(),
            "webp" => Self::ImageWebp(),
            "bmp" | "dib" => Self::ImageBmp(),
            "ico" => Self::Image { subtype: "x-icon".to_string() },
            "tiff" | "tif" => Self::Image { subtype: "tiff".to_string() },

            // Audio types
            "mp3" => Self::AudioMpeg(),
            "wav" => Self::Audio { subtype: "wav".to_string() },
            "ogg" => Self::Audio { subtype: "ogg".to_string() },
            "flac" => Self::Audio { subtype: "flac".to_string() },

            // Video types
            "mp4" | "m4v" => Self::VideoMp4(),
            "mov" => Self::Video { subtype: "quicktime".to_string() },
            "webm" => Self::Video { subtype: "webm".to_string() },
            "avi" => Self::Video { subtype: "x-msvideo".to_string() },

            // Model types
            "stl" => Self::ModelStl(),
            "obj" => Self::ModelObj(),
            "glb" => Self::Model { subtype: "gltf-binary".to_string() },
            "gltf" => Self::Model { subtype: "gltf+json".to_string() },

            // Documents
            "doc" => Self::Application { subtype: "msword".to_string(), parameters: None },
            "docx" => Self::Application { subtype: "vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(), parameters: None },
            "xls" => Self::Application { subtype: "vnd.ms-excel".to_string(), parameters: None },
            "xlsx" => Self::Application { subtype: "vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(), parameters: None },
            "ppt" => Self::Application { subtype: "vnd.ms-powerpoint".to_string(), parameters: None },
            "pptx" => Self::Application { subtype: "vnd.openxmlformats-officedocument.presentationml.presentation".to_string(), parameters: None },
            
            // Archive formats
            "tar" => Self::Application { subtype: "x-tar".to_string(), parameters: None },
            "gz" | "tgz" => Self::Application { subtype: "gzip".to_string(), parameters: None },

            // Special cases
            "form" => Self::ApplicationUrlEncodedForm(),
            // "multipart" => Self::MultipartFormData(),
            
            _ => Self::ApplicationOctetStream(),
        }
    } 

    /// Get content type based on file name
    /// This function extracts the extension from the file name and determines the appropriate content type.
    /// If the file has no extension or an unrecognized extension, it defaults to application/octet-stream.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hotaru_core::http::http_value::HttpContentType;
    /// 
    /// let content_type = HttpContentType::from_file_name("index.html");
    /// assert_eq!(content_type, HttpContentType::TextHtml());
    ///
    /// let content_type = HttpContentType::from_file_name("document.pdf");
    /// assert_eq!(content_type, HttpContentType::ApplicationPdf());
    ///
    /// // Works with paths too
    /// let content_type = HttpContentType::from_file_name("/var/www/images/logo.svg");
    /// assert_eq!(content_type, HttpContentType::ImageSvg());
    ///
    /// // Files without extension default to application/octet-stream
    /// let content_type = HttpContentType::from_file_name("README");
    /// assert_eq!(content_type, HttpContentType::ApplicationOctetStream());
    /// ```
    pub fn from_file_name(file_name: &str) -> Self {
        let path = std::path::Path::new(file_name);
        match path.extension() {
            Some(os_str) => match os_str.to_str() {
                Some(ext) => Self::from_extension(ext),
                None => Self::ApplicationOctetStream(),
            },
            None => Self::ApplicationOctetStream(),
        }
    } 
    
    // ======================
    // TEXT Content Types
    // ====================== 
    pub fn TextHtml() -> Self { 
        Self::Text { subtype: "html".to_string(), charset: Some("UTF-8".to_string()) } 
    } 

    pub fn TextPlain() -> Self { 
        Self::Text { subtype: "plain".to_string(), charset: Some("UTF-8".to_string()) } 
    } 

    pub fn TextCss() -> Self {
        Self::Text { 
            subtype: "css".to_string(), 
            charset: Some("UTF-8".to_string()) 
        }
    } 

    pub fn TextXml() -> Self {
        Self::Text { 
            subtype: "xml".to_string(), 
            charset: Some("UTF-8".to_string()) 
        }
    }

    pub fn TextCsv() -> Self {
        Self::Text { 
            subtype: "csv".to_string(), 
            charset: Some("UTF-8".to_string()) 
        }
    } 

    // ======================
    // APPLICATION Content Types
    // ====================== 
    pub fn ApplicationJavascript() -> Self {
        Self::Application { 
            subtype: "javascript".to_string(), 
            parameters: Some(vec![("charset".to_string(), "UTF-8".to_string())]) 
        }
    } 

    pub fn ApplicationJson() -> Self { 
        Self::Application { subtype: "json".to_string(), parameters: Some(vec![("charset".to_string(), "UTF-8".to_string())]) } 
    } 

    pub fn ApplicationUrlEncodedForm() -> Self { 
        Self::Application { subtype: "x-www-form-urlencoded".to_string(), parameters: Some(vec![("charset".to_string(), "UTF-8".to_string())]) } 
    }

    pub fn ApplicationXml() -> Self { 
        Self::Application { subtype: "xml".to_string(), parameters: Some(vec![("charset".to_string(), "UTF-8".to_string())]) } 
    } 

    pub fn ApplicationOctetStream() -> Self { 
        Self::Application { subtype: "octet-stream".to_string(), parameters: Some(vec![("charset".to_string(), "UTF-8".to_string())]) } 
    } 

    pub fn ApplicationPdf() -> Self {
        Self::Application { 
            subtype: "pdf".to_string(), 
            parameters: None 
        }
    }

    pub fn ApplicationZip() -> Self {
        Self::Application { 
            subtype: "zip".to_string(), 
            parameters: None 
        }
    } 

    pub fn ApplicationPkcs8() -> Self {
        Self::Application { 
            subtype: "pkcs8".to_string(), 
            parameters: None 
        }
    }

    pub fn ApplicationWasm() -> Self {
        Self::Application { 
            subtype: "wasm".to_string(), 
            parameters: Some(vec![("charset".to_string(), "UTF-8".to_string())]) 
        }
    } 

    // ======================
    // IMAGE Content Types
    // ====================== 
    pub fn ImagePng() -> Self {
        Self::Image { subtype: "png".to_string() }
    }

    pub fn ImageJpeg() -> Self {
        Self::Image { subtype: "jpeg".to_string() }
    }

    pub fn ImageGif() -> Self {
        Self::Image { subtype: "gif".to_string() }
    } 

    pub fn ImageSvg() -> Self {
        Self::Image { 
            subtype: "svg+xml".to_string() 
        }
    }

    pub fn ImageWebp() -> Self {
        Self::Image { 
            subtype: "webp".to_string() 
        }
    }

    pub fn ImageBmp() -> Self {
        Self::Image { 
            subtype: "bmp".to_string() 
        }
    }

    // ======================
    // AUDIO/VIDEO Content Types
    // ======================
    pub fn AudioMpeg() -> Self {
        Self::Audio { 
            subtype: "mpeg".to_string() 
        }
    }

    pub fn VideoMp4() -> Self {
        Self::Video { 
            subtype: "mp4".to_string() 
        }
    } 

    // ====================== 
    // MODEL Content Types 
    // ====================== 
    pub fn ModelStl() -> Self {
        Self::Model { 
            subtype: "stl".to_string() 
        }
    }

    pub fn ModelObj() -> Self {
        Self::Model { 
            subtype: "obj".to_string() 
        }
    }
}

impl std::fmt::Display for HttpContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
} 

impl Default for HttpContentType { 
    fn default() -> Self {
        Self::Other { 
            type_name: "Unknown".to_string(), 
            subtype: "Unknown".to_string(), 
            parameters: None, 
        }
    }
} 

/// Error type for Content-Disposition operations
#[derive(Debug)]
pub enum ContentDispositionError {
    /// Empty Content-Disposition header
    EmptyHeader,
    /// Invalid parameter format
    InvalidParameterFormat(String),
    /// Invalid extended parameter format
    InvalidExtendedParameter(String),
    /// UTF-8 decoding error
    Utf8DecodingError(String),
    /// Invalid charset
    InvalidCharset(String),
}

impl std::fmt::Display for ContentDispositionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyHeader => write!(f, "Empty Content-Disposition header"),
            Self::InvalidParameterFormat(s) => write!(f, "Invalid parameter format: {}", s),
            Self::InvalidExtendedParameter(s) => write!(f, "Invalid extended parameter format: {}", s),
            Self::Utf8DecodingError(s) => write!(f, "UTF-8 decoding error: {}", s),
            Self::InvalidCharset(s) => write!(f, "Invalid charset: {}", s),
        }
    }
}

impl std::error::Error for ContentDispositionError {}

/// Represents the Content-Disposition header as defined in RFC 6266.
///
/// The Content-Disposition header is commonly used in HTTP responses to suggest
/// a filename for a download, and to indicate whether the content should be displayed
/// inline or as an attachment.
///
/// This implementation supports both regular parameters and the extended parameter
/// syntax defined in RFC 5987 for internationalized filenames (filename*).
///
/// # Examples
///
/// ```
/// use hotaru_core::http::http_value::{ContentDisposition, ContentDispositionType};
///
/// // Create an attachment disposition
/// let disposition = ContentDisposition::attachment("report.pdf");
/// assert_eq!(disposition.to_string(), "attachment; filename=\"report.pdf\"");
///
/// // Parse a Content-Disposition header with extended parameter syntax
/// let header = "attachment; filename*=UTF-8''%F0%9F%93%96.txt";
/// let parsed = ContentDisposition::parse(header).unwrap();
/// assert_eq!(parsed.filename().unwrap(), "📖.txt");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ContentDisposition { 
    disposition_type: ContentDispositionType,
    // Unified parameter storage - stores both regular and extended parameters
    parameters: HashMap<String, ParameterValue>,
}

/// Represents a parameter value (either regular or extended)
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterValue {
    /// Regular parameter value
    Regular(String),
    /// Extended parameter value (RFC 5987)
    Extended(ExtendedValue),
}

/// Represents an extended parameter value as defined in RFC 5987. 
/// 
/// Note that currently only utf-8 is supported 
///
/// Extended parameter values have the format: `charset'language'value` 
/// where:
/// - `charset` is the character encoding (e.g., "UTF-8")
/// - `language` is an optional language tag (can be empty)
/// - `value` is the percent-encoded value
#[derive(Debug, Clone, PartialEq)]
pub struct ExtendedValue {
    /// The character encoding (e.g., "UTF-8")
    pub charset: String,
    /// The language tag (can be empty)
    pub language: String,
    /// The decoded value
    pub value: String,
}

/// Represents the type of a Content-Disposition header.
///
/// According to RFC 6266 and common usage, the main types are:
/// - `Inline`: Content should be displayed within the web page
/// - `Attachment`: Content should be downloaded as a file
/// - `FormData`: Content is part of a multipart form submission
/// - `Other`: Any other disposition type not covered by the standard types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentDispositionType { 
    /// Content that should be displayed inline in the browser
    Inline, 
    /// Content that should be downloaded as a file
    Attachment, 
    /// Content that is part of a multipart form submission
    FormData, 
    /// Any other disposition type not covered by the standard types
    Other(String),  
}

impl ContentDispositionType { 
    /// Creates a `ContentDispositionType` from a string.
    ///
    /// The comparison is case-insensitive. If the string does not match one of the
    /// standard types, it will be stored as `Other`.
    ///
    /// # Arguments
    ///
    /// * `string` - A string reference that can be converted to a str
    ///
    /// # Returns
    ///
    /// A `ContentDispositionType` corresponding to the input string
    pub fn from_string<A: AsRef<str>>(string: A) -> Self { 
        let string = string.as_ref().to_lowercase(); 
        match string.as_str() { 
            "inline" => ContentDispositionType::Inline, 
            "attachment" => ContentDispositionType::Attachment, 
            "form-data" => ContentDispositionType::FormData, 
            _ => ContentDispositionType::Other(string), 
        } 
    } 

    /// Converts the `ContentDispositionType` to its string representation.
    ///
    /// # Returns
    ///
    /// A `String` containing the canonical string representation of the type
    pub fn to_string(&self) -> String { 
        match self { 
            ContentDispositionType::Inline => "inline".to_string(), 
            ContentDispositionType::Attachment => "attachment".to_string(), 
            ContentDispositionType::FormData => "form-data".to_string(), 
            ContentDispositionType::Other(s) => s.to_string(), 
        } 
    } 
}

impl ExtendedValue {
    /// Creates a new ExtendedValue from its components.
    ///
    /// # Arguments
    ///
    /// * `charset` - The character encoding
    /// * `language` - The language tag
    /// * `value` - The decoded value
    ///
    /// # Returns
    ///
    /// A new `ExtendedValue`
    pub fn new<S1: Into<String>, S2: Into<String>, S3: Into<String>>(charset: S1, language: S2, value: S3) -> Self {
        ExtendedValue {
            charset: charset.into(),
            language: language.into(),
            value: value.into(),
        }
    }

    /// Parses an extended parameter value string.
    ///
    /// # Arguments
    ///
    /// * `value` - The extended parameter value string (charset'language'value)
    ///
    /// # Returns
    ///
    /// `Result<ExtendedValue, ContentDispositionError>` containing either the parsed value or an error
    pub fn parse(value: &str) -> Result<Self, ContentDispositionError> {
        let mut parts = value.splitn(3, '\'');
        
        let charset = parts.next()
            .ok_or_else(|| ContentDispositionError::InvalidExtendedParameter("missing charset".to_string()))?;
        
        let language = parts.next()
            .ok_or_else(|| ContentDispositionError::InvalidExtendedParameter("missing language".to_string()))?;
        
        let encoded_value = parts.next()
            .ok_or_else(|| ContentDispositionError::InvalidExtendedParameter("missing value".to_string()))?;
        
        // Decode the percent-encoded value
        let bytes = percent_decode(encoded_value.as_bytes()).collect::<Vec<u8>>(); 
        
        let decoded_value: String  = if charset.eq_ignore_ascii_case("utf-8") { 
            String::from_utf8(bytes)
                .map_err(|e| ContentDispositionError::Utf8DecodingError(e.to_string()))? 
        } else { 
            Err(ContentDispositionError::InvalidCharset(charset.into()))? 
        }; 

        Ok(ExtendedValue {
            charset: charset.to_string(),
            language: language.to_string(),
            value: decoded_value,
        })
    }

    /// Gets the percent-encoded form of the value
    pub fn encoded_value(&self) -> String {
        encode_url_owned(&self.value) 
    } 

    /// Converts the extended value to its string representation.
    ///
    /// # Returns
    ///
    /// A `String` containing the extended parameter value
    pub fn to_string(&self) -> String {
        format!("{}'{}'{}",
            self.charset,
            self.language,
            self.encoded_value()
        )
    }
}

impl ContentDisposition {
    /// Creates a new `ContentDisposition` with the specified type and no parameters.
    ///
    /// # Arguments
    ///
    /// * `disposition_type` - The type of content disposition
    ///
    /// # Returns
    ///
    /// A new `ContentDisposition` instance
    pub fn new(disposition_type: ContentDispositionType) -> Self {
        ContentDisposition {
            disposition_type,
            parameters: HashMap::new(),
        }
    }

    /// Creates a new `ContentDisposition` with type "inline".
    ///
    /// # Returns
    ///
    /// A new `ContentDisposition` instance with type set to `Inline`
    pub fn inline() -> Self {
        Self::new(ContentDispositionType::Inline)
    }

    /// Creates a new `ContentDisposition` with type "attachment".
    ///
    /// # Arguments
    ///
    /// * `filename` - Optional filename for the attachment
    ///
    /// # Returns
    ///
    /// A new `ContentDisposition` instance with type set to `Attachment`
    pub fn attachment<S: Into<String>>(filename: S) -> Self {
        let mut disposition = Self::new(ContentDispositionType::Attachment);
        disposition.set_filename(filename);
        disposition
    }

    /// Creates a new `ContentDisposition` with type "form-data".
    ///
    /// # Arguments
    ///
    /// * `name` - The form field name
    /// * `filename` - Optional filename for the form data
    ///
    /// # Returns
    ///
    /// A new `ContentDisposition` instance with type set to `FormData`
    pub fn form_data<S: Into<String>, T: Into<String>>(name: S, filename: Option<T>) -> Self {
        let mut disposition = Self::new(ContentDispositionType::FormData);
        disposition.set_parameter("name", name);
        if let Some(fname) = filename {
            disposition.set_filename(fname);
        }
        disposition
    }

    /// Parses a Content-Disposition header string into a `ContentDisposition`.
    ///
    /// This method supports both regular parameters and extended parameters using
    /// the RFC 5987 syntax (e.g., `filename*=UTF-8''file.txt`).
    ///
    /// # Arguments
    ///
    /// * `header` - The Content-Disposition header string
    ///
    /// # Returns
    ///
    /// `Result<ContentDisposition, ContentDispositionError>` containing either the parsed disposition
    /// or an error message
    ///
    /// # Examples
    ///
    /// ```
    /// use hotaru_core::http::http_value::ContentDisposition;
    ///
    /// // Regular parameter
    /// let header = "attachment; filename=\"example.txt\"";
    /// let disposition = ContentDisposition::parse(header).unwrap();
    /// assert_eq!(disposition.filename().unwrap(), "example.txt");
    ///
    /// // Extended parameter
    /// let header = "attachment; filename*=UTF-8''%F0%9F%93%96.txt";
    /// let disposition = ContentDisposition::parse(header).unwrap();
    /// assert_eq!(disposition.filename().unwrap(), "📖.txt");
    /// ```
    pub fn parse(header: &str) -> Result<Self, ContentDispositionError> {
        let mut parts = header.split(';').map(|s| s.trim());
        
        let first = parts.next().ok_or(ContentDispositionError::EmptyHeader)?;
        let disposition_type = ContentDispositionType::from_string(first);
        let mut disposition = ContentDisposition::new(disposition_type);
        
        // Keep track of regular vs extended parameters for precedence rules
        let mut extended_params = HashMap::new();
        let mut regular_params = HashMap::new();
        
        for part in parts {
            if let Some(idx) = part.find('=') {
                let (key, value) = part.split_at(idx);
                let key = key.trim();
                // Skip the '=' character
                let value = &value[1..].trim();
                
                // Check if this is an extended parameter (ends with *)
                if key.ends_with('*') {
                    let param_name = key[..key.len() - 1].to_lowercase();
                    match ExtendedValue::parse(value) {
                        Ok(extended_value) => {
                            extended_params.insert(param_name.clone(), extended_value.clone());
                        },
                        Err(e) => return Err(e),
                    }
                } else {
                    // Regular parameter
                    let param_name = key.to_lowercase();
                    let param_value = unescape_quoted_string(value);
                    regular_params.insert(param_name.clone(), param_value.clone());
                }
            } else if !part.is_empty() {
                return Err(ContentDispositionError::InvalidParameterFormat(part.to_string()));
            }
        }
        
        // Apply RFC 6266 precedence rules - extended parameters override regular ones
        for (name, value) in regular_params {
            if !extended_params.contains_key(&name) {
                disposition.set_parameter(&name, value);
            }
        }
        
        // Extended parameters have precedence
        for (name, value) in extended_params {
            disposition.set_parameter_extended(&name, &value.charset, &value.language, &value.value);
        }
        
        Ok(disposition)
    }

    /// Returns the disposition type.
    ///
    /// # Returns
    ///
    /// A reference to the `ContentDispositionType`
    pub fn disposition_type(&self) -> &ContentDispositionType {
        &self.disposition_type
    }

    /// Sets the disposition type.
    ///
    /// # Arguments
    ///
    /// * `disposition_type` - The new disposition type
    pub fn set_disposition_type(&mut self, disposition_type: ContentDispositionType) {
        self.disposition_type = disposition_type;
    }

    /// Returns the filename, if any.
    ///
    /// # Returns
    ///
    /// An `Option<&str>` containing the filename or `None` if no filename is not set
    ///
    /// Per RFC 6266, the filename* parameter has precedence over filename.
    pub fn filename(&self) -> Option<&str> {
        match self.parameters.get("filename") {
            Some(ParameterValue::Regular(v)) => Some(v),
            Some(ParameterValue::Extended(v)) => Some(&v.value),
            None => None,
        }
    }

    /// Sets the filename.
    ///
    /// # Arguments
    ///
    /// * `filename` - The new filename
    ///
    /// If the filename contains non-ASCII characters, it will automatically
    /// use the extended parameter format.
    pub fn set_filename<S: Into<String>>(&mut self, filename: S) {
        let filename = filename.into();
        if needs_extended_encoding(&filename) {
            self.set_parameter_extended("filename", "UTF-8", "", filename);
        } else {
            self.set_parameter("filename", filename);
        }
    }

    /// Gets a parameter value by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The parameter name
    ///
    /// # Returns
    ///
    /// An `Option<&str>` containing the parameter value or `None` if the parameter is not set
    pub fn get_parameter(&self, name: &str) -> Option<&str> {
        match self.parameters.get(&name.to_lowercase()) {
            Some(ParameterValue::Regular(v)) => Some(v),
            Some(ParameterValue::Extended(v)) => Some(&v.value),
            None => None,
        }
    }

    /// Sets a regular parameter value.
    ///
    /// # Arguments
    ///
    /// * `name` - The parameter name
    /// * `value` - The parameter value
    pub fn set_parameter<K: Into<String>, V: Into<String>>(&mut self, name: K, value: V) {
        let name = name.into().to_lowercase();
        self.parameters.insert(name, ParameterValue::Regular(value.into()));
    }

    /// Sets an extended parameter value.
    ///
    /// # Arguments
    ///
    /// * `name` - The parameter name (without the trailing '*')
    /// * `charset` - The character set (e.g., "UTF-8")
    /// * `language` - The language tag (can be empty)
    /// * `value` - The parameter value
    pub fn set_parameter_extended<K: Into<String>, C: Into<String>, L: Into<String>, V: Into<String>>(
        &mut self, 
        name: K, 
        charset: C, 
        language: L, 
        value: V
    ) {
        let name = name.into().to_lowercase();
        self.parameters.insert(
            name, 
            ParameterValue::Extended(ExtendedValue::new(charset, language, value))
        );
    }

    /// Removes a parameter.
    ///
    /// # Arguments
    ///
    /// * `name` - The parameter name
    ///
    /// # Returns
    ///
    /// `true` if the parameter was present and removed, `false` otherwise
    pub fn remove_parameter(&mut self, name: &str) -> bool {
        self.parameters.remove(&name.to_lowercase()).is_some()
    }

    /// Converts the `ContentDisposition` to its string representation suitable for
    /// use as an HTTP header value.
    ///
    /// This method handles both regular parameters and extended parameters using
    /// the RFC 5987 syntax.
    ///
    /// # Returns
    ///
    /// A `String` containing the header value
    pub fn to_string(&self) -> String {
        let mut parts = vec![self.disposition_type.to_string()];
        
        for (key, value) in &self.parameters {
            match value {
                ParameterValue::Regular(v) => {
                    parts.push(format!("{}=\"{}\"", key, escape_quoted_string(v)));
                },
                ParameterValue::Extended(v) => {
                    parts.push(format!("{}*={}", key, v.to_string()));
                    
                    // If this is a filename, also include a fallback ASCII parameter for better compatibility
                    if key == "filename" {
                        // Don't add ASCII fallback if the value is already ASCII
                        if !v.value.chars().all(|c| c <= '\u{7F}') {
                            let ascii_fallback = v.value.chars()
                                .map(|c| if c > '\u{7F}' { '_' } else { c })
                                .collect::<String>();
                            parts.push(format!("filename=\"{}\"", escape_quoted_string(&ascii_fallback)));
                        }
                    }
                },
            }
        }
        
        parts.join("; ")
    }

    /// Returns all parameters.
    ///
    /// # Returns
    ///
    /// A reference to the parameters map
    pub fn parameters(&self) -> &HashMap<String, ParameterValue> {
        &self.parameters
    }
}

impl ToString for ContentDisposition {
    fn to_string(&self) -> String {
        self.to_string()
    }
}

pub struct HeaderConstructor{ 
    pub headers: Vec<HeaderAttribute>
} 

impl HeaderConstructor{ 
    pub fn build<T: Into<String>>(string: T) -> Self { 
        let mut headers = Vec::new(); 
        let string = string.into(); 
        let parts: Vec<&str> = string.split(';').collect(); 
        for part in parts { 
            let part = part.trim(); 
            if !part.is_empty() { 
                headers.push(HeaderAttribute::build(part)); 
            } 
        } 
        Self { headers } 
    }
}

pub struct HeaderAttribute{ 
    pub main_value: String, 
    pub attributes: HashMap<String, String>, 
} 

impl HeaderAttribute{ 
    pub fn build<T: Into<String>>(part: T) -> Self{ 
        let part = part.into(); 
        let mut attributes = HashMap::new(); 
        let main_value = part.split(':').next().unwrap_or("").trim().to_string(); 
        for attr in part.split(';').skip(1) { 
            let attr_parts: Vec<&str> = attr.split('=').collect(); 
            if attr_parts.len() == 2 { 
                attributes.insert(attr_parts[0].trim().to_string(), attr_parts[1].trim().to_string()); 
            } 
        } 
        Self { main_value, attributes } 
    }
}  

#[derive(Debug, Clone)] 
pub struct RequestPath{ 
    path: Vec<String>, 
    arguments: HashMap<String, String>, 
} 

impl RequestPath{   
    pub fn new(path: Vec<String>, arguments: HashMap<String, String>) -> Self{ 
        Self { path, arguments }  
    } 

    pub fn to_string(&self) -> String{ 
        let mut result = String::new(); 
        for part in &self.path { 
            result.push('/'); 
            result.push_str(part); 
        } 
        result 
    } 

    pub fn from_string(url: &str) -> Self{ 
        let (path_str, args_str) = match url.find('?') {
            Some(pos) => (&url[..pos], &url[pos + 1..]),
            None => (url, ""),
        }; 
        let mut path = Vec::new(); 
        let parts: Vec<&str> = path_str.split('/').collect(); 
        for part in parts { 
            if !part.is_empty() { 
                path.push(part.to_string()); 
            } 
        } 
        let mut arguments = HashMap::new(); 
        for arg in args_str.split('&') { 
            let arg_parts: Vec<&str> = arg.split('=').collect(); 
            if arg_parts.len() == 2 { 
                arguments.insert(arg_parts[0].to_string(), arg_parts[1].to_string()); 
            } 
        } 
        Self { path, arguments } 
    } 

    pub fn url_part(&self, part: usize) -> String{ 
        // if part < 0 { 
        //     return self.path[self.path.len() as usize + part as usize].clone(); 
        // } else if part >= self.path.len() { 
        //     return "".to_string(); 
        // } 
        // self.path[part].clone()  
        if part >= self.path.len() { 
            return "".to_string(); 
        } 
        self.path[part].clone()    
    } 

    pub fn get_url_args(&self, key: &str) -> Option<String> {
        self.arguments.get(key).cloned()
    } 
} 

impl Default for RequestPath {
    fn default() -> Self {
        Self::new(Vec::new(), HashMap::new()) 
    }
} 

/// Represents HTTP `Accept-Language` header for client language preferences.
/// 
/// Stores language tags with quality weights (q-values) for content negotiation.
/// 
/// # RFC 7231 Compliance:
/// - Language tags are case-insensitive (but stored in original case)
/// - Default weight = 1.0 if not specified
/// - Weights range 0.0-1.0 (higher = more preferred)
/// - Order indicates priority for equal weights 
#[derive(Debug, Clone, PartialEq)] 
pub struct AcceptLang {
    langs: Vec<(String, f32)>,
}

impl AcceptLang {
    /// Parses an `Accept-Language` header string
    /// 
    /// # Example:
    /// ```
    /// use hotaru_core::http::http_value::AcceptLang;
    /// let accept_lang = AcceptLang::from_str("en-US, fr;q=0.7, zh-CN;q=0.5");
    /// assert_eq!(accept_lang.all_languages(), vec!["en-US", "fr", "zh-CN"]);
    /// ```
    pub fn from_str<S: AsRef<str>>(s: S) -> Self {
        let mut langs = Vec::new();
        
        for lang_str in s.as_ref().split(',') {
            let mut parts = lang_str.splitn(2, ';');
            let lang = parts.next().unwrap().trim().to_string();
            
            // Default weight = 1.0
            let mut weight = 1.0;
            
            // Parse q-value if exists
            if let Some(q_part) = parts.next() {
                if let Some(q_str) = q_part.trim().strip_prefix("q=") {
                    weight = q_str.trim().parse().unwrap_or(1.0);
                }
            } 
            
            langs.push((lang, weight));
        }
        
        AcceptLang { langs }
    }

    /// Returns most preferred language (highest weight, original case)
    /// 
    /// # Defaults to "en" if:
    /// - No languages exist
    /// - All weights <= 0.0
    /// 
    /// # Example:
    /// ```
    /// use hotaru_core::http::http_value::AcceptLang;
    /// let accept_lang = AcceptLang::from_str("en-US, fr;q=0.7, zh-CN;q=0.5");
    /// let lang = accept_lang.most_preferred();
    /// assert_eq!(lang, "en-US");
    /// ```
    pub fn most_preferred(&self) -> String {
        self.langs.iter()
            .max_by(|(_, w1), (_, w2)| w1.total_cmp(w2))
            .map(|(lang, _)| lang.clone())
            .unwrap_or_else(|| "en".to_string())
    } 

    /// Returns all languages in original order
    pub fn all_languages(&self) -> Vec<String> {
        self.langs.iter().map(|(lang, _)| lang.clone()).collect()
    }

    /// Gets weight for a language (case-insensitive)
    /// 
    /// # Returns 0.0 if not found
    pub fn get_weight(&self, lang: &str) -> f32 {
        self.langs.iter()
            .find(|(l, _)| l.eq_ignore_ascii_case(lang))
            .map(|(_, w)| *w)
            .unwrap_or(0.0)
    }

    /// Adds language (maintains insertion order)
    pub fn add_language(&mut self, lang: String, weight: f32) {
        self.langs.push((lang, weight));
    }

    /// Removes language (case-insensitive)
    pub fn remove_language(&mut self, lang: &str) {
        self.langs.retain(|(l, _)| !l.eq_ignore_ascii_case(lang));
    }

    /// Serializes to `Accept-Language` header format
    /// 
    /// # Formatting rules:
    /// - Omits q-value for 1.0 weights
    /// - Trims trailing zeros (0.7 → "0.7", 0.500 → "0.5")
    /// - Maintains original case
    pub fn to_header_string(&self) -> String {
        self.langs.iter()
            .map(|(lang, weight)| {
                if (weight - 1.0).abs() < f32::EPSILON {
                    lang.clone()
                } else {
                    let weight_str = format!("{:.3}", weight)
                        .trim_end_matches('0')
                        .trim_end_matches('.')
                        .to_string();
                    format!("{};q={}", lang, weight_str)
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    } 

    pub fn to_response_header(&self) -> String {
        self.most_preferred() 
    }  
}
