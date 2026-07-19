use core::iter::Peekable;

use proc_macro::{TokenStream, TokenTree};

use crate::helper::expect_array_consume;

use super::{Cloneable, Config};

impl Config {
    /// Parse the bracketed value after a configuration clause's separator.
    ///
    /// The caller owns the preceding `config` keyword and separator, as well as
    /// any tokens following the bracketed list.
    pub(crate) fn from_stream(
        stream: &mut Peekable<impl Iterator<Item = TokenTree>>,
        cloneable: Cloneable,
    ) -> Result<Self, TokenStream> {
        // TODO: Parse commas inside ungrouped generic arguments correctly
        // (for example, `Factory::<A, B>::new()`).
        let entries = expect_array_consume(stream, "Expected an array for config")?;
        Ok(Self::new(entries, cloneable))
    }
}
