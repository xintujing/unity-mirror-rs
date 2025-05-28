use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Fields};

pub(crate) fn handler(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(item as syn::ItemStruct);

    if let Fields::Named(fields_named) = &mut item_struct.fields {
        fields_named.named.push(parse_quote!(
            __virtual_helper:
        ))
    }

    TokenStream::from(quote! {
        #item_struct
    })
}
