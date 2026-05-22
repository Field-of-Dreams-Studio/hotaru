use proc_macro::TokenStream;

pub(crate) fn endpoint_trans(input: TokenStream) -> TokenStream {
    match super::parse::parse_trans(input) {
        Ok(url_args) => url_args.expand(),
        Err(err) => err,
    }
}

pub(crate) fn endpoint_attr(attr: TokenStream, input: TokenStream) -> TokenStream {
    match super::parse::parse_attr(attr, input) {
        Ok(url_args) => url_args.expand(),
        Err(err) => err,
    }
}

pub(crate) fn endpoint_semi_trans(input: TokenStream) -> TokenStream {
    match super::parse::parse_semi_trans(input) {
        Ok(url_args) => url_args.expand(),
        Err(err) => err,
    }
}
