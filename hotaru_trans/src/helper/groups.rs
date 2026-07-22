use core::iter::Peekable;

use proc_macro::{Delimiter, Span, TokenStream, TokenTree};

use super::generate_compile_error;

pub fn expect_group_consume_return_inner(
    stream: &mut Peekable<impl Iterator<Item = TokenTree>>,
    delimiter: Delimiter,
    error: &str,
) -> Result<TokenStream, TokenStream> {
    match stream.peek() {
        Some(TokenTree::Group(group)) if group.delimiter() == delimiter => {
            let inner = group.stream();
            stream.next();
            Ok(inner)
        }
        Some(tt) => Err(generate_compile_error(tt.span(), error)),
        None => Err(generate_compile_error(Span::call_site(), error)),
    }
}

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
                        if current.is_empty() {
                            return Err(generate_compile_error(punct.span(), error.as_ref()));
                        }
                        array.push(current);
                        current = TokenStream::new();
                    }
                    Some(token) => current.extend(core::iter::once(token)),
                    None => {
                        if !current.is_empty() {
                            array.push(current);
                        }
                        break;
                    }
                }
            }
            Ok(array)
        }
        Some(tt) => Err(generate_compile_error(tt.span(), error.as_ref())),
        None => Err(generate_compile_error(Span::call_site(), error.as_ref())),
    }
}

pub fn expect_stream_before_comma_consume<T: AsRef<str>>(
    tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
    must_have_comma: bool,
    error: T,
) -> Result<TokenStream, TokenStream> {
    let mut out = TokenStream::new();
    loop {
        match tokens.next() {
            Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => return Ok(out),
            Some(token) => out.extend(core::iter::once(token)),
            None => {
                if must_have_comma {
                    return Err(generate_compile_error(Span::call_site(), error.as_ref()));
                }
                return Ok(out);
            }
        }
    }
}
