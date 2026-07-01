/// Builder for protocol entries.
pub mod builder;

pub use builder::ProtocolEntryBuilder;

/// Traits used by protocol entry dispatch.
pub mod traits;

pub use traits::ProtocolEntryTrait;

/// Concrete protocol entry implementation.
pub mod entry;

pub use entry::ProtocolEntry;
