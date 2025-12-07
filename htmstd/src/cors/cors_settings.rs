//! Cross-Origin Resource Sharing (CORS) configuration module
//!
//! Provides a type-safe interface for configuring CORS policies with granular control
//! over allowed origins, methods, headers, credentials, and cache duration. Supports
//! merging configurations and generating appropriate HTTP headers.

use std::collections::HashSet;

/// Default allowed methods if not specified
const DEFAULT_METHODS: &[&str] = &["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

/// Default allowed headers if not specified
const DEFAULT_HEADERS: &[&str] = &[
    "accept",
    "accept-language",
    "content-language",
    "content-type",
];

/// Default max age if not specified (1 day)
const DEFAULT_MAX_AGE: u64 = 86400;

/// CORS policy settings container
///
/// # Example
/// ```
/// let base = AppCorsSettings::default();
/// let override_settings = AppCorsSettings {
///     allowed_origins: AllowedOrigins::Some(vec!["https://trusted.com".into()].into_iter().collect()),
///     allowed_methods: AllowedMethods::Unset,
///     allowed_headers: AllowedHeaders::All,
///     allowed_credentials: Some(true),
///     max_age: Some(3600),
/// };
///
/// let merged = base.merge(&override_settings);
/// ```
#[derive(Debug, Clone)]
pub struct AppCorsSettings {
    /// Configure allowed request origins
    pub allowed_origins: AllowedOrigins,
    
    /// Configure allowed HTTP methods
    pub allowed_methods: AllowedMethods,
    
    /// Configure allowed HTTP headers
    pub allowed_headers: AllowedHeaders,
    
    /// Enable including credentials (cookies, auth headers)
    /// - `None`: Unset (use default behavior)
    /// - `Some(true)`: Allow credentials
    /// - `Some(false)`: Explicitly disallow credentials
    pub allowed_credentials: Option<bool>,
    
    /// Preflight response cache duration (seconds)
    /// - `None`: Unset (use default)
    /// - `Some(0)`: Disable caching
    /// - `Some(seconds)`: Cache duration
    pub max_age: Option<u64>,
}

/// Policy for allowed request origins
#[derive(Debug, Clone, PartialEq)]
pub enum AllowedOrigins {
    /// Not configured (use default behavior)
    Unset,
    
    /// Explicitly deny all origins
    None,
    
    /// Allow only specifically listed origins
    Some(HashSet<String>),
    
    /// Allow any origin (use with caution)
    All,
}

/// Policy for allowed HTTP methods
#[derive(Debug, Clone, PartialEq)]
pub enum AllowedMethods {
    /// Not configured (use default methods)
    Unset,
    
    /// Explicitly deny all methods
    None,
    
    /// Allow only specifically listed methods
    Some(HashSet<String>),
    
    /// Allow any HTTP method
    All,
}

/// Policy for allowed HTTP headers
#[derive(Debug, Clone, PartialEq)]
pub enum AllowedHeaders {
    /// Not configured (use default headers)
    Unset,
    
    /// Explicitly deny all headers
    None,
    
    /// Allow only specifically listed headers
    Some(HashSet<String>),
    
    /// Allow any HTTP header
    All,
}

impl Default for AllowedOrigins {
    fn default() -> Self {
        Self::Unset
    }
}

impl Default for AllowedMethods {
    fn default() -> Self {
        Self::Unset
    }
}

impl Default for AllowedHeaders {
    fn default() -> Self {
        Self::Unset
    }
}

impl AllowedOrigins {
    /// Check if an origin is permitted
    ///
    /// # Arguments
    /// * `origin` - Origin header value to check
    ///
    /// # Returns
    /// `true` if origin is allowed, `false` otherwise
    ///
    /// # Behavior
    /// - `Unset`: Treated as `None` (deny all)
    /// - `None`: Deny all
    /// - `Some`: Check against allowlist
    /// - `All`: Allow any origin
    pub fn is_allowed(&self, origin: &str) -> bool {
        match self {
            Self::Unset | Self::None => false,
            Self::Some(origins) => origins.contains(origin),
            Self::All => true,
        }
    }

    /// Add new origin to allowlist
    ///
    /// # Arguments
    /// * `origin` - Origin to add (case-sensitive)
    ///
    /// # Notes
    /// - Converts `Unset` or `None` to `Some` with single origin
    /// - No effect if policy is `All`
    pub fn add_origin(&mut self, origin: String) {
        match self {
            Self::Some(origins) => {
                origins.insert(origin);
            }
            Self::Unset | Self::None => {
                let mut set = HashSet::new();
                set.insert(origin);
                *self = Self::Some(set);
            }
            Self::All => (),
        }
    }

    /// Reset to deny-all policy
    pub fn remove_all(&mut self) {
        *self = Self::None;
    }
}

impl AllowedMethods {
    /// Check if HTTP method is permitted
    ///
    /// # Arguments
    /// * `method` - HTTP method to check (case-insensitive)
    ///
    /// # Behavior
    /// - `Unset`: Treated as default allowlist (common HTTP methods)
    /// - `None`: Deny all methods
    /// - `Some`: Check against allowlist
    /// - `All`: Allow any method
    pub fn is_allowed(&self, method: &str) -> bool {
        let method_upper = method.to_uppercase();
        match self {
            Self::Unset => DEFAULT_METHODS.contains(&method_upper.as_str()),
            Self::None => false,
            Self::Some(methods) => methods.contains(&method_upper),
            Self::All => true,
        }
    }

    /// Add new method to allowlist
    ///
    /// # Arguments
    /// * `method` - HTTP method to add (converted to uppercase)
    ///
    /// # Notes
    /// - Converts `Unset` or `None` to `Some` with single method
    /// - No effect if policy is `All`
    pub fn add_method(&mut self, method: String) {
        let method_upper = method.to_uppercase();
        match self {
            Self::Some(methods) => {
                methods.insert(method_upper);
            }
            Self::Unset | Self::None => {
                let mut set = HashSet::new();
                set.insert(method_upper);
                *self = Self::Some(set);
            }
            Self::All => (),
        }
    }

    /// Reset to deny-all policy
    pub fn remove_all(&mut self) {
        *self = Self::None;
    }
    
    /// Get effective methods (resolve Unset to defaults)
    fn effective_methods(&self) -> HashSet<String> {
        match self {
            Self::Unset => DEFAULT_METHODS.iter().map(|s| s.to_string()).collect(),
            Self::Some(methods) => methods.clone(),
            Self::All | Self::None => HashSet::new(),
        }
    }
}

impl AllowedHeaders {
    /// Check if HTTP header is permitted
    ///
    /// # Arguments
    /// * `header` - Header name to check (case-insensitive)
    ///
    /// # Behavior
    /// - `Unset`: Treated as default allowlist (common HTTP headers)
    /// - `None`: Deny all headers
    /// - `Some`: Check against allowlist
    /// - `All`: Allow any header
    pub fn is_allowed(&self, header: &str) -> bool {
        let header_lower = header.to_lowercase();
        match self {
            Self::Unset => DEFAULT_HEADERS.contains(&header_lower.as_str()),
            Self::None => false,
            Self::Some(headers) => headers.contains(&header_lower),
            Self::All => true,
        }
    }

    /// Add new header to allowlist
    ///
    /// # Arguments
    /// * `header` - Header name to add (converted to lowercase)
    ///
    /// # Notes
    /// - Converts `Unset` or `None` to `Some` with single header
    /// - No effect if policy is `All`
    pub fn add_header(&mut self, header: String) {
        let header_lower = header.to_lowercase();
        match self {
            Self::Some(headers) => {
                headers.insert(header_lower);
            }
            Self::Unset | Self::None => {
                let mut set = HashSet::new();
                set.insert(header_lower);
                *self = Self::Some(set);
            }
            Self::All => (),
        }
    }

    /// Reset to deny-all policy
    pub fn remove_all(&mut self) {
        *self = Self::None;
    }
    
    /// Get effective headers (resolve Unset to defaults)
    fn effective_headers(&self) -> HashSet<String> {
        match self {
            Self::Unset => DEFAULT_HEADERS.iter().map(|s| s.to_string()).collect(),
            Self::Some(headers) => headers.clone(),
            Self::All | Self::None => HashSet::new(),
        }
    }
}

impl AppCorsSettings {
    /// Create new CORS settings with unset defaults
    pub fn new() -> Self {
        Self::default()
    } 

    pub fn allowed_origins(mut self, allowed_origins: AllowedOrigins) -> Self {
        self.allowed_origins = allowed_origins;
        self
    } 

    pub fn allowed_methods(mut self, allowed_methods: AllowedMethods) -> Self {
        self.allowed_methods = allowed_methods;
        self
    } 
    
    pub fn allowed_headers(mut self, allowed_headers: AllowedHeaders) -> Self {
        self.allowed_headers = allowed_headers;
        self
    } 
    
    pub fn allowed_credentials(mut self, allowed_credentials: bool) -> Self {
        self.allowed_credentials = Some(allowed_credentials);
        self
    } 
    
    pub fn max_age(mut self, max_age: u64) -> Self {
        self.max_age = Some(max_age);
        self
    } 

    /// Merge two CORS configurations
    ///
    /// # Arguments
    /// * `other` - Settings to merge into current configuration
    ///
    /// # Merge Rules
    /// - For enum fields (`allowed_origins`, `methods`, `headers`):
    ///   - `Unset` in `other` retains current value
    ///   - Other variants override current value
    /// - For option fields (`allowed_credentials`, `max_age`):
    ///   - `None` in `other` retains current value
    ///   - `Some(value)` overrides current value
    ///
    /// # Example
    /// ```
    /// let base = AppCorsSettings::default();
    /// let override_settings = AppCorsSettings {
    ///     allowed_origins: AllowedOrigins::All,
    ///     allowed_credentials: Some(true),
    ///     ..Default::default()
    /// };
    ///
    /// let merged = base.merge(&override_settings);
    /// ```
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            allowed_origins: match &other.allowed_origins {
                AllowedOrigins::Unset => self.allowed_origins.clone(),
                _ => other.allowed_origins.clone(),
            },
            allowed_methods: match &other.allowed_methods {
                AllowedMethods::Unset => self.allowed_methods.clone(),
                _ => other.allowed_methods.clone(),
            },
            allowed_headers: match &other.allowed_headers {
                AllowedHeaders::Unset => self.allowed_headers.clone(),
                _ => other.allowed_headers.clone(),
            },
            allowed_credentials: other.allowed_credentials.or(self.allowed_credentials),
            max_age: other.max_age.or(self.max_age),
        }
    }
    
    /// Generate CORS headers based on configuration
    ///
    /// # Arguments
    /// * `origin` - The origin from the request header
    /// * `is_preflight` - Whether this is for a preflight request
    ///
    /// # Returns
    /// Vector of (header, value) pairs
    ///
    /// # Header Generation Rules
    /// - `Access-Control-Allow-Origin`: 
    ///   - `All`: "*" (unless credentials allowed)
    ///   - `Some`: Specific origin if allowed
    /// - `Access-Control-Allow-Credentials`: Only if credentials allowed
    /// - Preflight-specific headers:
    ///   - `Access-Control-Allow-Methods`: Effective methods
    ///   - `Access-Control-Allow-Headers`: Effective headers
    ///   - `Access-Control-Max-Age`: Cache duration
    pub fn write_headers(&self, origin: &str, is_preflight: bool) -> Vec<(String, String)> {
        let mut headers = Vec::new();
        
        // Access-Control-Allow-Origin
        match &self.allowed_origins {
            AllowedOrigins::All => {
                // Cannot use wildcard if credentials are allowed
                if self.allowed_credentials == Some(true) {
                    headers.push(("Access-Control-Allow-Origin".into(), origin.to_string()));
                } else {
                    headers.push(("Access-Control-Allow-Origin".into(), "*".into()));
                }
            }
            AllowedOrigins::Some(origins) if origins.contains(origin) => {
                headers.push(("Access-Control-Allow-Origin".into(), origin.to_string()));
            }
            _ => {
                // If not explicitly allowed, don't set header (browser will block)
            }
        }
        
        // Access-Control-Allow-Credentials
        if self.allowed_credentials == Some(true) {
            headers.push(("Access-Control-Allow-Credentials".into(), "true".into()));
        }
        
        // Preflight-specific headers
        if is_preflight {
            // Access-Control-Allow-Methods
            let methods = self.allowed_methods.effective_methods();
            if !methods.is_empty() {
                let methods_str = methods.into_iter().collect::<Vec<_>>().join(", ");
                headers.push(("Access-Control-Allow-Methods".into(), methods_str));
            }
            
            // Access-Control-Allow-Headers
            let header_names = self.allowed_headers.effective_headers();
            if !header_names.is_empty() {
                let headers_str = header_names.into_iter().collect::<Vec<_>>().join(", ");
                headers.push(("Access-Control-Allow-Headers".into(), headers_str));
            }
            
            // Access-Control-Max-Age
            if let Some(age) = self.max_age.or(Some(DEFAULT_MAX_AGE)) {
                headers.push(("Access-Control-Max-Age".into(), age.to_string()));
            }
        }
        
        headers
    }
}

