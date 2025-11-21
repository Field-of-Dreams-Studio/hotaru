use hotaru::prelude::*;
use hotaru::http::{HttpSafety, HttpMethod};
use h2per::prelude::*;
use h2per::websocket::{
    WebSocketProtocol, 
    is_websocket_upgrade, 
    is_http2_websocket_upgrade,
    build_websocket_response,
    build_http2_websocket_response,
};
use serde_json::json;
// Response, StatusCode, Empty, and Bytes are already imported via h2per::prelude::*

// Create the app with Hyper protocol registration
pub static APP: SApp = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:3090")
        .handle(
            HandlerBuilder::new()
                .protocol(ProtocolBuilder::new(HYPER1::new(ProtocolRole::Server)))
                .protocol(ProtocolBuilder::new(HYPER2::new(ProtocolRole::Server)))
                .protocol(ProtocolBuilder::new(WebSocketProtocol::new(ProtocolRole::Server)))
        )
        .build()
});

#[tokio::main]
async fn main() {
    println!("Starting Hyper-powered Hotaru server on 127.0.0.1:3090");
    println!("Multi-protocol mode enabled");
    println!("Available protocols: HYPER1, HYPER2");
    APP.clone().run().await;
}

// ============================================================================
// HTTP/1.1 Endpoints using Hyper
// ============================================================================

endpoint! {
    APP.url("/"),
    
    /// Root endpoint using Hyper HTTP/1.1
    pub index <HYPER1> {
        text_response("Welcome to Hyper-powered Hotaru!")
    }
}

endpoint! {
    APP.url("/api/hello"),
    
    /// JSON API endpoint using Hyper
    pub hello_api <HYPER1> {
        let response_data = json!({
            "message": "Hello from Hyper HTTP/1.1",
            "version": "1.1",
            "powered_by": "hyper"
        });
        
        json_response(response_data)
    }
}

endpoint! {
    APP.url("/echo/<text>"),
    
    /// Echo endpoint with path parameters
    pub echo <HYPER1> {
        let text = req.pattern("text").unwrap_or("nothing".to_string());
        text_response(format!("Echo: {}", text))
    }
}

// ============================================================================
// HTTP/2 Endpoints using Hyper with Associated HYPER1 Upgrade Support
// ============================================================================

// HYPER1 endpoint - handles upgrade negotiation for /h2/test
endpoint! {
    APP.url("/h2/test"),
    
    /// HTTP/1.1 endpoint that can upgrade to HTTP/2 or WebSocket
    pub http2_test_upgrade <HYPER1> {
        let headers = req.request().inner.headers();
        let upgrade = headers.get("upgrade").and_then(|v| v.to_str().ok());
        let connection = headers.get("connection").and_then(|v| v.to_str().ok());
        
        // Check what the client is requesting
        match upgrade {
            Some("websocket") => {
                // WebSocket upgrade requested
                req.switch_to_ws();
                match build_websocket_response(&req.request().inner) {
                    Ok(response) => HyperResponse { inner: response },
                    Err(e) => normal_response(StatusCode::BAD_REQUEST, format!("WebSocket upgrade failed: {}", e))
                }
            }
            Some("h2c") => {
                // HTTP/2 upgrade explicitly requested
                req.switch_to_h2();
                let response = Response::builder()
                    .status(StatusCode::SWITCHING_PROTOCOLS)
                    .header("Connection", "Upgrade")
                    .header("Upgrade", "h2c")
                    .body(Empty::<Bytes>::new().boxed())
                    .unwrap();
                HyperResponse { inner: response }
            }
            _ => {
                // No explicit upgrade requested
                // Check if client supports upgrades via Connection header
                if let Some(conn) = connection {
                    if conn.to_lowercase().contains("upgrade") {
                        // Client supports upgrades, offer h2c upgrade
                        req.switch_to_h2();
                        let response = Response::builder()
                            .status(StatusCode::SWITCHING_PROTOCOLS)
                            .header("Connection", "Upgrade")
                            .header("Upgrade", "h2c")
                            .body(Empty::<Bytes>::new().boxed())
                            .unwrap();
                        HyperResponse { inner: response }
                    } else {
                        // Regular HTTP/1.1 response without upgrade headers
                        let response = Response::builder()
                            .status(StatusCode::OK)
                            .header("Content-Type", "application/json")
                            .body(Full::new(Bytes::from(json!({
                                "message": "HTTP/1.1 endpoint ready for upgrades",
                                "hint": "Add 'Connection: Upgrade' and 'Upgrade: h2c' headers to upgrade to HTTP/2",
                                "supported": ["websocket", "h2c"]
                            }).to_string())).boxed())
                            .unwrap();
                        HyperResponse { inner: response }
                    }
                } else {
                    // No Connection header, return normal response with upgrade advertisement
                    let response = Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", "application/json")
                        .header("Alt-Svc", "h2c=\":3090\"")  // Advertise h2c support
                        .body(Full::new(Bytes::from(json!({
                            "message": "HTTP/1.1 endpoint - upgrades available",
                            "current_protocol": "HTTP/1.1",
                            "available_upgrades": {
                                "http2": "Send 'Upgrade: h2c' header",
                                "websocket": "Send 'Upgrade: websocket' header"
                            },
                            "note": "Your browser doesn't support h2c upgrades via this method"
                        }).to_string())).boxed())
                        .unwrap();
                    HyperResponse { inner: response }
                }
            }
        }
    }
}

