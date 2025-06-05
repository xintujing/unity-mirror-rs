use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Path, parse_macro_input, FnArg, parse_quote};

struct ActionArgs {
    action_path: Path,
}

impl Parse for ActionArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let action_path = input.parse()?;
        Ok(Self { action_path })
    }
}

pub(crate) fn handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    // let ActionArgs { action_path } = parse_macro_input!(attr as ActionArgs);
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

    let output =  &item_fn.sig.output;

    let block = item_fn.block.clone();

    item_fn.block = parse_quote! {
        {
            if self.#fn_ident.is_registered() {
                self.#fn_ident.call((#(#inputs,)*))
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
