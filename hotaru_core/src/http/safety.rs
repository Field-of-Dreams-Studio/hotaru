use super::http_value::{HttpContentType, HttpMethod};

/// Centralized HTTP safety configuration with explicit state tracking
///
/// Tracks whether each parameter has been explicitly set or should use its default value.
/// Provides granular control over HTTP validation with secure defaults.
///
/// # Security Architecture
///
/// ## Request Timeout Handling
/// Request timeouts are NOT implemented at the HttpSafety level. This is an architectural
/// decision for efficiency:
/// - Timeout is enforced once at the APP/connection layer when accepting connections
/// - Per-request timeouts would be redundant and waste resources
/// - Connection-level timeout protects against slowloris attacks effectively
/// - This design choice means: one timeout check per connection, not per HTTP operation
///
/// If per-request timeout is needed for specific use cases, it should be implemented
/// as middleware at the application layer, not in the core framework.
///
/// ## Core Security: Size Limits
/// The primary security mechanism is enforcing size limits with secure defaults:
/// - max_body_size: 10MB (prevents memory exhaustion attacks)
/// - max_header_size: 1MB (prevents header bomb attacks)
/// - max_line_length: 64KB (prevents single-line DoS)
/// - max_headers: 100 (prevents header count DoS)
///
/// Method and content-type filtering are intentionally permissive by default, as these
/// are application-level concerns, not framework security concerns.
#[derive(Debug, Clone)]
pub struct HttpSafety {
    /// Maximum request body size (None = use default)
    max_body_size: Option<usize>,

    /// Allowed HTTP methods (None = allow all methods)
    allowed_methods: Option<Vec<HttpMethod>>,

    /// Allowed content types (None = allow all content types)
    allowed_content_types: Option<Vec<HttpContentType>>,

    /// Maximum header section size (None = use default)
    max_header_size: Option<usize>,

    /// Maximum header line length (None = use default)
    max_line_length: Option<usize>,

    /// Maximum number of headers (None = use default)
    max_headers: Option<usize>,
}

// Default constants for safety parameters
const DEFAULT_MAX_BODY_SIZE: usize = 10 * 1024 * 1024; // 10 MB
const DEFAULT_MAX_HEADER_SIZE: usize = 1024 * 1024; // 1 MB
const DEFAULT_MAX_LINE_LENGTH: usize = 1024 * 64; // 64 KB
const DEFAULT_MAX_HEADERS: usize = 100; // 100 headers

impl HttpSafety {
    // --------------------------------------------------
    // Constructor and Defaults
    // --------------------------------------------------

    /// Creates a new `HttpSafety` instance with all parameters unset
    ///
    /// # Examples
    /// ```
    /// # use hotaru_core::http::safety::HttpSafety;
    /// let safety = HttpSafety::new();
    /// assert!(safety.max_body_size().is_none());
    /// ```
    pub fn new() -> Self {
        Self {
            max_body_size: None,
            allowed_methods: None,
            allowed_content_types: None,
            max_header_size: None,
            max_line_length: None,
            max_headers: None,
        }
    }

    /// Returns the effective body size limit (set value or default)
    fn effective_max_body_size(&self) -> usize {
        self.max_body_size.unwrap_or(DEFAULT_MAX_BODY_SIZE)
    }

    /// Returns the effective header size limit (set value or default)
    fn effective_max_header_size(&self) -> usize {
        self.max_header_size.unwrap_or(DEFAULT_MAX_HEADER_SIZE)
    }

    /// Returns the effective line length limit (set value or default)
    fn effective_max_line_length(&self) -> usize {
        self.max_line_length.unwrap_or(DEFAULT_MAX_LINE_LENGTH)
    }

    /// Returns the effective headers count limit (set value or default)
    fn effective_max_headers(&self) -> usize {
        self.max_headers.unwrap_or(DEFAULT_MAX_HEADERS)
    }

