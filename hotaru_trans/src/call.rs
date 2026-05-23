//! `call!` and `run!` proc-macros — invoke a registered outpoint.
//!
//! ```ignore
//! call!(APP<HTTP>::ping)              // -> APP.call_fn::<HTTP>("ping")
//! call!(APP<HTTP>: "/ping")           // -> APP.call_url::<HTTP>("/ping")
//! run!(APP<HTTP>::ping, request)      // -> APP.request_fn::<HTTP>("ping", request)
//! ```
//!
//! Both expand to a method-call expression on `app`; the caller awaits
//! and handles the outer `Result<_, UrlError>` themselves. `call!` spawns
//! a persistent task; `run!` is one-shot.

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

use crate::helper::generate_compile_error;

pub enum CallTarget {
    /// `::ident` form — emits `call_fn::<P>("ident")`.
    Named(Ident),
    /// `: "literal"` form — emits `call_url::<P>("literal")`.
    Path(Literal),
}

pub struct CallArgs {
    pub app: Ident,
    pub protocol: Vec<TokenTree>,
    pub target: CallTarget,
}

pub fn parse_call(input: TokenStream) -> Result<CallArgs, TokenStream> {
    let mut tokens = input.into_iter();

    // 1. app ident
    let app = match tokens.next() {
        Some(TokenTree::Ident(ident)) => ident,
        Some(other) => return Err(generate_compile_error(other.span(), "expected app identifier")),
        None => return Err(generate_compile_error(Span::call_site(), "empty `call!` input")),
    };

    // 2. `<` — opens the protocol type parameter
    match tokens.next() {
        Some(TokenTree::Punct(p)) if p.as_char() == '<' => {}
        Some(other) => {
            return Err(generate_compile_error(other.span(), "expected `<Protocol>` after app name"));
        }
        None => {
            return Err(generate_compile_error(Span::call_site(), "expected `<Protocol>` after app name"));
        }
    }

    // Capture protocol tokens up to the matching `>`, depth-counted so
    // nested generics (e.g. `Http1Protocol<TcpStream>`) work.
    let mut protocol: Vec<TokenTree> = Vec::new();
    let mut depth: usize = 1;
    loop {
        match tokens.next() {
            Some(TokenTree::Punct(p)) if p.as_char() == '<' => {
                depth += 1;
                protocol.push(TokenTree::Punct(p));
            }
            Some(TokenTree::Punct(p)) if p.as_char() == '>' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
                protocol.push(TokenTree::Punct(p));
            }
            Some(tt) => protocol.push(tt),
            None => {
                return Err(generate_compile_error(Span::call_site(), "unterminated `<Protocol>`"));
            }
        }
    }

    // 3. `::ident` (Named) or `: "literal"` (Path).
    let target = match tokens.next() {
        Some(TokenTree::Punct(p1)) if p1.as_char() == ':' && p1.spacing() == Spacing::Joint => {
            // Joint `:` followed by `:` is the `::` token pair.
            match tokens.next() {
                Some(TokenTree::Punct(p2)) if p2.as_char() == ':' => {}
                _ => return Err(generate_compile_error(p1.span(), "expected `::` after protocol")),
            }
            match tokens.next() {
                Some(TokenTree::Ident(ident)) => CallTarget::Named(ident),
                _ => return Err(generate_compile_error(p1.span(), "expected identifier after `::`")),
            }
        }
        Some(TokenTree::Punct(p)) if p.as_char() == ':' => {
            match tokens.next() {
                Some(TokenTree::Literal(lit)) => CallTarget::Path(lit),
                _ => return Err(generate_compile_error(p.span(), "expected string literal after `:`")),
            }
        }
        Some(other) => {
            return Err(generate_compile_error(
                other.span(),
                "expected `::name` or `: \"/path\"` after protocol",
            ));
        }
        None => {
            return Err(generate_compile_error(
                Span::call_site(),
                "expected `::name` or `: \"/path\"` after protocol",
            ));
        }
    };

    Ok(CallArgs { app, protocol, target })
}

pub struct RunArgs {
    pub app: Ident,
    pub protocol: Vec<TokenTree>,
    pub name: Ident,
    /// Caller-supplied request expression — captured verbatim and passed as
    /// the second argument to `request_fn`.
    pub request: TokenStream,
}

