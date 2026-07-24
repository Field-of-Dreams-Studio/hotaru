use core::iter::Peekable;

use proc_macro::{Ident, Literal, Punct, Span, TokenStream, TokenTree};

use super::generate_compile_error;

pub fn match_ident_consume<T: AsRef<str>>(
    stream: &mut Peekable<impl Iterator<Item = TokenTree>>,
    token: T,
) -> bool {
    match stream.peek() {
        Some(TokenTree::Ident(ident)) if ident.to_string() == token.as_ref() => {
            stream.next();
            true
        }
        _ => false,
    }
}

pub fn expect_ident_consume<T: AsRef<str>>(
    stream: &mut Peekable<impl Iterator<Item = TokenTree>>,
    token: T,
    error: T,
) -> Result<Ident, TokenStream> {
    match stream.peek() {
        Some(TokenTree::Ident(ident)) if ident.to_string() == token.as_ref() => {
            let ident = ident.clone();
            stream.next();
            Ok(ident.clone())
        }
        Some(tt) => Err(generate_compile_error(tt.span(), error.as_ref())),
        None => Err(generate_compile_error(Span::call_site(), error.as_ref())),
    }
}

pub fn expect_any_ident<T: AsRef<str>>(
    stream: &mut Peekable<impl Iterator<Item = TokenTree>>,
    error: T,
) -> Result<Ident, TokenStream> {
    match stream.peek() {
        Some(TokenTree::Ident(ident)) => {
            let ident = ident.clone();
            stream.next();
            Ok(ident.clone())
        }
        Some(tt) => Err(generate_compile_error(tt.span(), error.as_ref())),
        None => Err(generate_compile_error(Span::call_site(), error.as_ref())),
    }
}

pub fn match_punct_consume<T: AsRef<str>>(
    stream: &mut Peekable<impl Iterator<Item = TokenTree>>,
    token: T,
) -> bool {
    match stream.peek() {
        Some(TokenTree::Punct(punct)) if punct.as_char().to_string() == token.as_ref() => {
            stream.next();
            true
        }
        _ => false,
    }
}

pub fn expect_punct_consume<T: AsRef<str>, U: AsRef<str>>(
    stream: &mut Peekable<impl Iterator<Item = TokenTree>>,
    token: T,
    error: U,
) -> Result<Punct, TokenStream> {
    match stream.peek() {
        Some(TokenTree::Punct(punct)) if punct.as_char().to_string() == token.as_ref() => {
            let ch = punct.clone();
            stream.next();
            Ok(ch)
        }
        Some(tt) => Err(generate_compile_error(tt.span(), error.as_ref())),
        None => Err(generate_compile_error(Span::call_site(), error.as_ref())),
    }
}

pub fn expect_literal_consume<T: AsRef<str>>(
    stream: &mut Peekable<impl Iterator<Item = TokenTree>>,
    error: T,
) -> Result<Literal, TokenStream> {
    match stream.peek() {
        Some(TokenTree::Literal(lit)) => {
            let lit = lit.clone();
            stream.next();
            Ok(lit.clone())
        }
        Some(tt) => Err(generate_compile_error(tt.span(), error.as_ref())),
        None => Err(generate_compile_error(Span::call_site(), error.as_ref())),
    }
}

pub fn match_any_literal_consume(
    stream: &mut Peekable<impl Iterator<Item = TokenTree>>,
) -> Option<Literal> {
    match stream.peek() {
        Some(TokenTree::Literal(lit)) => {
            let lit = lit.clone();
            stream.next();
            Some(lit.clone())
        }
        _ => None,
    }
}

pub fn into_peekable_iter(tokens: TokenStream) -> Peekable<impl Iterator<Item = TokenTree>> {
    tokens.into_iter().peekable()
}

/// Succeeds only when no token remains in this cursor.
pub fn expect_end<T: AsRef<str>>(
    stream: &mut Peekable<impl Iterator<Item = TokenTree>>,
    error: T,
) -> Result<(), TokenStream> {
    match stream.next() {
        Some(token) => Err(generate_compile_error(token.span(), error.as_ref())),
        None => Ok(()),
    }
}

/// Consumes and returns the next Rust string literal.
pub(crate) fn expect_string_literal_consume(
    tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
) -> Result<Literal, TokenStream> {
    match tokens.next() {
        Some(TokenTree::Literal(literal)) => {
            ensure_string_literal(&literal)?;
            Ok(literal)
        }
        Some(token) => Err(generate_compile_error(
            token.span(),
            "expected a string literal",
        )),
        None => Err(generate_compile_error(
            Span::call_site(),
            "expected a string literal",
        )),
    }
}

/// Rejects literal tokens that are not ordinary or raw Rust strings.
pub(crate) fn ensure_string_literal(literal: &Literal) -> Result<(), TokenStream> {
    let source = literal.to_string();
    if source.starts_with('"') || source.starts_with("r\"") || source.starts_with("r#") {
        Ok(())
    } else {
        Err(generate_compile_error(
            literal.span(),
            "expected a string literal",
        ))
    }
}
