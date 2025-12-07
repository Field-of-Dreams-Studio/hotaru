//! # HTTP Encoding
//!
//! This module provides types and functionality for working with HTTP encoding mechanisms,
//! specifically Transfer-Encoding and Content-Encoding as defined in HTTP standards.
//!
//! ## Overview
//!
//! HTTP allows for various encoding mechanisms:
//!
//! - **Transfer-Encoding**: Specifies the form in which the message body is transferred
//!   between HTTP nodes. The most common is "chunked" encoding.
//!
//! - **Content-Encoding**: Specifies how the content is compressed, such as gzip,
//!   deflate, or brotli.
//!
//! This module provides strongly-typed representations of these encodings with proper
//! validation according to HTTP standards.
//!
//! ## Examples
//!
//! ```
//! use hotaru_core::http::encoding::HttpEncoding;
//!
//! // Parse from headers
//! let encoding = HttpEncoding::from_headers(
//!     Some("chunked".to_string()),
//!     Some("br".to_string())
//! );
//!
//! // Check if chunked encoding is used
//! assert!(encoding.transfer().is_chunked());
//!
//! // Serialize back to headers
//! let (transfer, content) = encoding.to_headers();
//! assert_eq!(transfer, Some("chunked".to_string()));
//! assert_eq!(content, Some("br".to_string()));
//! ```

use hotaru_lib::compression;

/// Represents HTTP transfer coding types as defined in HTTP standards.
///
/// Transfer codings are primarily used to define the message transfer format
/// between HTTP nodes. The most common is "chunked" encoding.
#[derive(Debug, Clone, PartialEq)]
pub enum TransferCoding {
    /// Chunked transfer encoding, where the message body is divided into a series
    /// of chunks, each with its own size indicator.
    Chunked,
    
    /// Any other transfer encoding not explicitly defined in this enum.
    Other(Box<str>),
}

impl TransferCoding {
    /// Creates a new `TransferCoding` from a string.
    ///
    /// The string is trimmed and converted to lowercase before matching.
    ///
    /// # Arguments
    ///
    /// * `s` - The string representation of the transfer coding
    ///
    /// # Returns
    ///
    /// A `TransferCoding` variant corresponding to the provided string
    ///
    /// # Examples
    ///
    /// ```
    /// use hotaru_core::http::encoding::TransferCoding;
    ///
    /// let coding = TransferCoding::from_string("chunked");
    /// assert!(matches!(coding, TransferCoding::Chunked));
    ///
    /// let coding = TransferCoding::from_string("compress");
    /// assert!(matches!(coding, TransferCoding::Other(_)));
    /// ```
    pub fn from_string(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "chunked" => TransferCoding::Chunked,
            other => TransferCoding::Other(other.into()),
        }
    }

    /// Returns the string representation of this transfer coding.
    ///
    /// # Returns
    ///
    /// A string slice representing the transfer coding
    ///
    /// # Examples
    ///
    /// ```
    /// use hotaru_core::http::encoding::TransferCoding;
    ///
    /// let coding = TransferCoding::Chunked;
    /// assert_eq!(coding.as_str(), "chunked");
    ///
    /// let coding = TransferCoding::Other("custom".into());
    /// assert_eq!(coding.as_str(), "custom");
    /// ```
    pub fn as_str(&self) -> &str {
        match self {
            Self::Chunked => "chunked",
            Self::Other(s) => s,
        }
    }
}

/// Represents HTTP content coding types as defined in HTTP standards.
///
/// Content codings are compression algorithms applied to the message body.
#[derive(Debug, Clone, PartialEq)]
pub enum ContentCoding {
    /// gzip compression algorithm
    Gzip,
    
    /// deflate compression algorithm
    Deflate,
    
    /// compress compression algorithm
    Compress,
    
    /// Brotli compression algorithm (represented as "br" in HTTP headers)
    Brotli,
    
    /// Zstandard compression algorithm (represented as "zstd" in HTTP headers)
    Zstd,
    
    /// Any other content coding not explicitly defined in this enum
    Other(Box<str>),
}

