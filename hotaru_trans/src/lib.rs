use proc_macro::TokenStream;

// Procedural macros for Hotaru framework
// Entry points will be moved here from hotaru_meta
pub(crate) mod url; 
pub(crate) mod middleware; 

pub(crate) mod helper; 
pub(crate) mod ctor; 

/// Our own constructor attribute - works like #[ctor::ctor] but built-in
/// Generates platform-specific linker sections for automatic initialization 
#[cfg(not(feature = "external-ctor"))]
#[proc_macro_attribute] 
pub fn ctor(_attr: TokenStream, item: TokenStream) -> TokenStream {
    ctor::ctor(_attr, item)
} 

