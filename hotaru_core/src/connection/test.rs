#[tokio::test]
async fn test_https_connection() {
    use super::{ConnectionBuilder, Result, TcpConnectionStream};
    use crate::http::traits::HTTP;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[allow(dead_code)]
    const TEST_HTTP_SERVER: &str = "fds.rs";
    const TEST_HTTPS_SERVER: &str = "fds.rs";

    async fn send_http_request(conn: &mut TcpConnectionStream) -> Result<String> {
        let request = "GET / HTTP/1.1\r\nHost: fds.rs\r\nConnection: close\r\n\r\n";
        conn.write_all(request.as_bytes()).await?;

        let mut response = Vec::new();
        conn.read_to_end(&mut response).await?;
        Ok(String::from_utf8_lossy(&response).into())
    }

    let builder = ConnectionBuilder::<HTTP>::new(TEST_HTTPS_SERVER)
        .port(443)
        .tls(true);

    let mut conn = builder.connect().await.unwrap();
    let response = send_http_request(&mut conn).await.unwrap();

    // assert!(response.contains("HTTP/1.1 200 OK"));
    // assert!(response.contains("\"url\": \"https://httpbin.org/get\""));

    println!("Response: {}", response); // Debugging output
}

/// TCP Echo Protocol test - demonstrates custom Protocol implementation
#[cfg(test)]
mod tcp_echo_tests {
    use crate::connection::{Protocol, Transport, Stream, Message, RequestContext, ProtocolRole, TcpConnectionStream, TcpReader, TcpWriter};
    use crate::app::application::App;
    use std::error::Error;
    use async_trait::async_trait;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use bytes::BytesMut;
    use std::sync::Arc;

    // ============================================================================
    // Simple TCP Protocol Implementation (from tcp_echo_example)
    // ============================================================================

    /// A simple TCP echo protocol for testing
    #[derive(Clone)]
    pub struct TcpProtocol {
        role: ProtocolRole,
    }

    impl TcpProtocol {
        pub fn new(role: ProtocolRole) -> Self {
            Self { role }
        }
    }

    /// Simple TCP transport (no special transport layer)
    #[derive(Clone)]
    pub struct TcpTransport;

