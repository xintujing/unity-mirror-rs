use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, FnArg};

pub(crate) fn handler(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut item_fn = parse_macro_input!(item as syn::ItemFn);

    let fn_ident = &item_fn.sig.ident;
    let default_fn_ident = format_ident!("{}_default", fn_ident);

    let src_inputs = &item_fn.sig.inputs;

    let inputs = &item_fn
        .sig
        .inputs
        .iter()
        .filter(|input| {
            if let FnArg::Receiver(_) = input {
                return false;
            }
            true
        })
        .map(|input| {
            if let FnArg::Typed(pat_type) = input {
                let pat = &pat_type.pat;
                return quote! {
                    #pat
                };
            }
            quote! {}
            // parse_quote!(#)
        })
        .collect::<Vec<_>>();

    let output = &item_fn.sig.output;

    let block = item_fn.block.clone();

    item_fn.block = parse_quote! {
        {
            if let Some(virtual_trait) = self.virtual_trait.get() {
                virtual_trait.#fn_ident(#(#inputs,)*)
            } else {
                self.#default_fn_ident(#(#inputs,)*)
            }
        }
    };

    TokenStream::from(quote! {
        #item_fn

        pub fn #default_fn_ident(#src_inputs) #output #block
    })
}