impl Default for AppCorsSettings {
    /// Create default CORS settings
    ///
    /// - All fields unset
    /// - Will use default behaviors when resolved
    fn default() -> Self {
        Self {
            allowed_origins: AllowedOrigins::Unset,
            allowed_methods: AllowedMethods::Unset,
            allowed_headers: AllowedHeaders::Unset,
            allowed_credentials: None,
            max_age: None,
        }
    }
} 
 

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_settings() {
        let base = AppCorsSettings {
            allowed_origins: AllowedOrigins::Some(vec!["https://base.com".into()].into_iter().collect()),
            allowed_methods: AllowedMethods::Some(vec!["GET".into()].into_iter().collect()),
            allowed_headers: AllowedHeaders::Unset,
            allowed_credentials: Some(false),
            max_age: Some(300),
        };
        
        let override_settings = AppCorsSettings {
            allowed_origins: AllowedOrigins::All,
            allowed_methods: AllowedMethods::Unset,
            allowed_headers: AllowedHeaders::All,
            allowed_credentials: None,
            max_age: Some(600),
        };
        
        let merged = base.merge(&override_settings);
        
        assert!(matches!(merged.allowed_origins, AllowedOrigins::All));
        assert!(matches!(merged.allowed_methods, AllowedMethods::Some(_)));
        assert!(matches!(merged.allowed_headers, AllowedHeaders::All));
        assert_eq!(merged.allowed_credentials, Some(false));
        assert_eq!(merged.max_age, Some(600));
    }
    
    #[test]
    fn test_write_headers() {
        let settings = AppCorsSettings {
            allowed_origins: AllowedOrigins::Some(vec!["https://trusted.com".into()].into_iter().collect()),
            allowed_methods: AllowedMethods::Unset,
            allowed_headers: AllowedHeaders::Unset,
            allowed_credentials: Some(true),
            max_age: None,
        };
        
        // Simple request
        let headers = settings.write_headers("https://trusted.com", false);
        assert_eq!(headers.len(), 2);
        assert!(headers.iter().any(|(k, v)| k == "Access-Control-Allow-Origin" && v == "https://trusted.com"));
        assert!(headers.iter().any(|(k, v)| k == "Access-Control-Allow-Credentials" && v == "true"));
        
        // Preflight request
        let headers = settings.write_headers("https://trusted.com", true);
        assert_eq!(headers.len(), 5);
        assert!(headers.iter().any(|(k, v)| k == "Access-Control-Allow-Methods"));
        assert!(headers.iter().any(|(k, v)| k == "Access-Control-Allow-Headers"));
        assert!(headers.iter().any(|(k, v)| k == "Access-Control-Max-Age"));
    }
    
    #[test]
    fn test_effective_values() {
        // Test Unset resolution to defaults
        let methods = AllowedMethods::Unset;
        assert!(methods.is_allowed("GET"));
        assert!(!methods.is_allowed("CUSTOM"));
        
        let headers = AllowedHeaders::Unset;
        assert!(headers.is_allowed("Content-Type"));
        assert!(!headers.is_allowed("X-Custom"));
        
        // Test effective methods/headers
        let settings = AppCorsSettings::default();
        let headers = settings.write_headers("https://any.com", true);
        
        let methods_header = headers.iter()
            .find(|(k, _)| k == "Access-Control-Allow-Methods")
            .map(|(_, v)| v)
            .unwrap();
        
        assert!(DEFAULT_METHODS.iter().all(|m| methods_header.contains(m)));
        
        let headers_header = headers.iter()
            .find(|(k, _)| k == "Access-Control-Allow-Headers")
            .map(|(_, v)| v)
            .unwrap();
        
        assert!(DEFAULT_HEADERS.iter().all(|h| headers_header.contains(h)));
    }
} 
