use crate::protocol::ProtocolRole;

// ----------------------------------------------------------------------------
// RequestContext Trait
// ----------------------------------------------------------------------------

/// Context that flows through request handlers.
/// 
/// This trait links the request/response types that handlers work with.
/// It's the type that flows through `AsyncFinalHandler<C>` and `AsyncMiddleware<C>`.
/// 
/// Both server and client contexts implement this trait, with the role
/// determining the direction of communication.
pub trait RequestContext: Send + 'static {
    /// The request type for this context
    type Request;
    
    /// The response type for this context
    type Response;
    
    /// Handle protocol errors (bad request for server, bad response for client)
    fn handle_error(&mut self);
    
    /// Get the role of this context (Server or Client)
    fn role(&self) -> ProtocolRole;
}
