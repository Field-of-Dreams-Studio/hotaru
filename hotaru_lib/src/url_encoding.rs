use percent_encoding::{percent_encode, NON_ALPHANUMERIC, AsciiSet}; 
pub use percent_encoding::percent_decode; 

/// Custom encode set for application/x-www-form-urlencoded allowing unreserved characters including hyphens
const FORM_URLENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~');

/// Encodes a string for URL safety and returns an owned `String`
/// 
/// # Example
/// ```
/// use hotaru_lib::url_encoding::encode_url_owned; 
/// let encoded = encode_url_owned("Hello World!");
/// assert_eq!(encoded, "Hello%20World%21");
/// ```
pub fn encode_url_owned(input: &str) -> String {
    percent_encode(input.as_bytes(), FORM_URLENCODE_SET).to_string()
}

/// Encodes a string in place for URL safety
/// 
/// # Example
/// ```
/// use hotaru_lib::url_encoding::encode_url; 
/// let mut s = String::from("Hello World!");
/// encode_url(&mut s);
/// assert_eq!(s, "Hello%20World%21");
/// ```
pub fn encode_url(input: &mut String) {
    let encoded = encode_url_owned(input);
    *input = encoded;
} 

/// Decodes a URL-encoded string and returns an owned `String`.
///
/// # Arguments
///
/// * `input` - A URL-encoded string as a `&str`.
///
/// # Returns
///
/// A new `String` containing the decoded value.
pub fn decode_url_owned(input: &str) -> String {
    percent_decode(input.as_bytes())
        .decode_utf8_lossy()
        .into_owned() 
}

/// Decodes a URL-encoded string in place by updating the provided `String`.
///
/// # Arguments
///
/// * `input` - A mutable reference to a `String` holding a URL-encoded value.
///   After the call, it will contain the decoded version.
pub fn decode_url(input: &mut String) {
    let decoded = decode_url_owned(input);
    *input = decoded;
} 

/// Determines if a string needs extended encoding.
///
/// A string needs extended encoding if it contains non-ASCII characters or
/// characters that would require special handling in a quoted-string.
pub fn needs_extended_encoding(s: &str) -> bool {
    s.chars().any(|c| c > '\u{7F}' || c == '"' || c == '\\' || c == '%')
} 

/// Unescapes a quoted string according to RFC 2616.
///
/// # Arguments
///
/// * `s` - The string to unescape
///
/// # Returns
///
/// The unescaped string
pub fn unescape_quoted_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    let mut in_quotes = false;
    
    while let Some(c) = chars.next() {
        match c {
            '"' if !in_quotes => in_quotes = true,
            '"' if in_quotes => in_quotes = false,
            '\\' if in_quotes => {
                if let Some(next) = chars.next() {
                    result.push(next);
                }
            }
            _ if in_quotes => result.push(c),
            _ => {} // Skip non-quoted content
        }
    }
    result
} 

/// Escapes a string for use in a quoted-string according to RFC 2616.
///
/// # Arguments
///
/// * `s` - The string to escape
///
/// # Returns
///
/// The escaped string (without quotes)
pub fn escape_quoted_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 2);
    
    for c in s.chars() {
        if c == '"' || c == '\\' {
            result.push('\\');
        }
        result.push(c);
    }
    
    result
}
 