// HYPER2 endpoint - handles HTTP/2 requests for /h2/test
endpoint! {
    APP.url("/h2/test"),
    
    /// HTTP/2 endpoint - handles requests after upgrade or direct HTTP/2
    /// This is the associated endpoint for the HYPER1 handler
    pub http2_test <HYPER2> {
        let response_data = json!({
            "message": "This is HTTP/2 powered by Hyper",
            "protocol": "h2",
            "features": ["multiplexing", "server_push", "header_compression"],
            "note": "Associated with HYPER1 endpoint for upgrade handling"
        });
        
        json_response(response_data)
    }
}

endpoint! {
    APP.url("/h2/stream"),
    
    /// HTTP/2 streaming example
    pub http2_stream <HYPER2> {
        // In HTTP/2, we can access stream information
        let stream_info = if let Some(ctx) = req.as_any().downcast_ref::<HyperContext>() {
            format!("Stream ID: {:?}", ctx.stream_id)
        } else {
            "Stream info not available".to_string()
        };
        
        let response_data = json!({
            "message": "HTTP/2 Stream Example",
            "stream_info": stream_info
        });
        
        json_response(response_data)
    }
}

// ============================================================================
// JSON POST example
// ============================================================================

endpoint! {
    APP.url("/api/json"),
    config = [HttpSafety::new().with_allowed_methods(vec![HttpMethod::POST])],
    
    /// JSON POST handler using serde_json
    pub json_handler <HYPER1> {
        // Parse JSON from request body
        match req.json::<serde_json::Value>().await {
            Some(json_data) => {
                let response_data = json!({
                    "message": "JSON received",
                    "received_data": json_data,
                    "parsed_with": "serde_json"
                });
                json_response(response_data)
            }
            None => {
                text_response("Invalid or missing JSON data")
            }
        }
    }
}

// ============================================================================
// Form and POST examples
// ============================================================================

