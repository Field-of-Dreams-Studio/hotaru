//! Chapter 4: Protocol Abstraction - The Real Power of Hotaru
//! 
//! This example demonstrates running multiple protocols (TCP and HTTP)
//! on the same port with shared state between them.

use hotaru::prelude::*;
use hotaru::http::*;
use tutorial_examples::{TcpChat, ChatRoom};

// Shared chat room for both protocols
static CHAT_ROOM: Lazy<ChatRoom> = Lazy::new(|| ChatRoom::new());

// Multi-protocol application
pub static APP: SApp = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:3003")
        .handle(
            HandlerBuilder::new()
                // Add TCP chat protocol
                .protocol(ProtocolBuilder::new(TcpChat::with_room(
                    ProtocolRole::Server,
                    CHAT_ROOM.clone()
                )))
                // Add HTTP protocol
                .protocol(ProtocolBuilder::new(HTTP::server(HttpSafety::default())))
        )
        .build()
});

// HTTP endpoints for web interface

endpoint! {
    APP.url("/"),
    
    /// Web chat interface
    pub web_home <HTTP> {
        html_response(r#"<!DOCTYPE html>
<html>
<head>
    <title>Multi-Protocol Chat</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 800px;
            margin: 50px auto;
            padding: 20px;
        }
        .chat-box {
            border: 1px solid #ccc;
            padding: 20px;
            border-radius: 5px;
            background: #f9f9f9;
        }
        h1 { color: #333; }
        .protocol-badge {
            display: inline-block;
            padding: 5px 10px;
            border-radius: 3px;
            font-size: 12px;
            font-weight: bold;
        }
        .http { background: #4CAF50; color: white; }
        .tcp { background: #2196F3; color: white; }
        .instructions {
            background: #fffbcc;
            padding: 15px;
            border-radius: 5px;
            margin: 20px 0;
        }
        code {
            background: #f4f4f4;
            padding: 2px 5px;
            border-radius: 3px;
        }
    </style>
</head>
<body>
    <div class="chat-box">
        <h1>🌐 Multi-Protocol Chat Server</h1>
        <span class="protocol-badge http">HTTP</span>
        <span class="protocol-badge tcp">TCP</span>
        
        <p>This server demonstrates Hotaru's protocol abstraction:</p>
        <ul>
            <li>Both TCP and HTTP clients connect to port <strong>3003</strong></li>
            <li>Messages are shared between all protocols</li>
            <li>Each protocol maintains its own connection handling</li>
        </ul>
        
        <div class="instructions">
            <h3>Connect with TCP Client:</h3>
            <code>nc localhost 3003</code>
            <p>Then use commands:</p>
            <ul>
                <li><code>JOIN yourname</code> - Join the chat</li>
                <li><code>MSG your message</code> - Send a message</li>
                <li><code>LIST</code> - List online users</li>
                <li><code>HISTORY</code> - View recent messages</li>
                <li><code>LEAVE</code> - Exit chat</li>
            </ul>
        </div>
        
        <p><a href="/api/messages">View Messages (JSON)</a> | 
           <a href="/api/users">View Users (JSON)</a></p>
    </div>
</body>
</html>"#)
    }
}

endpoint! {
    APP.url("/api/send"),
    
    /// Send a message via HTTP
    pub send_message <HTTP> {
        if req.method() != HttpMethod::POST {
            let mut response = HttpResponse::default();
            response = response.status(StatusCode::METHOD_NOT_ALLOWED);
            response.body = "Use POST to send messages".into();
            return response;
        }
        
        let json_data = match req.json().await {
            Some(data) => data,
            None => return json_response(object!({
                error: "Invalid JSON"
            }))
        };
        
        let user = json_data.get("user").string();
        let message = json_data.get("message").string();
        
        if user.is_empty() || message.is_empty() {
            return json_response(object!({
                error: "User and message are required"
            }));
        }
        
        CHAT_ROOM.add_message(user.to_string(), message.to_string()).await;
        
        json_response(object!({
            success: true,
            message: "Message sent"
        }))
    }
}

endpoint! {
    APP.url("/api/messages"),
    
    /// Get recent messages
    pub get_messages <HTTP> {
        let messages = CHAT_ROOM.get_recent_messages(20).await;
        
        let msg_array = messages.iter().map(|msg| {
            object!({
                user: msg.user.clone(),
                content: msg.content.clone(),
                timestamp: msg.timestamp
            })
        }).collect::<Vec<_>>();
        
        json_response(object!({
            messages: msg_array,
            count: msg_array.len()
        }))
    }
}

endpoint! {
    APP.url("/api/users"),
    
    /// Get online users
    pub get_users <HTTP> {
        let users = CHAT_ROOM.get_users().await;
        
        json_response(object!({
            users: users,
            count: users.len()
        }))
    }
}

endpoint! {
    APP.url("/api/join"),
    
    /// Join chat via HTTP
    pub join_chat <HTTP> {
        if req.method() != HttpMethod::POST {
            let mut response = HttpResponse::default();
            response = response.status(StatusCode::METHOD_NOT_ALLOWED);
            response.body = "Use POST to join".into();
            return response;
        }
        
        let json_data = match req.json().await {
            Some(data) => data,
            None => return json_response(object!({
                error: "Invalid JSON"
            }))
        };
        
        let username = json_data.get("username").string();
        if username.is_empty() {
            return json_response(object!({
                error: "Username is required"
            }));
        }
        
        let user_id = format!("http_{}", std::process::id());
        CHAT_ROOM.add_user(user_id, username.to_string()).await;
        CHAT_ROOM.add_message("System".to_string(), 
            format!("{} joined via HTTP", username)).await;
        
        json_response(object!({
            success: true,
            message: format!("Welcome, {}!", username)
        }))
    }
}

#[tokio::main]
async fn main() {
    println!("\n🚀 Chapter 4: Multi-Protocol Server");
    println!("====================================");
    println!("Server running on port 3003");
    println!("Supporting both TCP and HTTP protocols!\n");
    
    println!("📱 Web Interface:");
    println!("  Open http://localhost:3003 in your browser\n");
    
    println!("💻 TCP Chat:");
    println!("  Connect: nc localhost 3003");
    println!("  Commands:");
    println!("    JOIN <name>    - Join the chat");
    println!("    MSG <message>  - Send a message");
    println!("    LIST          - List online users");
    println!("    HISTORY       - View recent messages");
    println!("    LEAVE         - Exit chat\n");
    
    println!("🔗 HTTP API:");
    println!("  POST /api/join     - Join chat");
    println!("  POST /api/send     - Send message");
    println!("  GET  /api/messages - Get recent messages");
    println!("  GET  /api/users    - Get online users\n");
    
    println!("Example HTTP commands:");
    println!(r#"  curl -X POST http://localhost:3003/api/join \"#);
    println!(r#"       -H "Content-Type: application/json" \"#);
    println!(r#"       -d '{"username":"WebUser"}'"#);
    println!();
    println!(r#"  curl -X POST http://localhost:3003/api/send \"#);
    println!(r#"       -H "Content-Type: application/json" \"#);
    println!(r#"       -d '{"user":"WebUser","message":"Hello from HTTP!"}'"#);
    println!();
    
    println!("✨ Both protocols share the same chat room!");
    println!("Messages from TCP clients appear to HTTP clients and vice versa.\n");
    
    APP.clone().run().await;
}