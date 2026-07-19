use proc_macro::{Ident, Literal};

/// How core should interpret the URL literal when the definition is bound.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum UrlMode {
    #[default]
    Pattern,
    Literal,
}

/// Syntax-level registration address.
///
/// `app` exists only on the macro side: it identifies the application whose
/// future registration hook will call `bind`.
pub(crate) struct RouteAddress {
    app: Ident,
    url: Literal,
    name: Ident,
    url_mode: UrlMode,
}

impl RouteAddress {
    pub(crate) fn new(app: Ident, url: Literal, name: Ident) -> Self {
        Self {
            app,
            url,
            name,
            url_mode: UrlMode::default(),
        }
    }

    pub(crate) fn with_url_mode(mut self, url_mode: UrlMode) -> Self {
        self.url_mode = url_mode;
        self
    }

    pub(crate) fn app(&self) -> &Ident {
        &self.app
    }

    pub(crate) fn url(&self) -> &Literal {
        &self.url
    }

    pub(crate) fn name(&self) -> &Ident {
        &self.name
    }

    pub(crate) fn url_mode(&self) -> UrlMode {
        self.url_mode
    }
}
