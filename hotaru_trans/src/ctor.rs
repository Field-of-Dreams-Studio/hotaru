use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

/// Generate constructor attribute for automatic registration
/// By default uses built-in #[hotaru::hotaru_trans::ctor]
/// Enable "external-ctor" feature to use #[ctor::ctor] from ctor crate instead
pub fn gen_ctor() -> TokenStream {
    #[cfg(feature = "external-ctor")]
    {
        // Use external ctor crate: #[ctor::ctor]
        let mut ctor_macro = TokenStream::new();
        ctor_macro.extend(vec![
            TokenTree::Punct(Punct::new('#', Spacing::Alone)),
            TokenTree::Group(Group::new(Delimiter::Bracket, {
                let mut attr = TokenStream::new();
                attr.extend(vec![
                    TokenTree::Ident(Ident::new("ctor", Span::call_site())),
                    TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                    TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                    TokenTree::Ident(Ident::new("ctor", Span::call_site())),
                ]);
                attr
            })),
        ]);
        ctor_macro
    }

    #[cfg(not(feature = "external-ctor"))]
    {
        // Use built-in hotaru::::ctor
        let mut ctor_macro = TokenStream::new();
        ctor_macro.extend(vec![
            TokenTree::Punct(Punct::new('#', Spacing::Alone)),
            TokenTree::Group(Group::new(Delimiter::Bracket, {
                let mut attr = TokenStream::new();
                attr.extend(vec![
                    TokenTree::Ident(Ident::new("hotaru", Span::call_site())),
                    TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                    TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                    TokenTree::Ident(Ident::new("hrt", Span::call_site())),
                    TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                    TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                    TokenTree::Ident(Ident::new("ctor", Span::call_site())),
                ]);
                attr
            })),
        ]);
        ctor_macro
    }
}

