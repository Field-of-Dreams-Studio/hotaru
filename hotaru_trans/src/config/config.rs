use proc_macro::TokenStream;

use super::Cloneable;

/// Ordered configuration expressions retained until code generation.
pub(crate) struct Config {
    entries: Vec<TokenStream>,
    cloneable: Cloneable,
}

impl Config {
    pub(crate) fn new(entries: Vec<TokenStream>, cloneable: Cloneable) -> Self {
        Self { entries, cloneable }
    }

    pub(crate) fn with_entries(mut self, entries: Vec<TokenStream>) -> Self {
        self.entries = entries;
        self
    }

    pub(crate) fn with_cloneable(mut self, cloneable: Cloneable) -> Self {
        self.cloneable = cloneable;
        self
    }

    pub(crate) fn push(&mut self, entry: TokenStream) {
        self.entries.push(entry);
    }

    pub(crate) fn entries(&self) -> &[TokenStream] {
        &self.entries
    }

    pub(crate) fn cloneable(&self) -> Cloneable {
        self.cloneable
    }

    pub(crate) fn into_parts(self) -> (Vec<TokenStream>, Cloneable) {
        (self.entries, self.cloneable)
    }
}
