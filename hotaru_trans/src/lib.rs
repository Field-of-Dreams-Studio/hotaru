use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

// Procedural macros for Hotaru framework
// Entry points will be moved here from hotaru_trans
pub(crate) mod call;
pub(crate) mod middleware;
pub(crate) mod mw_chain; 
pub(crate) mod url;

pub(crate) mod helper;
pub(crate) mod outer_attr;
use helper::*;
pub(crate) mod ctor;

/// Our own constructor attribute - works like #[ctor::ctor] but built-in
/// Generates platform-specific linker sections for automatic initialization
#[cfg(not(feature = "external-ctor"))]
#[proc_macro_attribute]
pub fn ctor(_attr: TokenStream, item: TokenStream) -> TokenStream {
    ctor::ctor(_attr, item)
}

cfg_if::cfg_if! {
    if #[cfg(feature = "trans")] {
        #[proc_macro]
        pub fn endpoint(input: TokenStream) -> TokenStream {
            url::endpoint_trans(input)
        }

        #[proc_macro]
        pub fn outpoint(input: TokenStream) -> TokenStream {
            url::outpoint_trans(input)
        }
    } else if #[cfg(feature = "attr")] {
        #[proc_macro_attribute]
        pub fn endpoint(attr: TokenStream, input: TokenStream) -> TokenStream {
            url::endpoint_attr(attr, input)
        }

        #[proc_macro_attribute]
        pub fn outpoint(attr: TokenStream, input: TokenStream) -> TokenStream {
            url::outpoint_attr(attr, input)
        }
    } else {
        // default: semi-trans
        #[proc_macro_attribute]
        pub fn endpoint(_attr: TokenStream, input: TokenStream) -> TokenStream {
            url::endpoint_semi_trans(input)
        }

        #[proc_macro_attribute]
        pub fn outpoint(_attr: TokenStream, input: TokenStream) -> TokenStream {
            url::outpoint_semi_trans(input)
        }
    }
}

/// Spawn a persistent outpoint call. Two forms:
///
///   call!(APP<HTTP>::ping)      -> APP.call_fn::<HTTP>("ping")
///   call!(APP<HTTP>: "/ping")   -> APP.call_url::<HTTP>("/ping")
///
/// Returns the method-call expression; caller awaits the
/// `Result<JoinHandle, UrlError>` themselves.
#[proc_macro]
pub fn call(input: TokenStream) -> TokenStream {
    match call::parse_call(input) {
        Ok(args) => args.expand(),
        Err(err) => err,
    }
}

/// One-shot outpoint request:
///
///   run!(APP<HTTP>::ping, request) -> APP.request_fn::<HTTP>("ping", request)
#[proc_macro]
pub fn run(input: TokenStream) -> TokenStream {
    match call::parse_run(input) {
        Ok(args) => args.expand(),
        Err(err) => err,
    }
}

/// `run_server!(APP)` — blocking entry, for sync `fn main()`.
#[proc_macro]
pub fn run_server(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    format!("::hotaru::hotaru_core::app::server::run_server(({s}).clone())")
        .parse()
        .expect("run_server! expansion")
}

/// `run_server_until!(APP, stop)` — blocking with user-supplied stop future.
#[proc_macro]
pub fn run_server_until(input: TokenStream) -> TokenStream {
    let (server, stop) = split_comma(&input);
    format!("::hotaru::hotaru_core::app::server::run_server_until(({server}).clone(), {stop})")
        .parse()
        .expect("run_server_until! expansion")
}

/// `run_server_no_block!(APP)` — fire-and-forget inside an async context.
#[proc_macro]
pub fn run_server_no_block(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    format!("::hotaru::hotaru_core::app::server::run_server_no_block(({s}).clone())")
        .parse()
        .expect("run_server_no_block! expansion")
}

/// `run_server_no_block_until!(APP, stop)` — fire-and-forget with stop.
#[proc_macro]
pub fn run_server_no_block_until(input: TokenStream) -> TokenStream {
    let (server, stop) = split_comma(&input);
    format!(
        "::hotaru::hotaru_core::app::server::run_server_no_block_until(({server}).clone(), {stop})"
    )
    .parse()
    .expect("run_server_no_block_until! expansion")
}

// Splits `input` at the first top-level comma. Used by the two `_until`
// macros. Not balance-aware for `<` / `>` (turbofish etc.) — upgrade to
// `syn` parsing if that stops holding.
fn split_comma(input: &TokenStream) -> (String, String) {
    let s = input.to_string();
    let mut depth: i32 = 0;
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            b',' if depth == 0 => {
                return (s[..i].trim().to_string(), s[i + 1..].trim().to_string());
            }
            _ => {}
        }
    }
    panic!("expected `server, stop` — got: {s}");
}

