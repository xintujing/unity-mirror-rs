use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Expr, FnArg, Path, Token, parse_macro_input};
use syn::punctuated::Punctuated;
use crate::utils::csharp::to_csharp_function_inputs;
use crate::utils::string_case::StringCase;

struct CommandArgs {
    struct_path: Path,
    authority: bool,
}

mod kw {
    syn::custom_keyword!(authority);
}

impl Parse for CommandArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let struct_path = match input.parse::<Path>() {
            Ok(struct_path) => struct_path,
            Err(_) => {
                return Err(syn::Error::new(
                    input.span(),
                    "Expected a struct path",
                ));
            }
        };
        let mut authority = false;

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::authority) {
                let _ = input.parse::<kw::authority>()?;
                let _: Token![=] = input.parse()?;
                let value: syn::LitBool = input.parse()?;
                authority = value.value();
            }
        }

        Ok(CommandArgs {
            struct_path,
            authority,
        })
    }
}

pub(crate) fn handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let CommandArgs {
        struct_path,
        authority,
    } = parse_macro_input!(attr as CommandArgs);

    let item_fn = parse_macro_input!(item as syn::ItemFn);

    let fn_ident = &item_fn.sig.ident;
    let fn_camel_name = fn_ident.to_string().to_camel_case();

    let invoke_user_code = format_ident!("__invoke_user_code_command_{}", fn_ident);

    let fn_inputs = item_fn
        .sig
        .inputs
        .iter()
        .filter(|input| match input {
            FnArg::Receiver(_) => false,
            _ => true,
        })
        .map(|_| {
            quote! {
                crate::mirror::network_reader::DataTypeDeserializer::deserialize(reader)
            }
        })
        .collect::<Vec<_>>();

    let csharp_func_inputs = to_csharp_function_inputs(item_fn.sig.inputs.clone());

    TokenStream::from(quote! {
        #item_fn

        fn #invoke_user_code(
            mut obj_chain: Vec<crate::commons::revel_weak::RevelWeak<Box<dyn crate::mirror::network_behaviour::NetworkBehaviourT>>>,
            reader: &mut crate::mirror::network_reader::NetworkReader,
            connection: crate::commons::revel_arc::RevelArc<crate::mirror::network_connection::NetworkConnection>,
        ) {
            obj_chain.reverse();

            for obj in obj_chain.iter() {
                if let Some(weak_this) = obj.downcast::<Self>() {
                    if let Some(real_this) = weak_this.get() {
                        real_this.#fn_ident(#(#fn_inputs,)*);
                        return;
                    }
                }
            }



            #[ctor::ctor]
            fn __static_init() {
                let fn_full_name= format!(
                    "System.Void {}::{}({})",
                    #struct_path::get_full_name(),
                    #fn_camel_name, #csharp_func_inputs,
                );
                crate::mirror::RemoteProcedureCalls.register_command::<#struct_path>(&fn_full_name, #struct_path::#invoke_user_code, true);
            }

            log::error!("Command {} invoke failed.", stringify!(#fn_camel_name));
        }

    })
}



