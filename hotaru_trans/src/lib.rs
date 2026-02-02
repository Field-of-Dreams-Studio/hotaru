use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

// Procedural macros for Hotaru framework
// Entry points will be moved here from hotaru_trans
pub(crate) mod url; 
pub(crate) mod middleware; 
pub(crate) mod outpoint;

pub(crate) mod outer_attr; 
pub(crate) mod helper; 
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
            match url::parse_trans(input) { 
                Ok(url_args) => url_args.expand(),
                Err(err) => err, 
            }
        } 
    } else if #[cfg(feature = "attr")] {
        #[proc_macro_attribute] 
        pub fn endpoint(attr: TokenStream, input: TokenStream) -> TokenStream {
            match url::parse_attr(attr, input) { 
                Ok(url_args) => url_args.expand(),
                Err(err) => err, 
            }
        } 
    } else { 
        #[proc_macro_attribute] 
        pub fn endpoint(_attr: TokenStream, input: TokenStream) -> TokenStream {
            match url::parse_semi_trans(input) { 
                Ok(url_args) => url_args.expand(),
                Err(err) => err, 
            }
        }  
    }
} 

cfg_if::cfg_if! {
    if #[cfg(feature = "trans")] {
        #[proc_macro]
        pub fn outpoint(input: TokenStream) -> TokenStream {
            match outpoint::parse_trans(input) {
                Ok(args) => {
                    let mut tokens = TokenStream::new();
                    tokens.extend(args.op.generate_function());
                    tokens.extend(args.op.wrapper_function());
                    tokens.extend(args.reg_func());
                    tokens
                }
                Err(err) => err,
            }
        }
    } else if #[cfg(feature = "attr")] {
        #[proc_macro_attribute]
        pub fn outpoint(attr: TokenStream, input: TokenStream) -> TokenStream {
            match outpoint::parse_attr(attr, input) {
                Ok(args) => {
                    let mut tokens = TokenStream::new();
                    tokens.extend(args.op.generate_function());
                    tokens.extend(args.op.wrapper_function());
                    tokens.extend(args.reg_func());
                    tokens
                }
                Err(err) => err,
            }
        }
    } else {
        #[proc_macro_attribute]
        pub fn outpoint(_attr: TokenStream, input: TokenStream) -> TokenStream {
            match outpoint::parse_semi_trans(input) {
                Ok(args) => {
                    let mut tokens = TokenStream::new();
                    tokens.extend(args.op.generate_function());
                    tokens.extend(args.op.wrapper_function());
                    tokens.extend(args.reg_func());
                    tokens
                }
                Err(err) => err,
            }
        }
    }
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


/// Helper macro to generate lazy static declarations
/// Used by LApp!, LUrl!, and LPattern! macros
macro_rules! generate_lazy_static {
    ($type_name:expr) => {
        |input: TokenStream| -> TokenStream {
            let mut tokens = input.into_iter().peekable();

            // Parse identifier
            let ident = match tokens.next() {
                Some(TokenTree::Ident(i)) => i,
                _ => return generate_compile_error(Span::call_site(), "Expected identifier before '='"),
            };

            // Expect '='
            match tokens.next() {
                Some(TokenTree::Punct(p)) if p.as_char() == '=' => {},
                _ => return generate_compile_error(Span::call_site(), "Expected '=' after identifier"),
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

/// `LApp!` - Creates a lazy static App instance
///
/// # Usage
/// ```rust
/// LApp!(APP = App::new().build());
/// ```
///
/// # Expansion
/// ```rust
/// pub static APP: SApp = Lazy::new(|| App::new().build());
/// ```
#[allow(non_snake_case)]
#[proc_macro]
pub fn LApp(input: TokenStream) -> TokenStream {
    generate_lazy_static!("SApp")(input)
}

/// `LClient!` - Creates a lazy static Client instance
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
