use crate::utils::csharp::to_csharp_function_inputs;
use crate::utils::string_case::StringCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Expr, FnArg, LitStr, Path, Token, parse_macro_input, parse_quote};

mod kw {
    syn::custom_keyword!(struct_path);
    syn::custom_keyword!(non_authority);
    syn::custom_keyword!(rename);
}

struct CommandArgs {
    struct_path: Path,
    authority: Option<Expr>,
    rename: Option<String>,
}

impl Parse for CommandArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let struct_path = match input.parse::<Path>() {
            Ok(struct_path) => struct_path,
            Err(_) => {
                return Err(syn::Error::new(input.span(), "Expected a struct path"));
            }
        };
        let mut authority = None;
        let mut rename = None;

        while !input.is_empty() {
            if input.peek(kw::struct_path) {
                input.parse::<kw::struct_path>()?;
            } else if input.peek(kw::non_authority) {
                let _ = input.parse::<kw::non_authority>()?;
                input.parse::<Token![=]>()?;
                authority = Some(input.parse()?);
            } else if input.peek(kw::rename) {
                let _ = input.parse::<kw::rename>()?;
                input.parse::<Token![=]>()?;
                let value: LitStr = input.parse()?;
                if value.value().is_empty() {
                    return Err(input.error("Rename argument cannot be empty"));
                }
                rename = Some(value.value());
            } else if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
                if input.is_empty() {
                    break;
                }
            } else {
                return Err(syn::Error::new(input.span(), "Expected a struct path"));
            }
        }

        Ok(CommandArgs {
            struct_path,
            authority,
            rename,
        })
    }
}

pub(crate) fn handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let CommandArgs {
        struct_path,
        mut authority,
        rename,
    } = parse_macro_input!(attr as CommandArgs);

    if authority.is_none() {
        authority = Some(parse_quote!{ true });
    }

    let item_fn = parse_macro_input!(item as syn::ItemFn);

    let fn_ident = &item_fn.sig.ident;

    let fn_name = match rename {
        None => item_fn.sig.ident.to_string().to_camel_case(),
        Some(rename) => rename,
    };

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
                DataTypeDeserializer::deserialize(reader)
            }
        })
        .collect::<Vec<_>>();

    let csharp_func_inputs = to_csharp_function_inputs(item_fn.sig.inputs.clone());

    TokenStream::from(quote! {
        #item_fn

        fn #invoke_user_code(
            mut obj_chain: Vec<RevelWeak<Box<dyn TNetworkBehaviour>>>,
            reader: &mut NetworkReader,
            connection: RevelArc<Box<NetworkConnectionToClient>>,
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

            log::error!("Command {} invoke failed.", #fn_name);

            #[ctor::ctor]
            fn __static_init() {
                let fn_full_name= format!(
                    "System.Void {}::{}({})",
                    #struct_path::get_full_name(),
                    #fn_name, #csharp_func_inputs,
                );
                RemoteProcedureCalls.register_command::<#struct_path>(&fn_full_name, #struct_path::#invoke_user_code, #authority);
            }
        }
    })
}
