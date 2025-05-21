use proc_macro::TokenStream;
use syn::parse::Parse;
use syn::parse_macro_input;

pub(crate) fn handler(input: TokenStream) -> TokenStream {
    // DeriveInput
    let mut derive_input = parse_macro_input!(input as syn::DeriveInput);

    // struct 名
    let struct_ident = &derive_input.ident;


    TokenStream::new()
}