    impl Transport for TcpTransport {
        fn id(&self) -> i128 {
            0
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    /// TCP doesn't have streams like HTTP/2
    #[derive(Clone)]
    pub struct TcpStream;

    impl Stream for TcpStream {
        fn id(&self) -> u32 {
            0
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    /// Simple TCP message (just raw bytes)
    #[derive(Clone, Debug, PartialEq)]
    pub enum TcpMessage {
        GoodBye,
        Hello,
        Get(String),
        Data(String)
    }

    impl TcpMessage {
        pub fn new() -> Self {
            TcpMessage::Data(String::new())
        }

        pub fn from_binary(data: Vec<u8>) -> Self {
            // Convert to string and trim whitespace for cleaner parsing
            let data_str = String::from_utf8_lossy(&data);
            let trimmed = data_str.trim();

            if trimmed.starts_with("GOODBYE") {
                TcpMessage::GoodBye
            } else if trimmed.starts_with("HELLO") {
                TcpMessage::Hello
            } else if trimmed.starts_with("GET ") {
                let key = trimmed[4..].to_string();
                TcpMessage::Get(key)
            } else if trimmed.starts_with("DATA ") {
                let content = trimmed[5..].to_string();
                TcpMessage::Data(content)
            } else {
                TcpMessage::Data(trimmed.to_string())
            }
        }

        pub fn into_binary(&self) -> Vec<u8> {
            match self {
                TcpMessage::GoodBye => b"GOODBYE\n".to_vec(),
                TcpMessage::Hello => b"HELLO\n".to_vec(),
                TcpMessage::Get(key) => {
                    let mut buf = b"GET ".to_vec();
                    buf.extend_from_slice(key.as_bytes());
                    buf.extend_from_slice(b"\n");
                    buf
                }
                TcpMessage::Data(data) => {
                    let mut buf = b"DATA ".to_vec();
                    buf.extend_from_slice(data.as_bytes());
                    buf.extend_from_slice(b"\n");
                    buf
                }
            }
        }
    }

    impl Message for TcpMessage {
        fn encode(&self, buf: &mut BytesMut) -> Result<(), Box<dyn Error + Send + Sync>> {
            buf.extend_from_slice(&self.into_binary());
            Ok(())
        }

        fn decode(buf: &mut BytesMut) -> Result<Option<Self>, Box<dyn Error + Send + Sync>>
        where
            Self: Sized,
        {
            // Look for a newline to determine if we have a complete message
            if let Some(newline_pos) = buf.iter().position(|&b| b == b'\n') {
                // We have a complete message
                let data = buf.split_to(newline_pos + 1).to_vec();
                // Remove the newline for parsing
                let data = if data.ends_with(b"\n") {
                    data[..data.len() - 1].to_vec()
                } else {
                    data
                };
                Ok(Some(TcpMessage::from_binary(data)))
            } else if !buf.is_empty() {
                // For simplicity, if no newline but we have data, consume it all
                let data = buf.split().to_vec();
                Ok(Some(TcpMessage::from_binary(data)))
            } else {
                // Need more data
                Ok(None)
            }
        }
    }

    /// Simple TCP context
    pub struct TcpContext {
        pub request: TcpMessage,
        pub response: TcpMessage,
        role: ProtocolRole,
    }

    impl TcpContext {
        pub fn new(role: ProtocolRole) -> Self {
            Self {
                request: TcpMessage::new(),
                response: TcpMessage::new(),
                role,
            }
        }
    }

    impl RequestContext for TcpContext {
        type Request = TcpMessage;
        type Response = TcpMessage;

        fn handle_error(&mut self) {
            self.response = TcpMessage::Data("ERROR".to_string());
        }

        fn role(&self) -> ProtocolRole {
            self.role
        }
    }

    #[async_trait]
    impl Protocol for TcpProtocol {
        type Transport = TcpTransport;
        type Stream = TcpStream;
        type Message = TcpMessage;
        type Context = TcpContext;

        fn detect(initial_bytes: &[u8]) -> bool {
            // Check if it's NOT an HTTP request first
            let data_str = String::from_utf8_lossy(initial_bytes);
            let first_line = data_str.lines().next().unwrap_or("");

            // If it contains HTTP/, it's likely an HTTP request
            if first_line.contains("HTTP/") {
                return false;
            }

            // Now check for our TCP protocol messages
            initial_bytes.starts_with(b"HELLO") ||
            initial_bytes.starts_with(b"GOODBYE") ||
            initial_bytes.starts_with(b"DATA ") ||
            (initial_bytes.starts_with(b"GET ") && !first_line.contains("HTTP/")) ||
            initial_bytes.starts_with(b"ECHO:")
        }

        fn role(&self) -> ProtocolRole {
            self.role
        }

        async fn handle(
            &mut self,
            mut reader: TcpReader,
            mut writer: TcpWriter,
            _app: Arc<App>,
        ) -> Result<(), Box<dyn Error + Send + Sync>> {
            match self.role {
                ProtocolRole::Server => {
                    let mut buffer = [0u8; 1024];

                    loop {
                        let n = match reader.read(&mut buffer).await {
                            Ok(0) => break,
                            Ok(n) => n,
                            Err(_) => break,
                        };

                        let msg = TcpMessage::from_binary(buffer[..n].to_vec());

                        let response = match msg {
                            TcpMessage::GoodBye => {
                                let goodbye = TcpMessage::GoodBye;
                                let _ = writer.write_all(&goodbye.into_binary()).await;
                                let _ = writer.flush().await;
                                break;
                            }
                            TcpMessage::Hello => {
                                TcpMessage::Data("Hello! Welcome to TCP server".to_string())
                            }
                            TcpMessage::Get(key) => {
                                TcpMessage::Data(format!("Got request for: {}", key))
                            }
                            TcpMessage::Data(data) => {
                                TcpMessage::Data(format!("ECHO: {}", data))
                            }
                        };

                        match writer.write_all(&response.into_binary()).await {
                            Ok(_) => {
                                let _ = writer.flush().await;
                            }
                            Err(_) => break,
                        }
                    }
                }
                ProtocolRole::Client => {
                    // Client implementation (not used in this test)
                }
            }

            Ok(())
        }
    }

    // ============================================================================
    // Tests
    // ============================================================================

    #[tokio::test]
    async fn test_tcp_message_encoding() {
        // Test message binary conversion
        let hello = TcpMessage::Hello;
        assert_eq!(hello.into_binary(), b"HELLO\n");

        let goodbye = TcpMessage::GoodBye;
        assert_eq!(goodbye.into_binary(), b"GOODBYE\n");

        let get = TcpMessage::Get("/status".to_string());
        assert_eq!(get.into_binary(), b"GET /status\n");

        let data = TcpMessage::Data("test data".to_string());
        assert_eq!(data.into_binary(), b"DATA test data\n");
    }

    #[tokio::test]
    async fn test_tcp_message_decoding() {
        // Test message parsing from bytes
        let hello = TcpMessage::from_binary(b"HELLO\n".to_vec());
        assert_eq!(hello, TcpMessage::Hello);

        let goodbye = TcpMessage::from_binary(b"GOODBYE\n".to_vec());
        assert_eq!(goodbye, TcpMessage::GoodBye);

        let get = TcpMessage::from_binary(b"GET /status\n".to_vec());
        assert_eq!(get, TcpMessage::Get("/status".to_string()));

        let data = TcpMessage::from_binary(b"DATA test data\n".to_vec());
        assert_eq!(data, TcpMessage::Data("test data".to_string()));

        // Test with whitespace
        let hello_ws = TcpMessage::from_binary(b"  HELLO  \n".to_vec());
        assert_eq!(hello_ws, TcpMessage::Hello);
    }

    #[tokio::test]
    async fn test_tcp_protocol_detection() {
        // Should detect our TCP protocol messages
        assert!(TcpProtocol::detect(b"HELLO\n"));
        assert!(TcpProtocol::detect(b"GOODBYE\n"));
        assert!(TcpProtocol::detect(b"DATA test\n"));
        assert!(TcpProtocol::detect(b"GET /status\n"));
        assert!(TcpProtocol::detect(b"ECHO: test\n"));

        // Should NOT detect HTTP requests
        assert!(!TcpProtocol::detect(b"GET / HTTP/1.1\r\n"));
        assert!(!TcpProtocol::detect(b"POST /api HTTP/1.1\r\n"));

        // Edge case: GET without HTTP should be detected
        assert!(TcpProtocol::detect(b"GET /test"));
    }

    #[tokio::test]
    async fn test_tcp_message_codec() {
        use bytes::BytesMut;

        // Test encode
        let mut buf = BytesMut::new();
        let msg = TcpMessage::Data("test".to_string());
        msg.encode(&mut buf).unwrap();
        assert_eq!(&buf[..], b"DATA test\n");

        // Test decode - complete message
        let mut buf = BytesMut::from(&b"HELLO\n"[..]);
        let decoded = TcpMessage::decode(&mut buf).unwrap();
        assert_eq!(decoded, Some(TcpMessage::Hello));
        assert!(buf.is_empty());

        // Test decode - incomplete message (no newline)
        let mut buf = BytesMut::from(&b"HELLO"[..]);
        let decoded = TcpMessage::decode(&mut buf).unwrap();
        // Should consume all data even without newline
        assert_eq!(decoded, Some(TcpMessage::Hello));

        // Test decode - need more data
        let mut buf = BytesMut::new();
        let decoded = TcpMessage::decode(&mut buf).unwrap();
        assert_eq!(decoded, None);
    }

    #[tokio::test]
    async fn test_tcp_echo_server_client() {
        use tokio::net::{TcpListener, TcpStream};
        use std::time::Duration;

        // Start a test server
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Spawn server task
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut protocol = TcpProtocol::new(ProtocolRole::Server);
            let app = App::new().build();

            let tcp_stream = TcpConnectionStream::Tcp(stream);
            let (reader, writer) = crate::connection::split_connection(tcp_stream);

            let _ = protocol.handle(reader, writer, app).await;
        });

        // Give server time to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Connect as client
        let mut client = TcpStream::connect(addr).await.unwrap();

        // Test HELLO
        client.write_all(b"HELLO\n").await.unwrap();
        let mut buf = vec![0u8; 1024];
        let n = client.read(&mut buf).await.unwrap();
        let response = TcpMessage::from_binary(buf[..n].to_vec());
        assert_eq!(response, TcpMessage::Data("Hello! Welcome to TCP server".to_string()));

        // Test DATA echo
        client.write_all(b"DATA test message\n").await.unwrap();
        let n = client.read(&mut buf).await.unwrap();
        let response = TcpMessage::from_binary(buf[..n].to_vec());
        assert_eq!(response, TcpMessage::Data("ECHO: test message".to_string()));

        // Test GET
        client.write_all(b"GET /status\n").await.unwrap();
        let n = client.read(&mut buf).await.unwrap();
        let response = TcpMessage::from_binary(buf[..n].to_vec());
        assert_eq!(response, TcpMessage::Data("Got request for: /status".to_string()));

        // Test GOODBYE
        client.write_all(b"GOODBYE\n").await.unwrap();
        let n = client.read(&mut buf).await.unwrap();
        let response = TcpMessage::from_binary(buf[..n].to_vec());
        assert_eq!(response, TcpMessage::GoodBye);
    }
}
