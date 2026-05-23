use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree}; 

use hotaru_core::url::parser::parse as parse_check_url;

use crate::{generate_compile_error, into_peekable_iter};

pub struct UrlExpr {
    app: Ident,
    method: Ident,
    literal: Literal 
} 

impl UrlExpr {
    pub fn new(app: Ident, method: Ident, literal: Literal) -> Self {
        Self {
            app,
            method,
            literal,
        }
    }

    /// Accepts any of the following forms:
    /// APP_IDENTIFIER("path")
    /// APP_IDENTIFIER: "path"
    /// APP_IDENTIFIER.[url|lit_url]("path")
    /// "path" // Defaults to APP
    pub fn from_tokens(input: TokenStream) -> Result<Self, TokenStream> {
        let mut tokens = into_peekable_iter(input);
        match tokens.peek() {
            Some(TokenTree::Ident(app_ident)) => {
                let app = app_ident.clone();
                tokens.next(); // Consume app identifier 
                match tokens.peek() {
                    Some(TokenTree::Punct(punct)) if punct.as_char() == ':' => {
                        tokens.next(); // Consume ':' 
                        match tokens.next() {
                            Some(TokenTree::Literal(lit)) => {
                                Self::check_url_literal_format(&lit)?;
                                Ok(Self::new(app, Ident::new("url", Span::call_site()), lit))
                            }
                            _ => Err(generate_compile_error(
                                Span::call_site(),
                                "Expected a string literal after ':'",
                            )),
                        }
                    }
                    Some(TokenTree::Punct(punct)) if punct.as_char() == '.' => {
                        tokens.next(); // Consume '.' 
                        match tokens.next() {
                            Some(TokenTree::Ident(method_ident))
                                if method_ident.to_string() == "url"
                                    || method_ident.to_string() == "lit_url" =>
                            {
                                let method = method_ident.clone();
                                match tokens.next() {
                                    Some(TokenTree::Group(group))
                                        if group.delimiter() == Delimiter::Parenthesis =>
                                    {
                                        let mut inner_tokens = group.stream().into_iter();
                                        match inner_tokens.next() {
                                            Some(TokenTree::Literal(lit)) => {
                                                Self::check_url_literal_format(&lit)?;
                                                Ok(Self::new(app, method, lit))
                                            }
                                            _ => Err(generate_compile_error(
                                                Span::call_site(),
                                                "Expected a string literal inside the parentheses",
                                            )),
                                        }
                                    }
                                    _ => Err(generate_compile_error(
                                        Span::call_site(),
                                        "Expected parentheses after method identifier",
                                    )),
                                }
                            }
                            _ => Err(generate_compile_error(
                                Span::call_site(),
                                "Expected 'url' or 'lit_url' method identifier after '.'",
                            )),
                        }
                    }
                    _ => Err(generate_compile_error(
                        Span::call_site(),
                        "Expected ':' or '.' after application identifier",
                    )),
                }
            }
            Some(TokenTree::Literal(lit)) => {
                Self::check_url_literal_format(&lit)?;
                Ok(Self::new(
                    Ident::new("APP", Span::call_site()),
                    Ident::new("url", Span::call_site()),
                    lit.clone(),
                ))
            }
            _ => Err(generate_compile_error(
                Span::call_site(),
                "Expected an application identifier or a string literal for URL",
            )),
        }
    }

    fn check_url_literal_format(lit: &Literal) -> Result<(), TokenStream> {
        parse_check_url(&lit.to_string())
            .map_err(|e| {
                generate_compile_error(
                    Span::call_site(),
                    &format!("Invalid URL literal format: {}", e),
                )
            })
            .map(|_| ())
    }

    pub fn expand(&self, protocol: Ident, fn_name: Ident, binding: Ident, config: Ident) -> TokenStream {
        // APP.url::<HTTP, _, _>("/path", name, binding, params)
        //  .expect("failed to register endpoint");
        let mut tokens = TokenStream::new();
        tokens.extend(vec![
            TokenTree::Ident(self.app.clone()),
            TokenTree::Punct(Punct::new('.', Spacing::Alone)),
            TokenTree::Ident(self.method.clone()),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Punct(Punct::new('<', Spacing::Alone)),
            TokenTree::Ident(protocol),
            TokenTree::Punct(Punct::new(',', Spacing::Alone)),
            TokenTree::Ident(Ident::new("_", Span::call_site())),
            TokenTree::Punct(Punct::new(',', Spacing::Alone)),
            TokenTree::Ident(Ident::new("_", Span::call_site())),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, {
                let mut g = TokenStream::new();
                g.extend(vec![
                    TokenTree::Literal(self.literal.clone()), 
                    TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                    TokenTree::Literal(Literal::string(&fn_name.to_string())),  
                    TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                    TokenTree::Ident(binding),
                    TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                    TokenTree::Ident(config) 
                ]);
                g
            })),
            TokenTree::Punct(Punct::new('.', Spacing::Alone)),
            TokenTree::Ident(Ident::new("expect", Span::call_site())), 
            TokenTree::Group(Group::new(Delimiter::Parenthesis, {
                let mut g = TokenStream::new();
                g.extend(vec![
                    TokenTree::Literal(Literal::string("failed to register URL")),
                ]); 
                g 
            })), 
        ]);
        tokens
    }
}

