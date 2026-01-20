use std::iter::Peekable;

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

pub fn generate_compile_error(span: proc_macro::Span, message: &str) -> TokenStream {
    let mut tokens = TokenStream::new();
    let ident = TokenTree::Ident(Ident::new("compile_error", span));
    let mut punct = TokenTree::Punct(Punct::new('!', Spacing::Alone));
    punct.set_span(span);
    let mut message = Literal::string(message);
    message.set_span(span);
    let mut message = TokenTree::Group(proc_macro::Group::new(
        proc_macro::Delimiter::Parenthesis,
        TokenStream::from(TokenTree::Literal(message)),
    ));
    message.set_span(span);
    let mut semi_column = Punct::new(';', Spacing::Alone);
    semi_column.set_span(span);
    let mut semi_column = TokenTree::Punct(semi_column);
    semi_column.set_span(span);
    tokens.extend(vec![ident, punct, message, semi_column]);
    tokens
}

pub use crate::outer_attr::*;

/// If the next token in the stream matches the given identifier, consume it and return true. 
/// Otherwise, return false without consuming anything. 
pub fn match_ident_consume<T: AsRef<str>>(
    stream: &mut Peekable<impl Iterator<Item = TokenTree>>, 
    token: T
) -> bool { 
    match stream.peek() {
        Some(TokenTree::Ident(ident)) if ident.to_string() == token.as_ref() => {
            stream.next(); 
            true 
        } 
        _ => false, 
    } 
} 

/// Expect the next token in the stream to be the given identifier. 
/// If it matches, consume it and return its string representation. 
/// If it does not match, return a compile error TokenStream with the given error message. 
pub fn expect_ident_consume<T: AsRef<str>>(
    stream: &mut Peekable<impl Iterator<Item = TokenTree>>, 
    token: T, 
    error: T
) -> Result<Ident, TokenStream> { 
    match stream.peek() {
        Some(TokenTree::Ident(ident)) if ident.to_string() == token.as_ref() => {
            let ident = ident.clone(); // Preserve the ident before consuming 
            stream.next(); 
            Ok(ident.clone()) 
        } 
        Some(tt) => Err(generate_compile_error(
            tt.span(), 
            error.as_ref()
        )), 
        None => Err(generate_compile_error(
            Span::call_site(), 
            error.as_ref()
        )), 
    } 
} 

/// Expect the next token in the stream to be an identifier. 
/// If it is, consume it and return it. 
/// If it is not, return a compile error TokenStream with the given error message. 
pub fn expect_any_ident<T: AsRef<str>>(
    stream: &mut Peekable<impl Iterator<Item = TokenTree>>, 
    error: T
) -> Result<Ident, TokenStream> { 
    match stream.peek() {
        Some(TokenTree::Ident(ident)) => {
            let ident = ident.clone(); // Preserve the ident before consuming 
            stream.next(); 
            Ok(ident.clone()) 
        } 
        Some(tt) => Err(generate_compile_error(
            tt.span(), 
            error.as_ref()
        )), 
        None => Err(generate_compile_error(
            Span::call_site(), 
            error.as_ref()
        )), 
    } 
}

/// If the next token in the stream matches the given punctuation, consume it and return true. 
/// Otherwise, return false without consuming anything.
pub fn match_punct_consume<T: AsRef<str>>(
    stream: &mut Peekable<impl Iterator<Item = TokenTree>>, 
    token: T
) -> bool { 
    match stream.peek() {
        Some(TokenTree::Punct(punct)) if punct.as_char().to_string() == token.as_ref() => {
            stream.next(); 
            true 
        } 
        _ => false, 
    } 
} 

/// Expect the next token in the stream to be the given punctuation. 
/// If it matches, consume it and return it. 
/// If it does not match, return a compile error TokenStream with the given error message. 
pub fn expect_punct_consume<T: AsRef<str>, U: AsRef<str>>(
    stream: &mut Peekable<impl Iterator<Item = TokenTree>>, 
    token: T, 
    error: U
) -> Result<Punct, TokenStream> { 
    match stream.peek() {
        Some(TokenTree::Punct(punct)) if punct.as_char().to_string() == token.as_ref() => {
            let ch = punct.clone(); 
            stream.next(); 
            Ok(ch) 
        } 
        Some(tt) => Err(generate_compile_error(
            tt.span(), 
            error.as_ref()
        )), 
        None => Err(generate_compile_error(
            Span::call_site(), 
            error.as_ref()
        )), 
    } 
} 

pub fn expect_group_consume_return_inner(
    stream: &mut Peekable<impl Iterator<Item = TokenTree>>, 
    delimiter: Delimiter, 
    error: &str
) -> Result<TokenStream, TokenStream> { 
    match stream.peek() {
        Some(TokenTree::Group(group)) if group.delimiter() == delimiter => {
            let inner = group.stream(); 
            stream.next(); 
            Ok(inner) 
        } 
        Some(tt) => Err(generate_compile_error(
            tt.span(), 
            error
        )), 
        None => Err(generate_compile_error(
            Span::call_site(), 
            error
        )), 
    } 
} 

/// Parse an array-like token stream: [item1, item2, item3] 
/// Return a vector of TokenStreams, each representing an item in the array. 
pub fn expect_array_consume<T: AsRef<str>>(
    tokens: &mut Peekable<impl Iterator<Item = TokenTree>>, 
    error: T, 
) -> Result<Vec<TokenStream>, TokenStream> {
    match tokens.next() {
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Bracket => {
            let mut array = Vec::new();
            let mut current = TokenStream::new();
            let mut inside_tokens = group.stream().into_iter();
            loop {
                match inside_tokens.next() {
                    Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => {
                        array.push(current);
                        current = TokenStream::new();
                    }
                    Some(token) => current.extend(std::iter::once(token)),
                    None => {
                        array.push(current);
                        break;
                    }
                }
            }
            Ok(array)
        }
        Some(tt) => Err(generate_compile_error(
            tt.span(),
            error.as_ref(),
        )),
        None => Err(generate_compile_error(
            Span::call_site(),
            error.as_ref(),
        )),
    }
}

/// Consume tokens until the next top-level comma, returning the collected stream.
/// If `must_have_comma` is true, return an error when the comma is not found.
pub fn expect_stream_before_comma_consume<T: AsRef<str>>(
    tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
    must_have_comma: bool,
    error: T,
) -> Result<TokenStream, TokenStream> {
    let mut out = TokenStream::new();
    loop {
        match tokens.next() {
            Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => {
                return Ok(out);
            }
            Some(token) => out.extend(std::iter::once(token)),
            None => {
                if must_have_comma {
                    return Err(generate_compile_error(
                        Span::call_site(),
                        error.as_ref(),
                    ));
                }
                return Ok(out);
            }
        }
    }
}

pub fn into_peekable_iter(
    tokens: TokenStream
) -> Peekable<impl Iterator<Item = TokenTree>> {
    tokens.into_iter().peekable()
} 
