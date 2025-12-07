//! Service implementation for bridging Hyper requests to Hotaru handlers

use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;
use std::convert::Infallible;
use std::task::{Context, Poll};

use hyper::{Request, Response, StatusCode};
use hyper::body::Incoming;
use http_body_util::{Full, Empty, BodyExt};
use bytes::Bytes;
use hyper::service::Service;

use hotaru_core::{
    app::application::App,
    connection::ProtocolRole,
};

use crate::context::{Body, HyperContext};
use crate::upgrade::manager::{UpgradeManager, UpgradeResult};

/// Service that routes Hyper requests through Hotaru's handler system
/// 
/// Generic over the protocol type to support HTTP/1, HTTP/2, HTTP/3, etc.
pub struct HotaruService<P> {
    app: Arc<App>,
    role: ProtocolRole,
    upgrade_manager: Arc<UpgradeManager>,
    _protocol: std::marker::PhantomData<P>,
}

impl<P> HotaruService<P> {
    pub fn new(app: Arc<App>, role: ProtocolRole) -> Self {
        Self { 
            app, 
            role,
            upgrade_manager: Arc::new(UpgradeManager::new()),
            _protocol: std::marker::PhantomData,
        }
    }
}

use hotaru_core::connection::Protocol;

// Implement hyper's Service trait for incoming requests
impl<P> Service<Request<Incoming>> for HotaruService<P>
where
    P: Protocol<Context = HyperContext> + 'static,
{
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, mut req: Request<Incoming>) -> Self::Future {
        let _app = self.app.clone();
        let role = self.role;
        let upgrade_manager = self.upgrade_manager.clone();
        
        Box::pin(async move {
            let path = req.uri().path().to_string();
            let method = req.method().clone();
            
            // println!("HotaruService routing request: {} {}", method, path);
            
            // Get the root handler for protocol P from the app's protocol registry
            let root_handler = match _app.handler.url::<P>() {
                Some(handler) => handler,
                None => {
                    // Return 500 if no handler is registered  
                    let error_text = format!("No handler registered for protocol {}", std::any::type_name::<P>());
                    eprintln!("Error: {}", error_text);
                    
                    return Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Full::new(Bytes::from(error_text)).boxed())
                        .unwrap());
                }
            };
            
            // Walk the URL tree to find the matching endpoint
            let endpoint = root_handler.clone().walk_str(&path).await;
            
            // Check if this is a WebSocket upgrade request early
            use crate::websocket::is_websocket_upgrade_generic;
            let is_ws_upgrade_request = is_websocket_upgrade_generic(&req);
            
            // Set up upgrade future before consuming the request
            let pending_upgrade = if is_ws_upgrade_request {
                Some(hyper::upgrade::on(&mut req))
            } else {
                None
            };
            
            // Extract request parts before consuming body
            let (parts, body) = req.into_parts();
            
            // Read the entire body into memory
            let body_bytes = body.collect().await.unwrap_or_default().to_bytes();
            let body_vec = body_bytes.to_vec();  // Clone for storing in context
            
            // Reconstruct request with the body for the context
            let hyper_req = Request::from_parts(parts, Full::new(body_bytes).boxed());
                
            // Create the context with the endpoint
            let mut ctx = HyperContext::new_server(hyper_req, _app.clone());
            ctx.endpoint = Some(endpoint.clone());
            ctx.set_body_bytes(body_vec);  // Store body bytes for form/json parsing
            
            // Run the endpoint like in the TCP example
            let mut result_ctx = endpoint.run(ctx).await;
            
            // Check if protocol switch was requested and validate the response
            let should_handle_upgrade = if let hotaru_core::connection::ConnectionStatus::SwitchProtocol(target_protocol) = &result_ctx.connection_status {
                println!("🔄 Protocol switch requested to: {:?}", target_protocol);
                
                // Check if this is a WebSocket upgrade
                use crate::websocket::WebSocketProtocol;
                if *target_protocol == std::any::TypeId::of::<WebSocketProtocol>() {
                    // Check if the response is 101 Switching Protocols
                    let response = result_ctx.response_mut();
                    if response.inner.status() == StatusCode::SWITCHING_PROTOCOLS {
                        println!("🚀 WebSocket upgrade validated");
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            };
            
            // Check if the endpoint was found or if it's a 404 (dangling URL)
            let response = result_ctx.response_mut();
            let status = response.inner.status();
            
            if status == StatusCode::BAD_REQUEST {
                // This is a dangling URL (no handler found), return 404
                // println!("❌ No endpoint found for: {} {} - returning 404", method, path);
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Full::new(Bytes::from(format!("404 Not Found: {} {}", method, path))).boxed())
                    .unwrap())
            } else {
                // Valid endpoint found - extract the actual response
                // println!("✅ Endpoint executed for: {} {}", method, path);
                
                // Extract the response that was built by the endpoint
                let mut final_response = Response::builder()
                    .status(status);
                
                // Copy headers from the context response
                for (key, value) in response.inner.headers() {
                    final_response = final_response.header(key, value);
                }
                
                // Move the body from the context response
                // We need to swap out the body since we can't clone it
                let body = std::mem::replace(
                    response.inner.body_mut(), 
                    Empty::<Bytes>::new().boxed()
                );
                
                let final_response = final_response
                    .body(body)
                    .unwrap();
                
                // If we have a pending upgrade and it was validated, spawn the handler
                if should_handle_upgrade {
                    if let Some(upgrade_future) = pending_upgrade {
                        println!("📡 Spawning WebSocket upgrade handler...");
                        
                        // Check if this is the download endpoint
                        let is_download_endpoint = path == "/download";
                        
                        tokio::spawn(async move {
                            match upgrade_future.await {
                                Ok(upgraded) => {
                                    println!("✅ WebSocket upgrade successful!");
                                    
                                    // Use appropriate handler based on endpoint
                                    if is_download_endpoint {
                                        use crate::websocket::handle_download_websocket;
                                        handle_download_websocket(upgraded).await;
                                    } else {
                                        use crate::websocket::handle_websocket_upgrade;
                                        handle_websocket_upgrade(upgraded).await;
                                    }
                                }
                                Err(e) => {
                                    eprintln!("❌ WebSocket upgrade failed: {:?}", e);
                                }
                            }
                        });
                    }
                }
                
                Ok(final_response)
            }
        })
    }
}

/// Clone implementation for the service
impl<P> Clone for HotaruService<P> {
    fn clone(&self) -> Self {
        Self {
            app: self.app.clone(),
            role: self.role,
            upgrade_manager: self.upgrade_manager.clone(),
            _protocol: std::marker::PhantomData,
        }
    }
}