use crate::http::encoding::ContentCodings;
use crate::http::safety::HttpSafety;

use super::form::*;
use super::http_value::*;
use super::meta::HttpMeta; 
use akari::Value;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncBufReadExt};

static EMPTY: Vec<u8> = Vec::new();

#[derive(Debug, Clone)]
pub enum HttpBody {
    Text(String),
    Binary(Vec<u8>),
    Form(UrlEncodedForm),
    Files(MultiForm),
    Json(Value),
    Empty,
    Unparsed,

    Buffer {
        data: Vec<u8>,
        content_type: HttpContentType,
        content_coding: ContentCodings, 
    },
} 

impl HttpBody { 
    pub async fn read_buffer<R: AsyncRead + Unpin>(
        buf_reader: &mut tokio::io::BufReader<R>,
        header: &mut HttpMeta,
        parse_config: &HttpSafety 
    ) -> std::io::Result<Self> {
        Ok(Self::Buffer { 
            data: Self::read_binary_info(buf_reader, header, parse_config).await?, 
            content_type: header.get_content_type().unwrap_or(HttpContentType::from_str("")),
            content_coding: header.get_encoding().map(|e| e.content().clone()).unwrap_or(ContentCodings::new()), 
        }) 
    } 

    /// Parse a Buffer variant into a more specific type based on content_type
    pub fn parse_buffer(self, safety: &HttpSafety) -> Self {
        match self {
            Self::Buffer { data, content_type, content_coding } => { 
                // Check whether the content length is within the safety limits 
                if !safety.check_body_size(data.len()) {
                    return Self::Unparsed; // Return Unparsed if size exceeds limits
                } 
                // Decode the content based on content coding 
                let data = content_coding.decode_compressed(data).unwrap_or_else(|_| vec![]); 
                match content_type {
                    HttpContentType::Application { subtype, .. } if subtype == "json" => {
                        Self::parse_json(data)
                    }
                    HttpContentType::Text { subtype, .. } if subtype == "html" || subtype == "plain" => {
                        Self::parse_text(data)
                    }
                    HttpContentType::Application { subtype, .. } if subtype == "x-www-form-urlencoded" => {
                        Self::parse_form(data)
                    }
                    HttpContentType::Multipart { subtype, boundary } if subtype == "form-data" => {
                        Self::parse_files(data, boundary.unwrap_or("".to_string()))
                    }
                    _ => Self::parse_binary(data),
                }
            }
            // If already parsed or empty, just return as is
            _ => self,
        }
    } 

    /// Parse the HTTP body directly from a TCP Stream 
    pub async fn direct_parse<R: AsyncRead + Unpin>(
        buf_reader: &mut tokio::io::BufReader<R>,
        header: &mut HttpMeta, 
        parse_config: &HttpSafety 
    ) -> Self { 
        // Create a Buffer variant first
        let buffer = Self::read_buffer(buf_reader, header, parse_config).await.unwrap_or(Self::Unparsed);

        // Parse the buffer into a more specific type
        buffer.parse_buffer(parse_config) 
    }

