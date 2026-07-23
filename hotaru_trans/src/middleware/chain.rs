use core::iter::Peekable;
use proc_macro::{Ident, Punct, Spacing, Span, TokenStream, TokenTree};

use crate::helper::{
    expect_array_consume, expect_punct_consume, generate_compile_error, into_peekable_iter,
    use_core,
};

pub enum MWSlot {
    Concrete(Ident),
    Inherit,
}

impl MWSlot {
    pub fn get_next(
        stream: &mut Peekable<impl Iterator<Item = TokenTree>>,
    ) -> Result<MWSlot, TokenStream> {
        let next_token = stream.next();
        match next_token {
            Some(TokenTree::Ident(ident)) => {
                if let Some(token) = stream.peek() {
                    return Err(generate_compile_error(
                        token.span(),
                        "Expected exactly one middleware identifier",
                    ));
                }
                return Ok(MWSlot::Concrete(ident));
            }
            Some(TokenTree::Punct(punct)) if punct.as_char() == '.' => {
                expect_punct_consume(stream, ".", "Expect `..` to represent inherited middleware")?;
                if let Some(token) = stream.peek() {
                    return Err(generate_compile_error(
                        token.span(),
                        "Expected exactly `..` for inherited middleware",
                    ));
                }
                return Ok(MWSlot::Inherit);
            }
            Some(token) => Err(generate_compile_error(
                token.span(),
                "expected one middleware identifier or `..`",
            )),
            None => Err(generate_compile_error(
                Span::call_site(),
                "expected one middleware identifier or `..`",
            )),
        }
    }

    pub fn is_concrete(&self) -> bool {
        match self {
            MWSlot::Concrete(_) => true,
            MWSlot::Inherit => false,
        }
    }

    pub fn is_inherited(&self) -> bool {
        match self {
            MWSlot::Concrete(_) => false,
            MWSlot::Inherit => true,
        }
    }

    pub fn expand_slot(&self) -> TokenStream {
        match self {
            MWSlot::Concrete(ident) => {
                let mut ts = TokenStream::new();
                ts.extend(use_core(&["executable", "def", "MWSlot", "Concrete"]));
                ts.extend(TokenStream::from(TokenTree::Group(proc_macro::Group::new(
                    proc_macro::Delimiter::Parenthesis,
                    // ::hotaru::prelude::Arc::new(expr)
                    TokenStream::from_iter(vec![
                        use_core(&["prelude", "Arc", "new"]),
                        TokenTree::Group(proc_macro::Group::new(
                            proc_macro::Delimiter::Parenthesis,
                            TokenStream::from(TokenTree::Ident(ident.clone())),
                        ))
                        .into(),
                    ]),
                ))));
                ts
            }
            MWSlot::Inherit => {
                let mut ts = TokenStream::new();
                ts.extend(use_core(&["executable", "def", "MWSlot", "Inherit"]));
                ts
            }
        }
    }
}

pub struct MWChain {
    slots: Vec<MWSlot>,
}

impl MWChain {
    pub fn new(slots: Vec<MWSlot>) -> Self {
        Self { slots }
    }

    /// Default AP chain when the DSL has no `middleware = [...]` clause.
    pub(crate) fn inheriting() -> Self {
        Self::new(vec![MWSlot::Inherit])
    }

    pub fn from_stream(
        stream: &mut Peekable<impl Iterator<Item = TokenTree>>,
    ) -> Result<MWChain, TokenStream> {
        let mut slots = Vec::new();
        let v = expect_array_consume(stream, "expect an array of middleware slots")?;
        for token in v {
            slots.push(MWSlot::get_next(&mut into_peekable_iter(token))?)
        }
        Ok(MWChain::new(slots))
    }

    pub fn expand_middleware_chain(self) -> TokenStream {
        let mut body = TokenStream::new();

        // let mut mw_chain = ::...::MWChain::new(::...::Vec::new());
        body.extend([
            TokenTree::Ident(Ident::new("let", Span::call_site())),
            TokenTree::Ident(Ident::new("mut", Span::call_site())),
            TokenTree::Ident(Ident::new("mw_chain", Span::call_site())),
            TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        ]);
        body.extend(use_core(&["executable", "def", "MWChain", "new"]));

        let mut empty_slots = use_core(&["prelude", "Vec", "new"]);
        empty_slots.extend([TokenTree::Group(proc_macro::Group::new(
            proc_macro::Delimiter::Parenthesis,
            TokenStream::new(),
        ))]);
        body.extend([
            TokenTree::Group(proc_macro::Group::new(
                proc_macro::Delimiter::Parenthesis,
                empty_slots,
            )),
            TokenTree::Punct(Punct::new(';', Spacing::Alone)),
        ]);

        for slot in self.slots {
            // mw_chain.push(<expanded slot>);
            body.extend([
                TokenTree::Ident(Ident::new("mw_chain", Span::call_site())),
                TokenTree::Punct(Punct::new('.', Spacing::Alone)),
                TokenTree::Ident(Ident::new("push", Span::call_site())),
                TokenTree::Group(proc_macro::Group::new(
                    proc_macro::Delimiter::Parenthesis,
                    slot.expand_slot(),
                )),
                TokenTree::Punct(Punct::new(';', Spacing::Alone)),
            ]);
        }

        // The final expression deliberately has no semicolon, so the
        // brace-wrapped block evaluates to the completed MWChain.
        body.extend([TokenTree::Ident(Ident::new("mw_chain", Span::call_site()))]);

        TokenStream::from(TokenTree::Group(proc_macro::Group::new(
            proc_macro::Delimiter::Brace,
            body,
        )))
    }
}