pub fn ctor(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input function
    let mut tokens = item.into_iter().peekable();

    // Collect the entire function
    let function_tokens: Vec<_> = tokens.collect();

    // Extract function name for generating unique static name
    let fn_name = function_tokens.iter()
        .skip_while(|t| !matches!(t, TokenTree::Ident(id) if id.to_string() == "fn"))
        .nth(1)
        .and_then(|t| if let TokenTree::Ident(id) = t { Some(id.to_string()) } else { None })
        .unwrap_or_else(|| "unknown".to_string());

    let mut output = TokenStream::new();

    // Re-emit the original function
    output.extend(function_tokens);

    // Generate platform-specific static variable with linker sections
    let mut static_decl = TokenStream::new();

    // #[allow(unsafe_code)] - suppress unsafe warnings like ctor does
    static_decl.extend(vec![
        TokenTree::Punct(Punct::new('#', Spacing::Alone)),
        TokenTree::Group(Group::new(Delimiter::Bracket, {
            let mut attr = TokenStream::new();
            attr.extend(vec![
                TokenTree::Ident(Ident::new("allow", Span::call_site())),
                TokenTree::Group(Group::new(Delimiter::Parenthesis, {
                    let mut inner = TokenStream::new();
                    inner.extend(vec![TokenTree::Ident(Ident::new("unsafe_code", Span::call_site()))]);
                    inner
                })),
            ]);
            attr
        })),
    ]);

    // #[used]
    static_decl.extend(vec![
        TokenTree::Punct(Punct::new('#', Spacing::Alone)),
        TokenTree::Group(Group::new(Delimiter::Bracket, {
            let mut attr = TokenStream::new();
            attr.extend(vec![TokenTree::Ident(Ident::new("used", Span::call_site()))]);
            attr
        })),
    ]);

    // #[cfg_attr(target_os = "linux", unsafe(link_section = ".init_array"))]
    static_decl.extend(vec![
        TokenTree::Punct(Punct::new('#', Spacing::Alone)),
        TokenTree::Group(Group::new(Delimiter::Bracket, {
            let mut attr = TokenStream::new();
            attr.extend(vec![
                TokenTree::Ident(Ident::new("cfg_attr", Span::call_site())),
                TokenTree::Group(Group::new(Delimiter::Parenthesis, {
                    let mut inner = TokenStream::new();
                    inner.extend(vec![
                        TokenTree::Ident(Ident::new("target_os", Span::call_site())),
                        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                        TokenTree::Literal(Literal::string("linux")),
                        TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                        TokenTree::Ident(Ident::new("unsafe", Span::call_site())),
                        TokenTree::Group(Group::new(Delimiter::Parenthesis, {
                            let mut unsafe_inner = TokenStream::new();
                            unsafe_inner.extend(vec![
                                TokenTree::Ident(Ident::new("link_section", Span::call_site())),
                                TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                                TokenTree::Literal(Literal::string(".init_array")),
                            ]);
                            unsafe_inner
                        })),
                    ]);
                    inner
                })),
            ]);
            attr
        })),
    ]);

    // #[cfg_attr(target_vendor = "apple", unsafe(link_section = "__DATA,__mod_init_func"))]
    static_decl.extend(vec![
        TokenTree::Punct(Punct::new('#', Spacing::Alone)),
        TokenTree::Group(Group::new(Delimiter::Bracket, {
            let mut attr = TokenStream::new();
            attr.extend(vec![
                TokenTree::Ident(Ident::new("cfg_attr", Span::call_site())),
                TokenTree::Group(Group::new(Delimiter::Parenthesis, {
                    let mut inner = TokenStream::new();
                    inner.extend(vec![
                        TokenTree::Ident(Ident::new("target_vendor", Span::call_site())),
                        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                        TokenTree::Literal(Literal::string("apple")),
                        TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                        TokenTree::Ident(Ident::new("unsafe", Span::call_site())),
                        TokenTree::Group(Group::new(Delimiter::Parenthesis, {
                            let mut unsafe_inner = TokenStream::new();
                            unsafe_inner.extend(vec![
                                TokenTree::Ident(Ident::new("link_section", Span::call_site())),
                                TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                                TokenTree::Literal(Literal::string("__DATA,__mod_init_func")),
                            ]);
                            unsafe_inner
                        })),
                    ]);
                    inner
                })),
            ]);
            attr
        })),
    ]);

    // #[cfg_attr(target_os = "windows", unsafe(link_section = ".CRT$XCU"))]
    static_decl.extend(vec![
        TokenTree::Punct(Punct::new('#', Spacing::Alone)),
        TokenTree::Group(Group::new(Delimiter::Bracket, {
            let mut attr = TokenStream::new();
            attr.extend(vec![
                TokenTree::Ident(Ident::new("cfg_attr", Span::call_site())),
                TokenTree::Group(Group::new(Delimiter::Parenthesis, {
                    let mut inner = TokenStream::new();
                    inner.extend(vec![
                        TokenTree::Ident(Ident::new("target_os", Span::call_site())),
                        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                        TokenTree::Literal(Literal::string("windows")),
                        TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                        TokenTree::Ident(Ident::new("unsafe", Span::call_site())),
                        TokenTree::Group(Group::new(Delimiter::Parenthesis, {
                            let mut unsafe_inner = TokenStream::new();
                            unsafe_inner.extend(vec![
                                TokenTree::Ident(Ident::new("link_section", Span::call_site())),
                                TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                                TokenTree::Literal(Literal::string(".CRT$XCU")),
                            ]);
                            unsafe_inner
                        })),
                    ]);
                    inner
                })),
            ]);
            attr
        })),
    ]);

    // static __CTOR_<name>: extern "C" fn() = { extern "C" fn __wrapper() { <fn_name>() }; __wrapper };
    static_decl.extend(vec![
        TokenTree::Ident(Ident::new("static", Span::call_site())),
        TokenTree::Ident(Ident::new(&format!("__CTOR_{}", fn_name.to_uppercase()), Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("extern", Span::call_site())),
        TokenTree::Literal(Literal::string("C")),
        TokenTree::Ident(Ident::new("fn", Span::call_site())),
        TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        TokenTree::Group(Group::new(Delimiter::Brace, {
            let mut inner = TokenStream::new();
            inner.extend(vec![
                TokenTree::Ident(Ident::new("extern", Span::call_site())),
                TokenTree::Literal(Literal::string("C")),
                TokenTree::Ident(Ident::new("fn", Span::call_site())),
                TokenTree::Ident(Ident::new(&format!("__wrapper_{}", fn_name), Span::call_site())),
                TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
                TokenTree::Group(Group::new(Delimiter::Brace, {
                    let mut call = TokenStream::new();
                    call.extend(vec![
                        TokenTree::Ident(Ident::new(&fn_name, Span::call_site())),
                        TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
                    ]);
                    call
                })),
                TokenTree::Punct(Punct::new(';', Spacing::Alone)),
                TokenTree::Ident(Ident::new(&format!("__wrapper_{}", fn_name), Span::call_site())),
            ]);
            inner
        })),
        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
    ]);

    output.extend(static_decl);
    output
} 