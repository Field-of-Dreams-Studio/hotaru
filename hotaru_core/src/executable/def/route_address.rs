//! Registration address for an `AccessPointDef`. Mirrors the shape
//! `hotaru_trans::UrlExpr` uses on the macro side (app + method +
//! literal) so reviewers see the same structural decomposition on
//! both sides.

use crate::prelude::String;

use super::url_mode::UrlMode;

/// URL + name + parse mode. Cheap to clone (two `String`s + one
/// enum) so the getters return owned copies of the mode and
/// references to the strings.
#[derive(Clone, Debug)]
pub struct RouteAddress {
    url: String,
    name: String,
    url_mode: UrlMode,
}

impl RouteAddress {
    /// Construct with defaults: `url_mode = UrlMode::Pattern`.
    pub fn new(url: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            name: name.into(),
            url_mode: UrlMode::default(),
        }
    }

    pub fn with_url_mode(mut self, url_mode: UrlMode) -> Self {
        self.url_mode = url_mode;
        self
    }

    pub fn url(&self) -> &str {
        &self.url
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn url_mode(&self) -> UrlMode {
        self.url_mode
    }
}