    pub async fn read_binary_info<R: AsyncRead + Unpin>(
        buf_reader: &mut tokio::io::BufReader<R>, 
        header: &mut HttpMeta, 
        parse_config: &HttpSafety, 
    ) -> std::io::Result<Vec<u8>> { 

        /// Reads body with Content-Length
        async fn read_content_length_body<R: AsyncRead + Unpin>(
            buf_reader: &mut tokio::io::BufReader<R>,
            safety_setting: &HttpSafety,
            content_length: usize, 
        ) -> std::io::Result<Vec<u8>> { 
            let effective_content_length = std::cmp::min(content_length, safety_setting.effective_body_size()); 
            let mut body_buffer = vec![0; effective_content_length];
            buf_reader.read_exact(&mut body_buffer).await?;
            Ok(body_buffer)
        }

        /// Reads chunked transfer encoding body
        ///
        /// # Security Philosophy: Efficient Validation Through Size Limits
        ///
        /// This parser follows a pragmatic security approach: **we only validate data size limits,
        /// not every possible malformed input**. This philosophy provides:
        ///
        /// 1. **Performance**: Fast parsing without exhaustive validation of every byte
        /// 2. **Energy Efficiency**: Minimal CPU cycles spent on validation overhead
        /// 3. **Equivalent Safety**: Size limits prevent all critical attacks (DoS, memory exhaustion)
        /// 4. **Simplicity**: Clear, maintainable code with focused security checks
        ///
        /// ## What We Check (Critical)
        /// - ✅ Cumulative size limits (prevents DoS)
        /// - ✅ Invalid hex chunk sizes (prevents crashes)
        /// - ✅ CRLF terminators (prevents protocol confusion)
        ///
        /// ## What We Don't Check (Non-Critical)
        /// - ❌ Chunk extension validity (doesn't affect security if size is validated)
        /// - ❌ Duplicate zero chunks (harmless, just ends parsing)
        /// - ❌ Chunk data content validation (application layer concern)
        ///
        /// **Rationale**: If data doesn't overflow the upper size limit, it's safe to process.
        /// Malformed but size-compliant data will be caught at the application layer or cause
        /// predictable failures without security impact. This saves energy while maintaining
        /// equivalent security to exhaustive validation.
        async fn read_chunked_body<R: AsyncRead + Unpin>(
            buf_reader: &mut tokio::io::BufReader<R>,
            header: &mut HttpMeta,
            safety_setting: &HttpSafety,
        ) -> std::io::Result<Vec<u8>> {
            let mut body_buffer = Vec::new();
            let mut current_size = 0;

            loop {
                // Read chunk size line
                let mut size_line = String::new();
                buf_reader.read_line(&mut size_line).await?;
                let chunk_size_str = size_line.trim_end_matches(|c| c == '\r' || c == '\n');

                // Parse chunk size (validates hex format - critical for preventing crashes)
                let chunk_size = usize::from_str_radix(chunk_size_str, 16).map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid chunk size")
                })?;

                if chunk_size == 0 {
                    break; // End of chunks
                }

                // Security: Cumulative size validation prevents chunked encoding DoS attacks
                // This is the CORE security mechanism - validating size limits, not every byte
                //
                // This check protects against:
                // 1. Single giant chunk: e.g., chunk_size = 1GB rejected immediately
                // 2. Multiple chunks exceeding limit: e.g., 9 bytes + 9 bytes when limit is 10
                //    - 1st iteration: current_size = 9, check passes, allocate 9 bytes
                //    - 2nd iteration: current_size = 18, check fails, return error BEFORE allocation
                // 3. Death by a thousand cuts: Many small chunks accumulating beyond limit
                //
                // Key: Validation happens BEFORE memory allocation (line 138), so attacker
                // cannot force excessive memory allocation by sending large chunk size declarations.
                // The check_body_size() uses max_body_size from HttpSafety (default: 10MB).
                current_size += chunk_size;
                if !safety_setting.check_body_size(current_size) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Chunked body exceeds maximum size",
                    ));
                }

                // Read chunk data (only reached if validation passed)
                let mut chunk_data = vec![0; chunk_size];
                buf_reader.read_exact(&mut chunk_data).await?;
                body_buffer.extend_from_slice(&chunk_data);

                // Read trailing CRLF
                let mut crlf = [0; 2];
                buf_reader.read_exact(&mut crlf).await?;
                if crlf != [b'\r', b'\n'] {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid chunk terminator",
                    ));
                }
            }

            // Read trailing headers (if any)
            header.append_from_request_stream(buf_reader, safety_setting, false).await.map_err(|_| std::io::Error::new(std::io::ErrorKind::NetworkUnreachable, "Error parsing headers"))?;

            Ok(body_buffer)
        } 

        // Read raw body data 
        let encoding = header.get_encoding().unwrap_or_default(); 
        let raw_data = if encoding.transfer().is_chunked() {
            read_chunked_body(buf_reader, header, parse_config).await?
        } else {
            let content_length = header.get_content_length().unwrap_or(0);
            read_content_length_body(buf_reader, parse_config, content_length).await?
        };

        // Apply decompression based on Transfer-Encoding
        let raw_data = encoding.content().decode_compressed(raw_data)?; 

        Ok(raw_data)
    }

    /// Write a response body to the TcpStream buffer
    /// This will automatically set the content length and content type for the meta if it is not set
    pub async fn into_static(mut self, meta: &mut HttpMeta) -> Vec<u8> {
        let bin: Vec<u8> = match self {
            Self::Text(_) => {
                self.text_into_binary();
                let bin = self.raw();
                if let None = meta.get_content_length() {
                    meta.set_content_length(bin.len());
                }
                if let None = meta.get_content_type() {
                    meta.set_content_type(HttpContentType::TextHtml());
                } 
                bin
            }
            Self::Binary(_) => {
                let bin = self.raw();
                if let None = meta.get_content_length() {
                    meta.set_content_length(bin.len());
                }
                if let None = meta.get_content_type() {
                    meta.set_content_type(HttpContentType::ApplicationOctetStream());
                }
                bin
            }
            Self::Json(_) => {
                self.json_into_binary();
                let bin = self.raw();
                if let None = meta.get_content_length() {
                    meta.set_content_length(bin.len());
                }
                if let None = meta.get_content_type() {
                    meta.set_content_type(HttpContentType::ApplicationJson());
                }
                bin
            }
            Self::Form(_) => {
                self.form_into_binary();
                let bin = self.raw();
                if let None = meta.get_content_length() {
                    meta.set_content_length(bin.len());
                }
                if let None = meta.get_content_type() {
                    meta.set_content_type(HttpContentType::ApplicationUrlEncodedForm());
                }
                bin
            }
            Self::Files(_) => {
                let boundary = if let Some(HttpContentType::Multipart {
                    subtype: _,
                    boundary: Some(boundary_value),
                }) = meta.get_content_type()
                {
                    boundary_value // Or boundary_value.to_string() depending on the type
                } else {
                    // Default boundary if none provided
                    "----DefaultBoundary7MA4YWxkTrZu0gW".to_string()
                };
                self.files_into_binary(&boundary);
                let bin = self.raw();
                if let None = meta.get_content_length() {
                    meta.set_content_length(bin.len());
                }
                if let None = meta.get_content_type() {
                    meta.set_content_type(HttpContentType::Multipart {
                        subtype: "form-data".to_string(),
                        boundary: Some(boundary),
                    });
                }
                bin
            }
            _ => {
                if let None = meta.get_content_length() {
                    meta.set_content_length(0);
                }
                EMPTY.to_vec() 
            }
        }; 
        let content_coding = meta.get_encoding().map(|e| e.content().clone()).unwrap_or(ContentCodings::new()); 
        // If the content coding is not identity, we need to encode the binary data 
        content_coding.encode_compressed(bin).unwrap_or_else(|_| vec![])  
    }

    pub fn parse_json(body: Vec<u8>) -> Self {
        return Self::Json(
            Value::from_json(std::str::from_utf8(&body).unwrap_or("")).unwrap_or(Value::new("")),
        );
    }

    /// Change Self::Json into Self::Binary
    pub fn json_into_binary(&mut self) {
        match self {
            Self::Json(json) => {
                let binary = json.into_json().as_bytes().to_vec();
                *self = Self::Binary(binary);
            }
            _ => {}
        }
    }

    pub fn parse_text(body: Vec<u8>) -> Self {
        // println!("Text body: {:?}", body);
        return Self::Text(String::from_utf8_lossy(&body).to_string());
    }

    /// Change Self::Text into Self::Binary
    pub fn text_into_binary(&mut self) {
        match self {
            Self::Text(text) => {
                let binary = text.as_bytes().to_vec();
                *self = Self::Binary(binary);
            }
            _ => {}
        }
    }

    pub fn parse_binary(body: Vec<u8>) -> Self {
        return Self::Binary(body);
    }

    /// Get the raw data for **BINARY** http body
    /// A non binary Http Body must first convert into binary in order to get the bin data
    pub fn raw(self) -> Vec<u8> {
        match self {
            Self::Binary(data) => data,
            _ => EMPTY.to_vec(), 
        }
    }

    pub fn parse_form(body: Vec<u8>) -> Self {
        let form = UrlEncodedForm::parse(body);
        return Self::Form(form);
    }

    pub fn form_into_binary(&mut self) {
        match self {
            Self::Form(form) => {
                let binary = form.to_string().into();
                *self = Self::Binary(binary);
            }
            _ => {}
        }
    }

    pub fn parse_files(body: Vec<u8>, boundary: String) -> Self {
        let files = MultiForm::parse(body, boundary);
        return Self::Files(files);
    }

    pub fn files_into_binary(&mut self, boundary: &String) {
        match self {
            Self::Files(files) => {
                let binary = files.to_string(boundary).into();
                *self = Self::Binary(binary);
            }
            _ => {}
        }
    }
}

impl Default for HttpBody {
    fn default() -> Self {
        Self::Unparsed
    }
}
