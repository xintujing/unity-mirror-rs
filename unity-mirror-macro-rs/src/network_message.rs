use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub(crate) fn handler(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_ident = &input.ident;

    let output = quote! {

        impl NetworkMessage for #struct_ident {

        }
    };
    output.into()
}
