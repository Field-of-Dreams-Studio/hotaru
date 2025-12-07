//! Example demonstrating WebSocket upgrade with protocol switching
//!
//! This example shows how to upgrade from HTTP/1.1 to WebSocket protocol.
//! Note: Protocol switching in Hotaru happens at the connection level.

use hotaru_core::{
    app::application::{App, AppBuilder},
    app::protocol::{HandlerBuilder, ProtocolBuilder},
    connection::{ProtocolRole, Protocol, RequestContext},
};
use hotaru_meta::*;
use h2per::{HYPER1, StatusCode};
use h2per::websocket::{WebSocketProtocol, is_websocket_upgrade, build_websocket_response};
use serde_json::json;
use once_cell::sync::Lazy;
use std::sync::Arc;

// Create the app with both HTTP and WebSocket protocols
pub static APP: Lazy<Arc<App>> = Lazy::new(|| {
    AppBuilder::new()
        .binding("127.0.0.1:3033")
        .handle(
            HandlerBuilder::new()
                // Register HTTP/1.1 protocol
                .protocol(ProtocolBuilder::new(HYPER1::new(ProtocolRole::Server)))
                // Register WebSocket protocol 
                .protocol(ProtocolBuilder::new(WebSocketProtocol::new(ProtocolRole::Server)))
        )
        .build()
});

#[tokio::main]
async fn main() {
    println!("Starting WebSocket Upgrade Example on 127.0.0.1:3033");
    println!("Test WebSocket with: wscat -c ws://127.0.0.1:3033/ws");
    println!("Or use the web interface at http://127.0.0.1:3033/");
    APP.clone().run().await;
}

// ============================================================================
// HTTP Endpoints
// ============================================================================

endpoint! {
    APP.url("/"),
    
    /// Root page with WebSocket test interface
    pub index <HYPER1> {
        let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>WebSocket Test</title>
    <style>
        body { font-family: Arial, sans-serif; padding: 20px; }
        #messages { 
            border: 1px solid #ccc; 
            height: 300px; 
            overflow-y: auto; 
            padding: 10px;
            margin: 10px 0;
            background: #f5f5f5;
        }
        input, button { padding: 5px 10px; margin: 5px; }
        .status { margin: 10px 0; font-weight: bold; }
        .connected { color: green; }
        .disconnected { color: red; }
    </style>
</head>
<body>
    <h1>WebSocket Protocol Switching Test</h1>
    
    <div class="status" id="status">Status: <span class="disconnected">Disconnected</span></div>
    
    <button onclick="connect()">Connect WebSocket</button>
    <button onclick="disconnect()">Disconnect</button>
    
    <div id="messages"></div>
    
    <input type="text" id="messageInput" placeholder="Enter message" onkeypress="if(event.key=='Enter') sendMessage()">
    <button onclick="sendMessage()">Send</button>
    <button onclick="sendPing()">Ping</button>
    <button onclick="clearMessages()">Clear</button>
    
    <script>
        let ws = null;
        
        function connect() {
            if (ws) {
                addMessage('Already connected');
                return;
            }
            
            addMessage('Connecting to WebSocket...');
            
            // Connect to WebSocket endpoint
            ws = new WebSocket('ws://127.0.0.1:3033/ws');
            
            ws.onopen = function() {
                addMessage('✅ WebSocket connected!');
                document.querySelector('#status span').className = 'connected';
                document.querySelector('#status span').textContent = 'Connected';
            };
            
            ws.onmessage = function(event) {
                addMessage('📥 Received: ' + event.data);
            };
            
            ws.onerror = function(error) {
                addMessage('❌ Error: ' + error);
            };
            
            ws.onclose = function() {
                addMessage('🔌 WebSocket disconnected');
                document.querySelector('#status span').className = 'disconnected';
                document.querySelector('#status span').textContent = 'Disconnected';
                ws = null;
            };
        }
        
        function disconnect() {
            if (ws) {
                ws.close();
            } else {
                addMessage('Not connected');
            }
        }
        
        function sendMessage() {
            const input = document.getElementById('messageInput');
            const message = input.value.trim();
            
            if (!message) return;
            
            if (ws && ws.readyState === WebSocket.OPEN) {
                ws.send(message);
                addMessage('📤 Sent: ' + message);
                input.value = '';
            } else {
                addMessage('❌ Not connected');
            }
        }
        
        function sendPing() {
            if (ws && ws.readyState === WebSocket.OPEN) {
                // Note: Browser WebSocket API doesn't expose ping/pong
                ws.send('ping');
                addMessage('📤 Sent: ping');
            } else {
                addMessage('❌ Not connected');
            }
        }
        
        function clearMessages() {
            document.getElementById('messages').innerHTML = '';
        }
        
        function addMessage(msg) {
            const messages = document.getElementById('messages');
            const msgDiv = document.createElement('div');
            const time = new Date().toLocaleTimeString();
            msgDiv.textContent = `[${time}] ${msg}`;
            messages.appendChild(msgDiv);
            messages.scrollTop = messages.scrollHeight;
        }
    </script>
</body>
</html>
        "#;
        
        req.response_mut().html(html.to_string());
    }
}

endpoint! {
    APP.url("/ws"),
    
    /// WebSocket endpoint - handles the upgrade
    pub websocket <HYPER1> {
        // Check if this is a WebSocket upgrade request
        if is_websocket_upgrade(&req.request().inner) {
            println!("🔄 WebSocket upgrade requested!");
            
            // Build the 101 Switching Protocols response
            match build_websocket_response(&req.request().inner) {
                Ok(response) => {
                    // Set our response to the switching protocols response
                    req.response_mut().inner = response;
                    
                    // Use the new switch_to_ws convenience method
                    req.switch_to_ws();
                    
                    println!("✅ Sent 101 Switching Protocols response");
                    println!("🚀 Protocol switch to WebSocket initiated");
                }
                Err(e) => {
                    println!("Failed to build WebSocket response: {}", e);
                    req.response_mut().set_status(StatusCode::BAD_REQUEST);
                    req.response_mut().text(format!("WebSocket upgrade failed: {}", e));
                }
            }
        } else {
            // Not a WebSocket request, return info
            req.response_mut().json(json!({
                "error": "Not a WebSocket upgrade request",
                "hint": "Use a WebSocket client or the web interface at /",
                "required_headers": {
                    "Upgrade": "websocket",
                    "Connection": "Upgrade",
                    "Sec-WebSocket-Key": "<base64-key>",
                    "Sec-WebSocket-Version": "13"
                }
            })).unwrap();
        }
    }
}

endpoint! {
    APP.url("/api/status"),
    
    /// API endpoint to check server status
    pub status <HYPER1> {
        req.response_mut().json(json!({
            "server": "Hotaru with Hyper",
            "protocols": ["HTTP/1.1", "WebSocket"],
            "endpoints": {
                "/": "Web interface for WebSocket testing",
                "/ws": "WebSocket upgrade endpoint",
                "/api/status": "This status endpoint"
            },
            "note": "Protocol switching happens at the Protocol level, not in endpoints"
        })).unwrap();
    }
}