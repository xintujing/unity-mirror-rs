use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

pub(crate) fn handler(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_ident = &input.ident;

    let output = quote! {

        impl crate::mirror::messages::message::NetworkMessage for #struct_ident {

        }
    };
    output.into()
}