    // --------------------------------------------------
    // Body Size Configuration
    // --------------------------------------------------

    /// Gets the explicitly set body size limit (None if unset)
    pub fn max_body_size(&self) -> Option<usize> {
        self.max_body_size
    }

    /// Sets the body size limit explicitly
    pub fn set_max_body_size(&mut self, size: Option<usize>) {
        self.max_body_size = size;
    }

    /// Gets the effective body size limit (always returns a value)
    pub fn effective_body_size(&self) -> usize {
        self.effective_max_body_size()
    }

    /// Checks if a body size is within effective limits
    pub fn check_body_size(&self, size: usize) -> bool {
        size <= self.effective_max_body_size()
    }

    // --------------------------------------------------
    // Method Allow List Configuration
    // --------------------------------------------------

    /// Gets the allowed methods list (None if unset = allow all)
    pub fn allowed_methods(&self) -> Option<&[HttpMethod]> {
        self.allowed_methods.as_deref()
    }

    /// Sets the allowed methods list
    pub fn set_allowed_methods(&mut self, methods: Option<Vec<HttpMethod>>) {
        self.allowed_methods = methods;
    }

    /// Adds a method to the allow list
    pub fn add_method(&mut self, method: HttpMethod) {
        let methods = self.allowed_methods.get_or_insert_with(Vec::new);
        if !methods.contains(&method) {
            methods.push(method);
        }
    }

    /// Checks if a method is allowed
    ///
    /// # Security Note: Allow-all default is intentional
    /// The `None => true` case (allow all methods) is a deliberate design choice, NOT a security
    /// vulnerability. Real security comes from size limits (max_body_size, max_header_size),
    /// which are enforced by default (10MB body, 1MB headers).
    ///
    /// Method filtering is application-level business logic, not a framework security concern.
    /// Attackers don't need specific HTTP methods to cause damage - they focus on:
    /// - Sending huge request bodies → Protected by max_body_size
    /// - Sending huge headers → Protected by max_header_size
    /// - Slowloris attacks → Handled by connection timeout at APP layer
    ///
    /// Applications should explicitly set allowed_methods only when business logic requires
    /// restricting operations (e.g., read-only API allowing only GET), not for security.
    pub fn check_method(&self, method: &HttpMethod) -> bool {
        match &self.allowed_methods {
            Some(methods) => methods.contains(method),
            None => true, // No restrictions - allow all methods (see security note above)
        }
    }

    // --------------------------------------------------
    // Content Type Allow List Configuration
    // --------------------------------------------------

    /// Gets the allowed content types list (None if unset = allow all)
    pub fn allowed_content_types(&self) -> Option<&[HttpContentType]> {
        self.allowed_content_types.as_deref()
    }

    /// Sets the allowed content types list
    pub fn set_allowed_content_types(&mut self, types: Option<Vec<HttpContentType>>) {
        self.allowed_content_types = types;
    }

    /// Adds a content type to the allow list
    pub fn add_content_type(&mut self, content_type: HttpContentType) {
        let types = self.allowed_content_types.get_or_insert_with(Vec::new);
        if !types.contains(&content_type) {
            types.push(content_type);
        }
    }

    /// Checks if a content type is allowed
    ///
    /// # Security Note: Content-type filtering is application logic, not security
    /// Similar to method filtering, allowing all content types by default is safe because:
    /// 1. Content-Type header doesn't control actual payload content
    /// 2. Attackers can send any header value regardless of restrictions
    /// 3. Real validation happens during parsing (JSON, form data, etc.)
    /// 4. Size limits (max_body_size) prevent payload-based attacks
    ///
    /// Applications should use allowed_content_types for API contract enforcement
    /// (e.g., "this endpoint only accepts JSON"), not as a security mechanism.
    pub fn check_content_type(&self, content_type: &HttpContentType) -> bool {
        match &self.allowed_content_types {
            Some(types) => types.contains(content_type),
            None => true, // No restrictions - allow all content types (see security note above)
        }
    }

