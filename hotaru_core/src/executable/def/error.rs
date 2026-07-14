use crate::prelude::String;
use crate::url::UrlError;

/// Contextual error returned by `App::bind` / `bind_all`. Carries the
/// offending route identity so a large blueprint can name which item
/// failed.
#[derive(Debug, Clone, PartialEq)]
pub struct BindError {
    route_name: String,
    route_url: String,
    source: UrlError,
    batch_index: Option<usize>,
}

impl BindError {
    pub fn new(
        route_name: impl Into<String>,
        route_url: impl Into<String>,
        source: UrlError,
    ) -> Self {
        Self {
            route_name: route_name.into(),
            route_url: route_url.into(),
            source,
            batch_index: None,
        }
    }

    pub fn with_batch_index(mut self, index: usize) -> Self {
        self.batch_index = Some(index);
        self
    }

    pub fn route_name(&self) -> &str { &self.route_name }
    pub fn route_url(&self) -> &str { &self.route_url }
    pub fn source_error(&self) -> &UrlError { &self.source }
    pub fn batch_index(&self) -> Option<usize> { self.batch_index }
}

impl core::fmt::Display for BindError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.batch_index {
            Some(i) => write!(
                f,
                "bind error at batch index {} on route {:?} ({}): {}",
                i, self.route_name, self.route_url, self.source
            ),
            None => write!(
                f,
                "bind error on route {:?} ({}): {}",
                self.route_name, self.route_url, self.source
            ),
        }
    }
}

impl core::error::Error for BindError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        Some(&self.source)
    }
} 
