//! Simple TCP Chat Protocol Implementation for Chapter 4

use std::sync::Arc;
use std::error::Error;
use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::RwLock;
use std::collections::HashMap;

use hotaru_core::connection::{
    Protocol, Transport, Stream, Message,
    RequestContext, ProtocolRole,
    TcpConnectionStream,
};
use hotaru_core::app::application::App;

// ============================================================================
// Shared Chat State
// ============================================================================

#[derive(Clone)]
pub struct ChatRoom {
    pub messages: Arc<RwLock<Vec<ChatMessage>>>,
    pub users: Arc<RwLock<HashMap<String, String>>>, // id -> name
}

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub user: String,
    pub content: String,
    pub timestamp: u64,
}

impl ChatRoom {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(RwLock::new(Vec::new())),
            users: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn add_message(&self, user: String, content: String) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        self.messages.write().await.push(ChatMessage {
            user,
            content,
            timestamp,
        });
    }
    
    pub async fn add_user(&self, id: String, name: String) {
        self.users.write().await.insert(id, name);
    }
    
    pub async fn remove_user(&self, id: &str) {
        self.users.write().await.remove(id);
    }
    
    pub async fn get_recent_messages(&self, count: usize) -> Vec<ChatMessage> {
        let messages = self.messages.read().await;
        let start = messages.len().saturating_sub(count);
        messages[start..].to_vec()
    }
    
    pub async fn get_users(&self) -> Vec<String> {
        self.users.read().await.values().cloned().collect()
    }
}

// ============================================================================
// TCP Chat Protocol
// ============================================================================

#[derive(Clone)]
pub struct TcpChat {
    role: ProtocolRole,
    pub chat_room: ChatRoom,
}

impl TcpChat {
    pub fn new(role: ProtocolRole) -> Self {
        Self {
            role,
            chat_room: ChatRoom::new(),
        }
    }
    
    pub fn with_room(role: ProtocolRole, room: ChatRoom) -> Self {
        Self {
            role,
            chat_room: room,
        }
    }
}

// Simple transport
#[derive(Clone)]
pub struct TcpChatTransport;

impl Transport for TcpChatTransport {
    fn id(&self) -> i128 { 1 }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

// Simple stream
#[derive(Clone)]
pub struct TcpChatStream;

impl Stream for TcpChatStream {
    fn id(&self) -> u32 { 0 }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

// Message type
#[derive(Clone, Debug)]
pub enum TcpChatMessage {
    Join(String),
    Message(String),
    List,
    History,
    Leave,
    Response(String),
}

impl TcpChatMessage {
    pub fn parse(data: &[u8]) -> Self {
        let text = String::from_utf8_lossy(data).trim().to_string();
        
        if text.starts_with("JOIN ") {
            TcpChatMessage::Join(text[5..].to_string())
        } else if text.starts_with("MSG ") {
            TcpChatMessage::Message(text[4..].to_string())
        } else if text == "LIST" {
            TcpChatMessage::List
        } else if text == "HISTORY" {
            TcpChatMessage::History
        } else if text == "LEAVE" {
            TcpChatMessage::Leave
        } else {
            TcpChatMessage::Message(text)
        }
    }
    
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            TcpChatMessage::Response(msg) => format!("{}\n", msg).into_bytes(),
            _ => Vec::new(),
        }
    }
}

impl Message for TcpChatMessage {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), Box<dyn Error + Send + Sync>> {
        buf.extend_from_slice(&self.to_bytes());
        Ok(())
    }
    
    fn decode(buf: &mut bytes::BytesMut) -> Result<Option<Self>, Box<dyn Error + Send + Sync>>
    where
        Self: Sized,
    {
        if let Some(newline_pos) = buf.iter().position(|&b| b == b'\n') {
            let data = buf.split_to(newline_pos + 1);
            Ok(Some(TcpChatMessage::parse(&data)))
        } else {
            Ok(None)
        }
    }
}

// Context
pub struct TcpChatContext {
    pub response: TcpChatMessage,
    role: ProtocolRole,
}

impl RequestContext for TcpChatContext {
    type Request = TcpChatMessage;
    type Response = TcpChatMessage;
    
    fn handle_error(&mut self) {
        self.response = TcpChatMessage::Response("ERROR".to_string());
    }
    
    fn role(&self) -> ProtocolRole {
        self.role
    }
}

#[async_trait]
impl Protocol for TcpChat {
    type Transport = TcpChatTransport;
    type Stream = TcpChatStream;
    type Message = TcpChatMessage;
    type Context = TcpChatContext;
    
    fn detect(initial_bytes: &[u8]) -> bool {
        let text = String::from_utf8_lossy(initial_bytes);
        text.starts_with("JOIN ") ||
        text.starts_with("MSG ") ||
        text.starts_with("LIST") ||
        text.starts_with("HISTORY") ||
        text.starts_with("CHAT:")
    }
    
    fn role(&self) -> ProtocolRole {
        self.role
    }
    
    async fn handle(
        &mut self,
        reader: tokio::io::BufReader<tokio::io::ReadHalf<TcpConnectionStream>>,
        writer: tokio::io::BufWriter<tokio::io::WriteHalf<TcpConnectionStream>>,
        _app: Arc<App>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let stream = TcpConnectionStream::from_parts(reader.into_inner(), writer.into_inner());
        let (read_half, write_half) = stream.split();
        let mut reader = tokio::io::BufReader::new(read_half);
        let mut writer = tokio::io::BufWriter::new(write_half);
        
        // Send welcome
        let welcome = "Welcome to TCP Chat! Commands: JOIN <name>, MSG <message>, LIST, HISTORY, LEAVE\n";
        writer.write_all(welcome.as_bytes()).await?;
        writer.flush().await?;
        
        let user_id = format!("tcp_{}", std::process::id());
        let mut username = String::from("Anonymous");
        let mut buffer = [0u8; 1024];
        
        loop {
            let n = match reader.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => n,
                Err(_) => break,
            };
            
            let msg = TcpChatMessage::parse(&buffer[..n]);
            
            let response = match msg {
                TcpChatMessage::Join(name) => {
                    username = name.clone();
                    self.chat_room.add_user(user_id.clone(), username.clone()).await;
                    self.chat_room.add_message("System".to_string(), 
                        format!("{} joined the chat", username)).await;
                    format!("Welcome, {}!\n", username)
                }
                TcpChatMessage::Message(content) => {
                    self.chat_room.add_message(username.clone(), content.clone()).await;
                    format!("Message sent: {}\n", content)
                }
                TcpChatMessage::List => {
                    let users = self.chat_room.get_users().await;
                    format!("Online users: {}\n", users.join(", "))
                }
                TcpChatMessage::History => {
                    let messages = self.chat_room.get_recent_messages(10).await;
                    let mut output = String::from("Recent messages:\n");
                    for msg in messages {
                        output.push_str(&format!("[{}]: {}\n", msg.user, msg.content));
                    }
                    output
                }
                TcpChatMessage::Leave => {
                    self.chat_room.remove_user(&user_id).await;
                    self.chat_room.add_message("System".to_string(), 
                        format!("{} left the chat", username)).await;
                    writer.write_all(b"Goodbye!\n").await?;
                    writer.flush().await?;
                    break;
                }
                _ => "Unknown command\n".to_string(),
            };
            
            writer.write_all(response.as_bytes()).await?;
            writer.flush().await?;
        }
        
        self.chat_room.remove_user(&user_id).await;
        Ok(())
    }
}