impl ContentCoding {
    /// Creates a new `ContentCoding` from a string.
    ///
    /// The string is trimmed and converted to lowercase before matching.
    ///
    /// # Arguments
    ///
    /// * `s` - The string representation of the content coding
    ///
    /// # Returns
    ///
    /// A `ContentCoding` variant corresponding to the provided string
    ///
    /// # Examples
    ///
    /// ```
    /// use hotaru_core::http::encoding::ContentCoding;
    ///
    /// let coding = ContentCoding::from_string("gzip");
    /// assert!(matches!(coding, ContentCoding::Gzip));
    ///
    /// let coding = ContentCoding::from_string("br");
    /// assert!(matches!(coding, ContentCoding::Brotli));
    /// ```
    pub fn from_string(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "gzip" => ContentCoding::Gzip,
            "deflate" => ContentCoding::Deflate,
            "compress" => ContentCoding::Compress,
            "br" => ContentCoding::Brotli,
            "zstd" => ContentCoding::Zstd,
            other => ContentCoding::Other(other.into()),
        }
    }

    /// Returns the string representation of this content coding.
    ///
    /// # Returns
    ///
    /// A string slice representing the content coding
    ///
    /// # Examples
    ///
    /// ```
    /// use hotaru_core::http::encoding::ContentCoding;
    ///
    /// let coding = ContentCoding::Gzip;
    /// assert_eq!(coding.as_str(), "gzip");
    ///
    /// let coding = ContentCoding::Brotli;
    /// assert_eq!(coding.as_str(), "br");
    /// ```
    pub fn as_str(&self) -> &str {
        match self {
            Self::Gzip => "gzip",
            Self::Deflate => "deflate",
            Self::Compress => "compress",
            Self::Brotli => "br",
            Self::Zstd => "zstd",
            Self::Other(s) => s,
        }
    } 

    pub fn decode_compressed(encoding: &ContentCoding, data: &[u8]) -> std::io::Result<Vec<u8>> {
        match encoding {
            ContentCoding::Gzip => compression::decompress_gzip(data),
            ContentCoding::Deflate => compression::decompress_deflate(data),
            ContentCoding::Brotli => compression::decompress_brotli(data),
            ContentCoding::Zstd => compression::decompress_zstd(data),
            ContentCoding::Compress => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "compress encoding not supported",
            )),
            _ => Ok(data.to_vec()), // Identity or unsupported
        }
    } 

    pub fn encode_compressed(encoding: &ContentCoding, data: &[u8]) -> std::io::Result<Vec<u8>> {
        match encoding {
            ContentCoding::Gzip => compression::compress_gzip(data),
            ContentCoding::Deflate => compression::compress_deflate(data),
            ContentCoding::Brotli => compression::compress_brotli(data),
            ContentCoding::Zstd => compression::compress_zstd(data, 1),
            ContentCoding::Compress => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "compress encoding not supported",
            )),
            _ => Ok(data.to_vec()), // Identity or unsupported
        }
    } 
}

/// A collection of transfer codings with validation according to HTTP standards.
///
/// This struct ensures that:
/// - "chunked" appears at most once
/// - "chunked" is always the last transfer coding
#[derive(Debug, Clone, Default)]
pub struct TransferCodings {
    codings: Vec<TransferCoding>,
}

impl TransferCodings {
    /// Creates a new empty `TransferCodings` collection.
    ///
    /// # Returns
    ///
    /// A new `TransferCodings` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use hotaru_core::http::encoding::TransferCodings;
    ///
    /// let codings = TransferCodings::new();
    /// assert!(codings.is_identity());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a transfer coding to the collection, with validation.
    ///
    /// According to HTTP standards:
    /// - "chunked" can appear at most once
    /// - "chunked" must be the last transfer coding
    ///
    /// # Arguments
    ///
    /// * `coding` - The transfer coding to add
    ///
    /// # Returns
    ///
    /// `Ok(())` if the coding was successfully added, or an error message
    /// explaining why the coding could not be added.
    ///
    /// # Examples
    ///
    /// ```
    /// use hotaru_core::http::encoding::{TransferCodings, TransferCoding};
    ///
    /// let mut codings = TransferCodings::new();
    ///
    /// // Add a non-chunked coding
    /// codings.push(TransferCoding::Other("gzip".into())).unwrap();
    ///
    /// // Add chunked coding (must be last)
    /// codings.push(TransferCoding::Chunked).unwrap();
    ///
    /// // Cannot add another coding after chunked
    /// assert!(codings.push(TransferCoding::Other("compress".into())).is_err());
    ///
    /// // Cannot add chunked twice
    /// let mut codings = TransferCodings::new();
    /// codings.push(TransferCoding::Chunked).unwrap();
    /// assert!(codings.push(TransferCoding::Chunked).is_err());
    /// ```
    pub fn push(&mut self, coding: TransferCoding) -> Result<(), &'static str> {
        if matches!(coding, TransferCoding::Chunked) {
            if self.codings.iter().any(|c| matches!(c, TransferCoding::Chunked)) {
                return Err("chunked can only appear once");
            }
        } else if self.codings.last().is_some_and(|c| matches!(c, TransferCoding::Chunked)) {
            return Err("no coding can follow chunked");
        }
        
