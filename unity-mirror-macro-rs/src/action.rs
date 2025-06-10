use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, parse_quote, FnArg, Pat, Path};

struct ActionArgs {
    #[allow(unused)]
    action_path: Path,
}

impl Parse for ActionArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let action_path = input.parse()?;
        Ok(Self { action_path })
    }
}

pub(crate) fn handler(_: TokenStream, item: TokenStream) -> TokenStream {
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

                // match pat.as_ref() {
                //     Pat::Const(_) => {
                //         panic!("Pat::Const")
                //     }
                //     Pat::Ident(_) => {
                //         panic!("Pat::Ident")
                //     }
                //     Pat::Lit(_) => {
                //         panic!("Pat::Lit")
                //     }
                //     Pat::Macro(_) => {
                //         panic!("Pat::Macro")
                //     }
                //     Pat::Or(_) => {
                //         panic!("Pat::Or")
                //     }
                //     Pat::Paren(_) => {
                //         panic!("Pat::Paren")
                //     }
                //     Pat::Path(_) => {
                //         panic!("Pat::Path")
                //     }
                //     Pat::Range(_) => {
                //         panic!("Pat::Range")
                //     }
                //     Pat::Reference(_) => {
                //         panic!("Pat::Reference")
                //     }
                //     Pat::Rest(_) => {
                //         panic!("Pat::Rest")
                //     }
                //     Pat::Slice(_) => {
                //         panic!("Pat::Slice")
                //     }
                //     Pat::Struct(_) => {
                //         panic!("Pat::Struct")
                //     }
                //     Pat::Tuple(_) => {
                //         panic!("Pat::Tuple")
                //     }
                //     Pat::TupleStruct(_) => {
                //         panic!("Pat::TupleStruct")
                //     }
                //     Pat::Type(_) => {
                //         panic!("Pat::Type")
                //     }
                //     Pat::Verbatim(_) => {
                //         panic!("Pat::Verbatim")
                //     }
                //     Pat::Wild(_) => {
                //         panic!("Pat::Wild")
                //     }
                //     _ => {}
                // }

                if let Pat::Ident(pat_ident) = pat.as_ref() {
                    let pat_ident = &pat_ident.ident;
                    return quote! {
                      #pat_ident
                    };
                } else {
                    return quote! {
                      #pat
                    };
                }
                //
                // if pat_type
                //     .to_token_stream()
                //     .to_string()
                //     .contains("connection")
                // {
                //     panic!("{}", pat.to_token_stream());
                // }
                //
                // return quote! {
                //     #pat
                // };
            }
            quote! {}
            // parse_quote!(#)
        })
        .collect::<Vec<_>>();

    let output = &item_fn.sig.output;

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
