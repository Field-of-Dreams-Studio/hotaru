use proc_macro::{Ident, Literal, Punct, Spacing, TokenStream, TokenTree};

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