        self.codings.push(coding);
        Ok(())
    }

    /// Checks if chunked transfer encoding is used.
    ///
    /// # Returns
    ///
    /// `true` if chunked encoding is present, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use hotaru_core::http::encoding::{TransferCodings, TransferCoding};
    ///
    /// let mut codings = TransferCodings::new();
    /// assert!(!codings.is_chunked());
    ///
    /// codings.push(TransferCoding::Chunked).unwrap();
    /// assert!(codings.is_chunked());
    /// ```
    pub fn is_chunked(&self) -> bool {
        self.codings.iter().any(|c| matches!(c, TransferCoding::Chunked))
    }

    /// Checks if identity transfer encoding is used (no transfer encoding).
    ///
    /// # Returns
    ///
    /// `true` if no transfer encodings are present, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use hotaru_core::http::encoding::{TransferCodings, TransferCoding};
    ///
    /// let mut codings = TransferCodings::new();
    /// assert!(codings.is_identity());
    ///
    /// codings.push(TransferCoding::Chunked).unwrap();
    /// assert!(!codings.is_identity());
    /// ```
    pub fn is_identity(&self) -> bool {
        self.codings.is_empty()
    }

    /// Converts the transfer codings to a header value string.
    ///
    /// # Returns
    ///
    /// A comma-separated string of transfer codings
    ///
    /// # Examples
    ///
    /// ```
    /// use hotaru_core::http::encoding::{TransferCodings, TransferCoding};
    ///
    /// let mut codings = TransferCodings::new();
    /// codings.push(TransferCoding::Other("gzip".into())).unwrap();
    /// codings.push(TransferCoding::Chunked).unwrap();
    ///
    /// assert_eq!(codings.to_header(), "gzip, chunked");
    /// ```
    pub fn to_header(&self) -> String {
        self.codings
            .iter()
            .map(|c| c.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// A collection of content codings.
#[derive(Debug, Clone, Default)]
pub struct ContentCodings {
    codings: Vec<ContentCoding>,
}

impl ContentCodings {
    /// Creates a new empty `ContentCodings` collection.
    ///
    /// # Returns
    ///
    /// A new `ContentCodings` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use hotaru_core::http::encoding::ContentCodings;
    ///
    /// let codings = ContentCodings::new();
    /// assert!(codings.is_identity());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a content coding to the collection.
    ///
    /// # Arguments
    ///
    /// * `coding` - The content coding to add
    ///
    /// # Examples
    ///
    /// ```
    /// use hotaru_core::http::encoding::{ContentCodings, ContentCoding};
    ///
    /// let mut codings = ContentCodings::new();
    /// codings.push(ContentCoding::Gzip);
    /// codings.push(ContentCoding::Brotli);
    /// ```
    pub fn push(&mut self, coding: ContentCoding) {
        self.codings.push(coding);
    }

    /// Checks if identity content encoding is used (no content encoding).
    ///
    /// # Returns
    ///
    /// `true` if no content encodings are present, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use hotaru_core::http::encoding::{ContentCodings, ContentCoding};
    ///
    /// let mut codings = ContentCodings::new();
    /// assert!(codings.is_identity());
    ///
    /// codings.push(ContentCoding::Gzip);
    /// assert!(!codings.is_identity());
    /// ```
    pub fn is_identity(&self) -> bool {
        self.codings.is_empty()
    }

    /// Converts the content codings to a header value string.
    ///
    /// # Returns
    ///
    /// A comma-separated string of content codings
    ///
    /// # Examples
    ///
    /// ```
    /// use hotaru_core::http::encoding::{ContentCodings, ContentCoding};
    ///
    /// let mut codings = ContentCodings::new();
    /// codings.push(ContentCoding::Gzip);
    /// codings.push(ContentCoding::Brotli);
    ///
    /// assert_eq!(codings.to_header(), "gzip, br");
    /// ```
    pub fn to_header(&self) -> String {
        self.codings
            .iter()
            .map(|c| c.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    } 

    /// Decodes compressed data using the content codings in this collection. 
    /// 
    /// # Arguments 
    /// 
    /// * `data` - The compressed data to decode 
    /// 
    /// # Returns 
    /// 
    /// A `Result` containing the decompressed data as a `Vec<u8>`, or an error if decoding fails. 
    /// 
    /// # Examples 
    /// 
    /// ```
    /// use hotaru_core::http::encoding::ContentCodings;
    /// let codings = ContentCodings::new();
    /// let data = b"hello".to_vec();
    /// let result = codings.decode_compressed(data.clone()).unwrap();
    /// assert_eq!(result, data);
    /// ```
    pub fn decode_compressed(&self, data: Vec<u8>) -> std::io::Result<Vec<u8>> {
        if self.is_identity() {
            return Ok(data);
        }

        let mut result = data;
        // Decompress in REVERSE order (last applied first)
        for coding in self.codings.iter().rev() {
            result = ContentCoding::decode_compressed(coding, &result)?;
        }
        Ok(result)
    } 

    /// Encodes data using the content codings in this collection. 
    /// 
    /// # Arguments
    /// 
    /// * `data` - The data to encode 
    /// 
    /// # Returns
    /// 
    /// A `Result` containing the encoded data as a `Vec<u8>`, or an error if encoding fails.
    /// 
    /// # Examples
    /// ```
    /// use hotaru_core::http::encoding::ContentCodings;
    /// let codings = ContentCodings::new();
    /// let data = b"hello".to_vec();
    /// let result = codings.encode_compressed(data.clone()).unwrap();
    /// assert_eq!(result, data);
    /// ```
    pub fn encode_compressed(&self, data: Vec<u8>) -> std::io::Result<Vec<u8>> {
        if self.is_identity() {
            return Ok(data);
        }

        let mut result = data;
        // Compress in ORDER (first applied first)
        for coding in &self.codings {
            result = ContentCoding::encode_compressed(coding, &result)?;
        }
        Ok(result)
    } 
}

/// Combines HTTP transfer and content encodings into a single structure.
///
/// This struct handles both Transfer-Encoding and Content-Encoding HTTP headers.
#[derive(Debug, Clone, Default)]
pub struct HttpEncoding {
    transfer: TransferCodings,
    content: ContentCodings,
}

impl HttpEncoding {
    /// Creates a new `HttpEncoding` from HTTP header values.
    ///
    /// # Arguments
    ///
    /// * `transfer_header` - Optional Transfer-Encoding header value
    /// * `content_header` - Optional Content-Encoding header value
    ///
    /// # Returns
    ///
    /// A new `HttpEncoding` instance parsed from the provided headers
    ///
    /// # Examples
    ///
    /// ```
    /// use hotaru_core::http::encoding::HttpEncoding;
    ///
    /// let encoding = HttpEncoding::from_headers(
    ///     Some("chunked, gzip".to_string()),
    ///     Some("br".to_string())
    /// );
    ///
    /// assert!(encoding.transfer().is_chunked());
    /// assert!(!encoding.content().is_identity());
    /// ```
    pub fn from_headers(
        transfer_header: Option<String>,
        content_header: Option<String>,
    ) -> Self {
        let mut transfer = TransferCodings::new();
        let mut content = ContentCodings::new();

        if let Some(header) = transfer_header {
            for part in header.split(',') {
                if !part.trim().is_empty() {
                    let coding = TransferCoding::from_string(part);
                    if let Err(e) = transfer.push(coding) {
                        eprintln!("[WARN] Invalid Transfer-Encoding: {}", e);
                    }
                }
            }
        }

        if let Some(header) = content_header {
            for part in header.split(',') {
                if !part.trim().is_empty() {
                    content.push(ContentCoding::from_string(part));
                }
            }
        }

        Self { transfer, content }
    }

    /// Converts the HTTP encodings to header values.
    ///
    /// # Returns
    ///
    /// A tuple of optional strings representing the Transfer-Encoding and
    /// Content-Encoding header values. If an encoding is identity (empty),
    /// its corresponding header value will be None.
    ///
    /// # Examples
    ///
    /// ```
    /// use hotaru_core::http::encoding::{HttpEncoding, TransferCoding, ContentCoding};
    ///
    /// let mut encoding = HttpEncoding::from_headers(
    ///     Some("chunked".to_string()),
    ///     Some("gzip".to_string())
    /// );
    ///
    /// let (transfer, content) = encoding.to_headers();
    /// assert_eq!(transfer, Some("chunked".to_string()));
    /// assert_eq!(content, Some("gzip".to_string()));
    /// ```
    pub fn to_headers(&self) -> (Option<String>, Option<String>) {
        let transfer = if !self.transfer.is_identity() {
            Some(self.transfer.to_header())
        } else {
            None
        };

        let content = if !self.content.is_identity() {
            Some(self.content.to_header())
        } else {
            None
        };

        (transfer, content)
    }

    /// Returns a reference to the transfer codings.
    ///
    /// # Returns
    ///
    /// A reference to the `TransferCodings` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use hotaru_core::http::encoding::HttpEncoding;
    ///
    /// let encoding = HttpEncoding::from_headers(
    ///     Some("chunked".to_string()),
    ///     None
    /// );
    ///
    /// assert!(encoding.transfer().is_chunked());
    /// ```
    pub fn transfer(&self) -> &TransferCodings {
        &self.transfer
    }

    /// Returns a reference to the content codings.
    ///
    /// # Returns
    ///
    /// A reference to the `ContentCodings` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use hotaru_core::http::encoding::HttpEncoding;
    ///
    /// let encoding = HttpEncoding::from_headers(
    ///     None,
    ///     Some("gzip, br".to_string())
    /// );
    ///
    /// assert!(!encoding.content().is_identity());
    /// ```
    pub fn content(&self) -> &ContentCodings {
        &self.content
    }
} 
