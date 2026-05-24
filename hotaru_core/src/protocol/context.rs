use crate::protocol::{Channel, ProtocolError, ProtocolRole};

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
pub trait RequestContext: Default + Send + 'static {
    /// The request type for this context
    type Request;

    /// The response type for this context
    type Response;

    /// The error type produced by middleware, handlers, and the protocol
    /// that owns this context.
    ///
    /// The `From<std::io::Error>` bound is universal: the client path
    /// touches transport-level I/O (`Outbound::build`, `Outbound::connect`)
    /// which surfaces `std::io::Error`, and every protocol's error type
    /// has to absorb that to propagate cleanly through the chain. Making
    /// the bound part of the trait keeps it off every Client method's
    /// where-clause.
    ///
    /// If you don't want to define your own error type, use
    /// [`EmptyError`](crate::protocol::EmptyError) — a zero-payload
    /// stand-in that already satisfies every bound.
    type Error: ProtocolError + From<std::io::Error>;

    /// Type-system anchor for the channel of the current exchange. No
    /// accessor is exposed on this trait; the matching `Protocol` impl
    /// reaches the channel through visibility-controlled accessors on
    /// the concrete context type.
    type Channel: Channel;

    /// Handle protocol errors (bad request for server, bad response for client)
    fn handle_error(&mut self);

    /// Get the role of this context (Server or Client)
    fn role(&self) -> ProtocolRole;

    /// Install a user-provided request into a freshly-built context.
    /// Called by `Client::request_fn` before running the outpoint chain.
    fn inject_request(&mut self, request: Self::Request);

    /// Consume the context and return its response. Called by
    /// `Client::request_fn` / `Server::request_fn` after the chain finishes.
    fn into_response(self) -> Self::Response;
}
