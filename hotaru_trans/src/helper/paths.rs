use proc_macro::{Ident, Punct, Spacing, Span, TokenStream, TokenTree};

pub fn use_core<A: AsRef<str>>(path: &[A]) -> TokenStream {
    let mut ts = TokenStream::new();
    ts.extend([TokenTree::Punct(Punct::new(':', Spacing::Joint))]);
    ts.extend([TokenTree::Punct(Punct::new(':', Spacing::Alone))]);
    #[cfg(feature = "facade")]
    {
        ts.extend([TokenTree::Ident(Ident::new("hotaru", Span::call_site()))]);
        ts.extend([TokenTree::Punct(Punct::new(':', Spacing::Joint))]);
        ts.extend([TokenTree::Punct(Punct::new(':', Spacing::Alone))]);
    }
    ts.extend([TokenTree::Ident(Ident::new(
        "hotaru_core",
        Span::call_site(),
    ))]);
    ts.extend([TokenTree::Punct(Punct::new(':', Spacing::Joint))]);
    ts.extend([TokenTree::Punct(Punct::new(':', Spacing::Alone))]);
    for (index, segment) in path.iter().enumerate() {
        ts.extend([TokenTree::Ident(Ident::new(
            &segment.as_ref(),
            Span::call_site(),
        ))]);
        if index + 1 < path.len() {
            ts.extend([TokenTree::Punct(Punct::new(':', Spacing::Joint))]);
            ts.extend([TokenTree::Punct(Punct::new(':', Spacing::Alone))]);
        }
    }
    ts
}