pub fn parse_run(input: TokenStream) -> Result<RunArgs, TokenStream> {
    let mut tokens = input.into_iter();

    // app + `<Protocol>` follow the same shape as `call!`.
    let app = match tokens.next() {
        Some(TokenTree::Ident(ident)) => ident,
        Some(other) => return Err(generate_compile_error(other.span(), "expected app identifier")),
        None => return Err(generate_compile_error(Span::call_site(), "empty `run!` input")),
    };
    match tokens.next() {
        Some(TokenTree::Punct(p)) if p.as_char() == '<' => {}
        Some(other) => {
            return Err(generate_compile_error(other.span(), "expected `<Protocol>` after app name"));
        }
        None => {
            return Err(generate_compile_error(Span::call_site(), "expected `<Protocol>` after app name"));
        }
    }
    let mut protocol: Vec<TokenTree> = Vec::new();
    let mut depth: usize = 1;
    loop {
        match tokens.next() {
            Some(TokenTree::Punct(p)) if p.as_char() == '<' => {
                depth += 1;
                protocol.push(TokenTree::Punct(p));
            }
            Some(TokenTree::Punct(p)) if p.as_char() == '>' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
                protocol.push(TokenTree::Punct(p));
            }
            Some(tt) => protocol.push(tt),
            None => {
                return Err(generate_compile_error(Span::call_site(), "unterminated `<Protocol>`"));
            }
        }
    }

    // `::name` (only the named form is supported for `run!` until a
    // `request_url` shape lands on Client/Server).
    match tokens.next() {
        Some(TokenTree::Punct(p1)) if p1.as_char() == ':' && p1.spacing() == Spacing::Joint => {
            match tokens.next() {
                Some(TokenTree::Punct(p2)) if p2.as_char() == ':' => {}
                _ => return Err(generate_compile_error(p1.span(), "expected `::` after protocol")),
            }
        }
        Some(other) => {
            return Err(generate_compile_error(other.span(), "expected `::name` after protocol"));
        }
        None => {
            return Err(generate_compile_error(Span::call_site(), "expected `::name` after protocol"));
        }
    }
    let name = match tokens.next() {
        Some(TokenTree::Ident(ident)) => ident,
        _ => return Err(generate_compile_error(Span::call_site(), "expected identifier after `::`")),
    };

    // Expect `,` then capture the rest as the request expression.
    match tokens.next() {
        Some(TokenTree::Punct(p)) if p.as_char() == ',' => {}
        Some(other) => {
            return Err(generate_compile_error(other.span(), "expected `,` before request expression"));
        }
        None => {
            return Err(generate_compile_error(Span::call_site(), "expected `,` then request expression"));
        }
    }
    let request: TokenStream = tokens.collect();
    if request.clone().into_iter().next().is_none() {
        return Err(generate_compile_error(Span::call_site(), "expected request expression after `,`"));
    }

    Ok(RunArgs { app, protocol, name, request })
}

impl RunArgs {
    pub fn expand(&self) -> TokenStream {
        // <app>.request_fn::<<protocol>>(<name_literal>, <request>)
        let mut out = TokenStream::new();
        out.extend(vec![
            TokenTree::Ident(self.app.clone()),
            TokenTree::Punct(Punct::new('.', Spacing::Alone)),
            TokenTree::Ident(Ident::new("request_fn", Span::call_site())),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Punct(Punct::new('<', Spacing::Alone)),
        ]);
        out.extend(self.protocol.iter().cloned());
        out.extend(vec![
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, {
                let mut g = TokenStream::new();
                g.extend(vec![
                    TokenTree::Literal(Literal::string(&self.name.to_string())),
                    TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                ]);
                g.extend(self.request.clone());
                g
            })),
        ]);
        out
    }
}

impl CallArgs {
    pub fn expand(&self) -> TokenStream {
        // Method name + arg shape vary by target form.
        let (method, arg): (&'static str, TokenTree) = match &self.target {
            CallTarget::Named(ident) => (
                "call_fn",
                TokenTree::Literal(Literal::string(&ident.to_string())),
            ),
            CallTarget::Path(lit) => ("call_url", TokenTree::Literal(lit.clone())),
        };

        // <app>.<method>::<<protocol>>(<arg>)
        let mut out = TokenStream::new();
        out.extend(vec![
            TokenTree::Ident(self.app.clone()),
            TokenTree::Punct(Punct::new('.', Spacing::Alone)),
            TokenTree::Ident(Ident::new(method, Span::call_site())),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Punct(Punct::new('<', Spacing::Alone)),
        ]);
        out.extend(self.protocol.iter().cloned());
        out.extend(vec![
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, {
                let mut g = TokenStream::new();
                g.extend(std::iter::once(arg));
                g
            })),
        ]);
        out
    }
}