    // --------------------------------------------------
    // Header Size Configuration
    // --------------------------------------------------

    /// Gets the header size limit (None if unset)
    pub fn max_header_size(&self) -> Option<usize> {
        self.max_header_size
    }

    /// Sets the header size limit explicitly
    pub fn set_max_header_size(&mut self, size: Option<usize>) {
        self.max_header_size = size;
    }

    /// Gets the effective header size limit (always returns a value)
    pub fn effective_header_size(&self) -> usize {
        self.effective_max_header_size()
    }

    /// Checks if header size is within effective limits
    pub fn check_header_size(&self, size: usize) -> bool {
        size <= self.effective_max_header_size()
    }

    // --------------------------------------------------
    // Line Length Configuration
    // --------------------------------------------------

    /// Gets the line length limit (None if unset)
    pub fn max_line_length(&self) -> Option<usize> {
        self.max_line_length
    }

    /// Sets the line length limit explicitly
    pub fn set_max_line_length(&mut self, size: Option<usize>) {
        self.max_line_length = size;
    }

    /// Gets the effective line length limit (always returns a value)
    pub fn effective_line_length(&self) -> usize {
        self.effective_max_line_length()
    }

    /// Checks if line length is within effective limits
    pub fn check_line_length(&self, size: usize) -> bool {
        size <= self.effective_max_line_length()
    }

    // --------------------------------------------------
    // Header Count Configuration
    // --------------------------------------------------

    /// Gets the headers count limit (None if unset)
    pub fn max_headers(&self) -> Option<usize> {
        self.max_headers
    }

    /// Sets the headers count limit explicitly
    pub fn set_max_headers(&mut self, size: Option<usize>) {
        self.max_headers = size;
    }

    /// Gets the effective headers count limit (always returns a value)
    pub fn effective_headers_count(&self) -> usize {
        self.effective_max_headers()
    }

    /// Checks if headers count is within effective limits
    pub fn check_headers_count(&self, count: usize) -> bool {
        count <= self.effective_max_headers()
    }

    // --------------------------------------------------
    // Configuration Merging
    // --------------------------------------------------

    /// Updates explicitly set parameters from another configuration
    ///
    /// Only modifies parameters that are explicitly set in the source.
    /// Preserves unset parameters in the current configuration.
    ///
    /// # Examples
    /// ```
    /// # use hotaru_core::http::safety::HttpSafety;
    /// let mut base = HttpSafety::new();
    /// base.set_max_body_size(Some(1024));
    ///
    /// let mut target = HttpSafety::new();
    /// target.update(&base);
    ///
    /// assert_eq!(target.max_body_size(), Some(1024));
    /// ```
    pub fn update(&mut self, source: &HttpSafety) {
        // Update only explicitly set parameters
        if source.max_body_size.is_some() {
            self.max_body_size = source.max_body_size;
        }
        if source.allowed_methods.is_some() {
            self.allowed_methods = source.allowed_methods.clone();
        }
        if source.allowed_content_types.is_some() {
            self.allowed_content_types = source.allowed_content_types.clone();
        }
        if source.max_header_size.is_some() {
            self.max_header_size = source.max_header_size;
        }
        if source.max_line_length.is_some() {
            self.max_line_length = source.max_line_length;
        }
        if source.max_headers.is_some() {
            self.max_headers = source.max_headers;
        }
    }

