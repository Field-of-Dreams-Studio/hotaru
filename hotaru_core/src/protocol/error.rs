// ----------------------------------------------------------------------------
// Protocol error trait — minimal, protocol-defined.
// ----------------------------------------------------------------------------

pub type BoxProtocolError = Box<dyn ProtocolError>;

/// Protocol-defined error. Each protocol owns its own concrete error type.
///
/// `can_continue` is the policy hook: when a chain returns `Err(boxed)`, the
/// protocol's `handle`/`send` decides whether the channel survives. The
/// framework never interprets this flag itself.
pub trait ProtocolError: std::error::Error + Send + Sync + 'static {
    fn can_continue(&self) -> bool {
        false
    }

    fn boxed(self) -> BoxProtocolError
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

// Blanket helper so plain `std::error::Error` types can be wrapped trivially
// when a protocol does not need richer behaviour.
impl<T> ProtocolError for T where
    T: std::error::Error + Send + Sync + 'static + DefaultProtocolError
{
}

/// Marker so the blanket impl above does not conflict with hand-written impls
/// that want custom `can_continue` behaviour. Implement this on types that
/// should get the default `can_continue() = false`.
///
/// Implement this trait on your error type if you want to use the blanket
/// `ProtocolError` impl and don't need custom `can_continue` logic.
pub trait DefaultProtocolError {}

impl DefaultProtocolError for std::io::Error {}

/// Template error type for `RequestContext::Error`.
///
/// Carries no payload — converting any source error into `EmptyError` drops
/// the source's information. Useful when prototyping a new `RequestContext`
/// impl, in tests, or in protocols that genuinely have nothing to report
/// beyond "something went wrong."
///
/// Satisfies all bounds required by `RequestContext::Error`:
/// `std::error::Error + Send + Sync + 'static + ProtocolError +
/// From<std::io::Error>`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EmptyError;

impl std::fmt::Display for EmptyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("empty error")
    }
}

impl std::error::Error for EmptyError {}

impl DefaultProtocolError for EmptyError {}

impl From<std::io::Error> for EmptyError {
    fn from(_: std::io::Error) -> Self {
        EmptyError
    }
}
