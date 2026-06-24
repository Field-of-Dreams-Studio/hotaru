//! Preferred-language negotiation for Hotaru/htmstd.
//!
//! The module is split by responsibility:
//! - [`types`]: the [`PreferredLanguage`] / [`LanguageRange`] data model
//! - [`parser`]: pure `Accept-Language` parsing and matching helpers
//! - [`settings`]: [`PreferredLanguageSettings`] configuration
//! - [`middleware`]: the [`PreferredLanguageMiddleware`]
//! - [`ext`]: the [`PreferredLanguageRequestExt`] request convenience trait

pub mod ext;
pub mod middleware;
pub mod parser;
pub mod settings;
pub mod types;

pub use self::ext::PreferredLanguageRequestExt;
pub use self::middleware::PreferredLanguageMiddleware;
pub use self::settings::PreferredLanguageSettings;
pub use self::types::{LanguageRange, MAX_QUALITY_MILLIS, PreferredLanguage};