    /// Merges another configuration using "most restrictive wins" policy
    ///
    /// # Merge Logic
    /// - **Size Limits**: Takes the minimum value (more restrictive)
    /// - **Allow Lists**: Takes the intersection of allowed values
    /// - **Unset Parameters**: Treated as using default values during merge
    ///
    /// # Examples
    /// ```
    /// # use hotaru_core::http::safety::HttpSafety;
    /// # use hotaru_core::http::http_value::HttpMethod;
    /// let mut global = HttpSafety::new();
    /// global.set_max_body_size(Some(2048));
    /// global.set_allowed_methods(Some(vec![HttpMethod::Get, HttpMethod::Post]));
    ///
    /// let mut route = HttpSafety::new();
    /// route.set_max_body_size(Some(1024));
    /// route.set_allowed_methods(Some(vec![HttpMethod::Post]));
    ///
    /// global.merge(&route);
    ///
    /// assert_eq!(global.max_body_size(), Some(1024));
    /// assert_eq!(
    ///     global.allowed_methods(),
    ///     Some(vec![HttpMethod::Post].as_slice())
    /// );
    /// ```
    pub fn merge(&mut self, other: &HttpSafety) {
        // Merge size limits: take the more restrictive (minimum) value
        self.max_body_size = Some(
            self.effective_max_body_size()
                .min(other.effective_max_body_size()),
        );

        self.max_header_size = Some(
            self.effective_max_header_size()
                .min(other.effective_max_header_size()),
        );

        self.max_line_length = Some(
            self.effective_max_line_length()
                .min(other.effective_max_line_length()),
        );

        self.max_headers = Some(
            self.effective_max_headers()
                .min(other.effective_max_headers()),
        );

        // Merge method allow lists
        self.allowed_methods = match (&self.allowed_methods, &other.allowed_methods) {
            (Some(a), Some(b)) => Some(a.iter().filter(|m| b.contains(m)).cloned().collect()),
            (Some(_), None) => self.allowed_methods.clone(),
            (None, Some(_)) => other.allowed_methods.clone(),
            (None, None) => None,
        };

        // Merge content type allow lists
        self.allowed_content_types =
            match (&self.allowed_content_types, &other.allowed_content_types) {
                (Some(a), Some(b)) => Some(a.iter().filter(|ct| b.contains(ct)).cloned().collect()),
                (Some(_), None) => self.allowed_content_types.clone(),
                (None, Some(_)) => other.allowed_content_types.clone(),
                (None, None) => None,
            };
    }

    // --------------------------------------------------
    // Builder Pattern Methods
    // --------------------------------------------------

    /// Builder method to set body size
    pub fn with_max_body_size(mut self, size: usize) -> Self {
        self.set_max_body_size(Some(size));
        self
    }

    /// Builder method to add a single allowed method
    pub fn with_allowed_method(mut self, method: HttpMethod) -> Self {
        self.add_method(method);
        self
    }

    /// Builder method to set method allow list
    pub fn with_allowed_methods(mut self, methods: Vec<HttpMethod>) -> Self {
        self.set_allowed_methods(Some(methods));
        self
    }

    /// Builder method to add a single allowed content type
    pub fn with_allowed_content_type(mut self, content_type: HttpContentType) -> Self {
        self.add_content_type(content_type);
        self
    }

    /// Builder method to set content type allow list
    pub fn with_allowed_content_types(mut self, types: Vec<HttpContentType>) -> Self {
        self.set_allowed_content_types(Some(types));
        self
    }

    /// Builder method to set header size
    pub fn with_max_header_size(mut self, size: usize) -> Self {
        self.set_max_header_size(Some(size));
        self
    }

    /// Builder method to set line length
    pub fn with_max_line_length(mut self, size: usize) -> Self {
        self.set_max_line_length(Some(size));
        self
    }

    /// Builder method to set headers count
    pub fn with_max_headers(mut self, size: usize) -> Self {
        self.set_max_headers(Some(size));
        self
    }
}

impl Default for HttpSafety {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for &HttpSafety {
    fn default() -> Self {
        static DEFAULT_SAFETY: HttpSafety = HttpSafety {
            max_body_size: None,
            allowed_methods: None,
            allowed_content_types: None,
            max_header_size: None,
            max_line_length: None,
            max_headers: None,
        };
        &DEFAULT_SAFETY
    }
}