endpoint! {
    APP.url("/form"),
    config = [HttpSafety::new().with_allowed_methods(vec![HttpMethod::GET, HttpMethod::POST])],
    
    /// Form handling with Hyper
    pub form_handler <HYPER1> {
        if req.method() == &Method::GET {
            html_response(r#"
                <!DOCTYPE html>
                <html>
                <head>
                    <title>Hyper Form Example</title>
                    <style>
                        body { font-family: Arial, sans-serif; padding: 20px; }
                        form { max-width: 400px; }
                        input, button { display: block; margin: 10px 0; padding: 5px; width: 100%; }
                    </style>
                </head>
                <body>
                    <h1>Hyper Form Example</h1>
                    <form method="POST" action="/form">
                        <input type="text" name="name" placeholder="Your name" required>
                        <input type="email" name="email" placeholder="Your email" required>
                        <button type="submit">Submit</button>
                    </form>
                </body>
                </html>
            "#)
        } else {
            // Handle POST
            match req.form().await {
                Some(form) => {
                    let default_name = "Anonymous".to_string();
                    let default_email = "no-email".to_string();
                    let name = form.data.get("name").unwrap_or(&default_name);
                    let email = form.data.get("email").unwrap_or(&default_email);
                    
                    let response_data = json!({
                        "message": "Form received",
                        "name": name,
                        "email": email
                    });
                    
                    json_response(response_data)
                }
                None => {
                    text_response("No form data received")
                }
            }
        }
    }
}

// ============================================================================
// Middleware example with Hyper
// ============================================================================

middleware! {
    pub HyperLogger <HYPER1> {
        println!("[HYPER1] Request: {} {}", req.method(), req.path());
        let start = std::time::Instant::now();
        
        let result = next(req).await;
        
        println!("[HYPER1] Response time: {:?}", start.elapsed());
        result
    }
}

endpoint! {
    APP.url("/logged"),
    middleware = [HyperLogger],
    
    /// Endpoint with Hyper logging middleware
    pub logged_endpoint <HYPER1> {
        text_response("This request was logged by Hyper middleware")
    }
}

// ============================================================================
// Version detection endpoint
// ============================================================================

endpoint! {
    APP.url("/version"),
    
    /// Shows which HTTP version is being used
    pub version_info <HYPER1> {
        // This endpoint can be accessed via both HTTP/1.1 and HTTP/2
        // The actual version used depends on client negotiation
        
        let version = if let Some(ctx) = req.as_any().downcast_ref::<HyperContext>() {
            format!("{:?}", ctx.version)
        } else {
            "Unknown".to_string()
        };
        
        let response_data = json!({
            "message": "Version detection",
            "http_version": version,
            "handler": "HYPER1"
        });
        
        json_response(response_data)
    }
}

// ============================================================================
// Status endpoint
// ============================================================================

endpoint! {
    APP.url("/status"),
    
    /// Server status with protocol info
    pub status <HYPER1> {
        let response_data = json!({
            "status": "running",
            "server": "Hotaru with Hyper",
            "multi_protocol": true,
            "protocols": {
                "HYPER1": "HTTP/1.1 via Hyper",
                "HYPER2": "HTTP/2 via Hyper",
                "WebSocketProtocol": "WebSocket via tokio-tungstenite"
            },
            "endpoints": [
                "/",
                "/api/hello",
                "/echo/<text>",
                "/h2/test",
                "/h2/stream",
                "/form",
                "/logged",
                "/version",
                "/status",
                "/ws (HTTP/1.1 WebSocket)",
                "/ws2 (HTTP/2 Extended CONNECT WebSocket)"
            ]
        });
        
        json_response(response_data)
    }
}

// ============================================================================
// WebSocket File Download Endpoint
// ============================================================================

// HTTP/1.1 endpoint - handles the upgrade handshake for file download
endpoint! {
    APP.url("/download"),
    
    /// WebSocket file download endpoint - upgrade handler
    pub download_upgrade <HYPER1> {
        if is_websocket_upgrade(&req.request().inner) {
            // Build and return 101 Switching Protocols response
            match build_websocket_response(&req.request().inner) {
                Ok(response) => {
                    println!("📥 WebSocket download endpoint upgrade initiated");
                    
                    // Signal protocol switch to WebSocket
                    req.switch_to_ws();
                    
                    HyperResponse { inner: response }
                }
                Err(e) => {
                    normal_response(StatusCode::BAD_REQUEST, format!("WebSocket upgrade failed: {}", e))
                }
            }
        } else {
            // Serve a simple HTML page for testing file downloads
            html_response(r#"
                <!DOCTYPE html>
                <html>
                <head>
                    <title>WebSocket File Download</title>
                    <style>
                        body { font-family: Arial, sans-serif; margin: 40px; }
                        .container { max-width: 800px; margin: 0 auto; }
                        button { padding: 10px 20px; margin: 5px; cursor: pointer; }
                        button:hover { background-color: #e0e0e0; }
                        #status { margin: 20px 0; padding: 10px; background: #f5f5f5; border-radius: 5px; }
                        #fileList { margin: 20px 0; }
                        .file-item { padding: 8px; margin: 5px 0; background: #f9f9f9; border-radius: 3px; cursor: pointer; }
                        .file-item:hover { background: #e9e9e9; }
                        #downloadProgress { margin: 20px 0; display: none; }
                        .progress-bar { width: 100%; height: 20px; background: #e0e0e0; border-radius: 10px; overflow: hidden; }
                        .progress-fill { height: 100%; background: #4CAF50; width: 0%; transition: width 0.3s; }
                    </style>
                </head>
                <body>
                    <div class="container">
                        <h1>WebSocket File Download</h1>
                        <p>Download files from the server's programfiles directory via WebSocket</p>
                        
                        <div id="status">Status: Not connected</div>
                        
                        <button onclick="connect()">Connect</button>
                        <button onclick="disconnect()">Disconnect</button>
                        <button onclick="listFiles()">List Files</button>
                        
                        <div id="fileList"></div>
                        
                        <div id="downloadProgress">
                            <h3>Download Progress</h3>
                            <div class="progress-bar">
                                <div class="progress-fill" id="progressFill"></div>
                            </div>
                            <div id="progressText">0%</div>
                        </div>
                        
                        <script>
                            let ws = null;
                            let downloadBuffer = [];
                            let downloadInfo = null;
                            
                            function connect() {
                                const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
                                ws = new WebSocket(protocol + '//' + window.location.host + '/download');
                                
                                ws.onopen = () => {
                                    document.getElementById('status').innerHTML = 'Status: Connected';
                                    console.log('WebSocket connected');
                                };
                                
                                ws.onmessage = (event) => {
                                    if (typeof event.data === 'string') {
                                        // Text message - could be file list or metadata
                                        const msg = JSON.parse(event.data);
                                        
                                        if (msg.type === 'file_list') {
                                            displayFileList(msg.files);
                                        } else if (msg.type === 'file_start') {
                                            startDownload(msg);
                                        } else if (msg.type === 'file_complete') {
                                            completeDownload();
                                        } else if (msg.type === 'error') {
                                            alert('Error: ' + msg.message);
                                        }
                                    } else {
                                        // Binary data - file chunk
                                        downloadBuffer.push(event.data);
                                        updateProgress();
                                    }
                                };
                                
                                ws.onerror = (error) => {
                                    console.error('WebSocket error:', error);
                                    document.getElementById('status').innerHTML = 'Status: Error';
                                };
                                
                                ws.onclose = () => {
                                    document.getElementById('status').innerHTML = 'Status: Disconnected';
                                    console.log('WebSocket disconnected');
                                };
                            }
                            
                            function disconnect() {
                                if (ws) {
                                    ws.close();
                                    ws = null;
                                }
                            }
                            
                            function listFiles() {
                                if (ws && ws.readyState === WebSocket.OPEN) {
                                    ws.send(JSON.stringify({ command: 'list' }));
                                } else {
                                    alert('Not connected to WebSocket');
                                }
                            }
                            
                            function displayFileList(files) {
                                const listDiv = document.getElementById('fileList');
                                listDiv.innerHTML = '<h3>Available Files:</h3>';
                                
                                files.forEach(file => {
                                    const fileDiv = document.createElement('div');
                                    fileDiv.className = 'file-item';
                                    fileDiv.innerHTML = `📄 ${file.name} (${formatBytes(file.size)})`;
                                    fileDiv.onclick = () => downloadFile(file.name);
                                    listDiv.appendChild(fileDiv);
                                });
                            }
                            
                            function downloadFile(filename) {
                                if (ws && ws.readyState === WebSocket.OPEN) {
                                    downloadBuffer = [];
                                    ws.send(JSON.stringify({ command: 'download', filename: filename }));
                                } else {
                                    alert('Not connected to WebSocket');
                                }
                            }
                            
                            function startDownload(info) {
                                downloadInfo = info;
                                document.getElementById('downloadProgress').style.display = 'block';
                                updateProgress();
                            }
                            
                            function updateProgress() {
                                if (!downloadInfo) return;
                                
                                const received = downloadBuffer.reduce((total, chunk) => total + chunk.size, 0);
                                const progress = Math.min(100, (received / downloadInfo.size) * 100);
                                
                                document.getElementById('progressFill').style.width = progress + '%';
                                document.getElementById('progressText').innerHTML = 
                                    `${progress.toFixed(1)}% - ${formatBytes(received)} / ${formatBytes(downloadInfo.size)}`;
                            }
                            
                            function completeDownload() {
                                if (!downloadInfo || downloadBuffer.length === 0) return;
                                
                                // Combine all chunks into a single blob
                                const blob = new Blob(downloadBuffer);
                                
                                // Create download link
                                const url = URL.createObjectURL(blob);
                                const a = document.createElement('a');
                                a.href = url;
                                a.download = downloadInfo.filename;
                                a.click();
                                
                                // Cleanup
                                URL.revokeObjectURL(url);
                                downloadBuffer = [];
                                downloadInfo = null;
                                document.getElementById('downloadProgress').style.display = 'none';
                            }
                            
                            function formatBytes(bytes) {
                                if (bytes === 0) return '0 Bytes';
                                const k = 1024;
                                const sizes = ['Bytes', 'KB', 'MB', 'GB'];
                                const i = Math.floor(Math.log(bytes) / Math.log(k));
                                return (bytes / Math.pow(k, i)).toFixed(2) + ' ' + sizes[i];
                            }
                        </script>
                    </div>
                </body>
                </html>
            "#.to_string())
        }
    }
}

// ============================================================================
// WebSocket Protocol Switching Examples - Clean Separation
// ============================================================================

// HTTP/1.1 endpoint - handles the upgrade handshake only
endpoint! {
    APP.url("/ws"),
    
    /// HTTP/1.1 WebSocket upgrade endpoint
    /// This handles ONLY the HTTP upgrade negotiation
    pub websocket_upgrade <HYPER1> {
        if is_websocket_upgrade(&req.request().inner) {
            // Build and return 101 Switching Protocols response
            match build_websocket_response(&req.request().inner) {
                Ok(response) => {
                    println!("✅ HTTP/1.1 → WebSocket upgrade response sent");
                    
                    // Signal protocol switch to WebSocket
                    req.switch_to_ws();
                    
                    println!("📡 Protocol switch signaled to framework");
                    println!("🔄 Connection will switch from HTTP/1.1 → WebSocket");
                    
                    // The framework will:
                    // 1. Detect ConnectionStatus::SwitchProtocol in the context
                    // 2. Use HyperTransport.into_websocket_transport() to convert the transport
                    // 3. Switch to WebSocketProtocol with the new transport
                    // 4. Continue handling the connection with the WebSocket protocol
                    
                    // Return the 101 Switching Protocols response
                    HyperResponse { inner: response }
                }
                Err(e) => {
                    println!("❌ WebSocket upgrade failed: {}", e);
                    normal_response(StatusCode::BAD_REQUEST, format!("WebSocket upgrade failed: {}", e))
                }
            }
        } else {
            // Not an upgrade request - serve WebSocket client page
            html_response(websocket_demo_page())
        }
    }
}

// WebSocket endpoint - handles ONLY WebSocket frames
// This would be registered for the same URL but different protocol
endpoint! {
    APP.url("/ws"),
    
    /// Pure WebSocket endpoint - no HTTP logic
    /// This runs AFTER successful protocol switch
    pub websocket_handler <WebSocketProtocol> {
        // This endpoint would only be called after successful upgrade
        // It handles pure WebSocket communication
        
        // In a real implementation, this would work with WebSocket messages:
        // match req.ws_message() {
        //     WsMessage::Text(text) => {
        //         req.send_ws_message(WsMessage::Text(format!("Echo: {}", text)));
        //     }
        //     WsMessage::Binary(data) => {
        //         req.send_ws_message(WsMessage::Binary(data));
        //     }
        //     WsMessage::Ping(data) => {
        //         req.send_ws_message(WsMessage::Pong(data));
        //     }
        //     _ => {}
        // }
        
        // For now, just log that we would handle WebSocket
        println!("🔌 WebSocket endpoint would handle frames here");
        text_response("WebSocket handler active")
    }
}

// ============================================================================
// HTTP/2 WebSocket Protocol Switching via Extended CONNECT
// ============================================================================

// HTTP/2 endpoint - handles Extended CONNECT WebSocket upgrade
endpoint! {
    APP.url("/ws2"),
    
    /// HTTP/2 WebSocket upgrade endpoint using Extended CONNECT
    /// This demonstrates stream-level protocol switching
    pub http2_websocket_upgrade <HYPER2> {
        if is_http2_websocket_upgrade(&req.request().inner) {
            // Build and return 200 OK response for Extended CONNECT
            match build_http2_websocket_response(&req.request().inner) {
                Ok(response) => {
                    println!("✅ HTTP/2 → WebSocket upgrade via Extended CONNECT");
                    
                    // Get the stream ID if available
                    let stream_id = req.stream_id.unwrap_or(0);
                    println!("📊 Upgrading HTTP/2 stream {}", stream_id);
                    
                    // Signal protocol switch to WebSocket for this stream
                    req.switch_to_ws();
                    
                    println!("📡 Stream-level protocol switch signaled");
                    println!("🔄 Stream {} will switch from HTTP/2 → WebSocket", stream_id);
                    
                    // The framework will:
                    // 1. Detect ConnectionStatus::SwitchProtocol in the context
                    // 2. Use Http2Transport.upgrade_stream_to_websocket(stream_id)
                    // 3. Switch this stream to WebSocketProtocol
                    // 4. Other HTTP/2 streams remain unaffected
                    
                    // Return 200 OK for Extended CONNECT
                    HyperResponse { inner: response }
                }
                Err(e) => {
                    println!("❌ HTTP/2 WebSocket upgrade failed: {}", e);
                    normal_response(StatusCode::BAD_REQUEST, format!("WebSocket upgrade failed: {}", e))
                }
            }
        } else {
            // Return information about HTTP/2 WebSocket
            json_response(json!({
                "endpoint": "/ws2",
                "protocol": "HTTP/2",
                "method": "Extended CONNECT",
                "description": "Use Extended CONNECT with :protocol pseudo-header for WebSocket over HTTP/2",
                "example": {
                    ":method": "CONNECT",
                    ":protocol": "websocket",
                    ":path": "/ws2",
                    ":scheme": "https"
                }
            }))
        }
    }
}

// WebSocket endpoint for /ws2 - handles frames after HTTP/2 upgrade
endpoint! {
    APP.url("/ws2"),
    
    /// Pure WebSocket handler for HTTP/2 upgraded streams
    pub http2_websocket_handler <WebSocketProtocol> {
        println!("🔌 HTTP/2 WebSocket stream handler active");
        text_response("HTTP/2 WebSocket handler active")
    }
}

// Helper function for WebSocket demo page
fn websocket_demo_page() -> String {
    r#"
<!DOCTYPE html>
<html>
<head>
    <title>Clean WebSocket Architecture</title>
    <style>
        body { font-family: Arial; padding: 20px; background: #f5f5f5; }
        .container { max-width: 800px; margin: 0 auto; background: white; padding: 20px; border-radius: 10px; }
        .endpoint { 
            background: #f9f9f9; 
            padding: 15px; 
            margin: 15px 0;
            border-radius: 5px;
            border-left: 4px solid #4CAF50;
        }
        h1 { color: #333; }
        h3 { color: #555; margin-top: 0; }
        code { 
            background: #e8e8e8; 
            padding: 2px 6px; 
            border-radius: 3px;
            font-size: 14px;
        }
        button {
            background: #4CAF50;
            color: white;
            border: none;
            padding: 10px 20px;
            border-radius: 5px;
            cursor: pointer;
            font-size: 16px;
            margin: 10px 0;
        }
        button:hover { background: #45a049; }
        #output {
            background: #f0f0f0;
            padding: 15px;
            border-radius: 5px;
            margin-top: 20px;
            min-height: 100px;
            font-family: monospace;
        }
        .success { color: #4CAF50; }
        .error { color: #f44336; }
        .info { color: #2196F3; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🚀 Clean Protocol Separation Demo</h1>
        
        <div class="endpoint">
            <h3>HTTP/1.1 Endpoint (<code>&lt;HYPER1&gt;</code>)</h3>
            <p>✅ Handles HTTP upgrade negotiation</p>
            <p>✅ Returns 101 Switching Protocols</p>
            <p>❌ No WebSocket logic here</p>
        </div>
        
        <div class="endpoint">
            <h3>WebSocket Endpoint (<code>&lt;WebSocketProtocol&gt;</code>)</h3>
            <p>✅ Pure WebSocket frame handling</p>
            <p>✅ Echo server logic</p>
            <p>❌ No HTTP concerns</p>
        </div>
        
        <div class="endpoint">
            <h3>Transport Conversion</h3>
            <p>✅ <code>HyperTransport</code> → <code>WebSocketTransport</code></p>
            <p>✅ Preserves connection ID: <code>transport.connection_id()</code></p>
            <p>✅ Tracks upgrade source: <code>UpgradeSource::Http1</code></p>
        </div>
        
        <button onclick="testWebSocket()">Test WebSocket Connection</button>
        <button onclick="clearOutput()">Clear Output</button>
        
        <div id="output"></div>
        
        <script>
            let ws = null;
            
            function log(message, className = '') {
                const output = document.getElementById('output');
                const time = new Date().toLocaleTimeString();
                output.innerHTML += '<div class="' + className + '">[' + time + '] ' + message + '</div>';
                output.scrollTop = output.scrollHeight;
            }
            
            function testWebSocket() {
                if (ws && ws.readyState === WebSocket.OPEN) {
                    log('Already connected!', 'info');
                    return;
                }
                
                log('Initiating WebSocket upgrade...', 'info');
                ws = new WebSocket('ws://' + window.location.host + '/ws');
                
                ws.onopen = () => {
                    log('✅ WebSocket connected (protocol switched successfully!)', 'success');
                    log('Sending test message...', 'info');
                    ws.send('Hello from clean architecture!');
                };
                
                ws.onmessage = (e) => {
                    log('📥 Received: ' + e.data, 'success');
                };
                
                ws.onerror = (e) => {
                    log('❌ Error: ' + e, 'error');
                };
                
                ws.onclose = () => {
                    log('🔌 WebSocket connection closed', 'info');
                    ws = null;
                };
            }
            
            function clearOutput() {
                document.getElementById('output').innerHTML = '';
                log('Output cleared', 'info');
            }
            
            // Initial message
            log('Ready to test clean protocol separation', 'info');
        </script>
    </div>
</body>
</html>
    "#.to_string()
}