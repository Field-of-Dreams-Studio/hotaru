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