cfg_if::cfg_if! {
    if #[cfg(feature = "trans")] {
        #[proc_macro]
        pub fn middleware(input: TokenStream) -> TokenStream {
            match middleware::parse_trans(input) {
                Ok(mw_func) => mw_func.expand(),
                Err(err) => err,
            }
        }
    } else if #[cfg(feature = "attr")] {
        #[proc_macro_attribute]
        pub fn middleware(_attr: TokenStream, input: TokenStream) -> TokenStream {
            match middleware::parse_semi_trans_or_attr(input) {
                Ok(mw_func) => mw_func.expand(),
                Err(err) => err,
            }
        }
    } else {
        #[proc_macro_attribute]
        pub fn middleware(_attr: TokenStream, input: TokenStream) -> TokenStream {
            match middleware::parse_semi_trans_or_attr(input) {
                Ok(mw_func) => mw_func.expand(),
                Err(err) => err,
            }
        }
    }
}

/// Helper macro to generate lazy static declarations.
/// Used by LServer!, LClient!, LUrl!, and LPattern! macros.
macro_rules! generate_lazy_static {
    ($type_name:expr) => {
        |input: TokenStream| -> TokenStream {
            let mut tokens = input.into_iter().peekable();

            // Parse identifier
            let ident = match tokens.next() {
                Some(TokenTree::Ident(i)) => i,
                _ => {
                    return generate_compile_error(
                        Span::call_site(),
                        "Expected identifier before '='",
                    );
                }
            };

            // Expect '='
            match tokens.next() {
                Some(TokenTree::Punct(p)) if p.as_char() == '=' => {}
                _ => {
                    return generate_compile_error(
                        Span::call_site(),
                        "Expected '=' after identifier",
                    );
                }
            };

            // Collect the rest as the expression
            let expr: TokenStream = tokens.collect();

            if expr.clone().into_iter().next().is_none() {
                return generate_compile_error(Span::call_site(), "Expected expression after '='");
            }

            // Generate: pub static IDENT: TYPE = Lazy::new(|| EXPR);
            let mut output = TokenStream::new();

            // pub static
            output.extend(vec![
                TokenTree::Ident(Ident::new("pub", Span::call_site())),
                TokenTree::Ident(Ident::new("static", Span::call_site())),
            ]);

            // IDENT
            output.extend(vec![TokenTree::Ident(ident)]);

            // : TYPE
            output.extend(vec![
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new($type_name, Span::call_site())),
            ]);

            // = Lazy::new
            output.extend(vec![
                TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                TokenTree::Ident(Ident::new("Lazy", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new("new", Span::call_site())),
            ]);

            // (|| EXPR)
            let mut closure = TokenStream::new();
            closure.extend(vec![
                TokenTree::Punct(Punct::new('|', Spacing::Joint)),
                TokenTree::Punct(Punct::new('|', Spacing::Alone)),
            ]);
            closure.extend(expr);

            output.extend(vec![
                TokenTree::Group(Group::new(Delimiter::Parenthesis, closure)),
                TokenTree::Punct(Punct::new(';', Spacing::Alone)),
            ]);

            output
        }
    };
}

/// `LServer!` - Creates a lazy static Server instance.
///
/// # Usage
/// ```rust
/// LServer!(APP = Server::new().build());
/// ```
///
/// # Expansion
/// ```rust
/// pub static APP: SServer = Lazy::new(|| Server::new().build());
/// ```
#[allow(non_snake_case)]
#[proc_macro]
pub fn LServer(input: TokenStream) -> TokenStream {
    generate_lazy_static!("SServer")(input)
}

/// `LClient!` - Creates a lazy static Client instance.
///
/// # Usage
/// ```rust
/// LClient!(CLIENT = Client::new().build());
/// ```
///
/// # Expansion
/// ```rust
/// pub static CLIENT: SClient = Lazy::new(|| Client::new().build());
/// ```
#[allow(non_snake_case)]
#[proc_macro]
pub fn LClient(input: TokenStream) -> TokenStream {
    generate_lazy_static!("SClient")(input)
}

/// `LUrl!` - Creates a lazy static Url instance
///
/// # Usage
/// ```rust
/// LUrl!(HOME = Url::new("/"));
/// ```
///
/// # Expansion
/// ```rust
/// pub static HOME: SUrl<_> = Lazy::new(|| Url::new("/"));
/// ```
#[allow(non_snake_case)]
#[proc_macro]
pub fn LUrl(input: TokenStream) -> TokenStream {
    generate_lazy_static!("SUrl")(input)
}

/// `LPattern!` - Creates a lazy static PathPattern instance
///
/// # Usage
/// ```rust
/// LPattern!(PATTERN = PathPattern::new("/*"));
/// ```
///
/// # Expansion
/// ```rust
/// pub static PATTERN: SPattern = Lazy::new(|| PathPattern::new("/*"));
/// ```
#[allow(non_snake_case)]
#[proc_macro]
pub fn LPattern(input: TokenStream) -> TokenStream {
    generate_lazy_static!("SPattern")(input)
}
