use crate::executable::def::BindError;
use crate::prelude::String;

/// Errors from Blueprint construction, admission, and application.
#[derive(Debug, Clone, PartialEq)]
pub enum BlueprintError {
    /// A group for this concrete protocol type already exists.
    DuplicateProtocol(String),
    /// No group or protocol entry exists for the requested protocol type.
    ProtocolNotFound(&'static str),
    /// The protocol set is frozen once the handle has been cloned.
    SharedBlueprint,
    /// Compiling or registering one retained definition failed.
    Bind(BindError),
}

impl core::fmt::Display for BlueprintError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::DuplicateProtocol(name) => {
                write!(f, "duplicate protocol group: {name}")
            }
            Self::ProtocolNotFound(ty) => {
                write!(f, "no protocol group or entry for {ty}")
            }
            Self::SharedBlueprint => f.write_str("blueprint is shared; protocol set is frozen"),
            Self::Bind(error) => write!(f, "blueprint bind: {error}"),
        }
    }
}

impl core::error::Error for BlueprintError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Bind(error) => Some(error),
            _ => None,
        }
    }
}

impl From<BindError> for BlueprintError {
    fn from(error: BindError) -> Self {
        Self::Bind(error)
    }
}
