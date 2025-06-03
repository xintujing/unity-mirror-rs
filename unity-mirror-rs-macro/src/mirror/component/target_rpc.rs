use crate::utils::csharp::to_csharp_function_inputs;
use crate::utils::string_case::StringCase;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, parse_quote, Expr, FnArg, LitStr, Pat, PatType, Token, Type};

mod kw {
    syn::custom_keyword!(channel);
    syn::custom_keyword!(rename);
}

struct TargetRpcArgs {
    channel: Option<Expr>,
    rename: Option<String>,
}

impl Parse for TargetRpcArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut channel = None;
        let mut rename = None;

        while !input.is_empty() {
            if input.peek(kw::channel) {
                input.parse::<kw::channel>()?;
                input.parse::<Token![=]>()?;
                channel = input.parse().ok();
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
                return Err(input.error("Unexpected argument"));
            }
        }

        Ok(TargetRpcArgs { channel, rename })
    }
}

pub(crate) fn handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let TargetRpcArgs {
        mut channel,
        rename,
    } = parse_macro_input!(attr as TargetRpcArgs);

    if channel.is_none() {
        channel = Some(parse_quote! {1})
    }

    let mut item_fn = parse_macro_input!(item as syn::ItemFn);

    let mut arg_block: Vec<proc_macro2::TokenStream> = vec![];

    let mut to = quote! {None};

    for (i, fn_arg) in item_fn.sig.inputs.iter().enumerate() {
        if let FnArg::Typed(PatType { pat, ty, .. }) = fn_arg {
            if let Pat::Ident(a) = pat.as_ref() {
                let arg_name = &a.ident;
                if let Type::Path(_) = &**ty {
                    if i == 1 {
                        to = quote! {Some(#arg_name)};
                    }
                }
                if i > 1 {
                    arg_block.push(quote! {
                        crate::mirror::network_writer::MethodParameterSerializer::serialize(#arg_name, &mut writer);
                    });
                }
            }
        }
    }

    let csharp_func_inputs = to_csharp_function_inputs(item_fn.sig.inputs.clone());

    let fn_name = match rename {
        None => item_fn.sig.ident.to_string().to_camel_case(),
        Some(rename) => rename,
    };

    item_fn.block.stmts.insert(
        0,
        syn::parse_quote! {
            {
                use crate::mirror::stable_hash::StableHash;
                use crate::mirror::network_writer::NetworkWriter;
                use crate::mirror::NetworkBehaviour;
                use crate::commons::object::Object;

                crate::mirror::network_writer_pool::NetworkWriterPool::get_return(|mut writer|{
                    #(#arg_block)*

                    let full_path_str = format!(
                        "System.Void {}::{}({})",
                        Self::get_full_name(),
                        #fn_name,
                        #csharp_func_inputs,
                    );

                    self.send_target_rpc_internal(
                        #to,
                        &full_path_str,
                        full_path_str.fn_hash() as u16,
                        &mut writer,
                        #channel,
                    );
                });
            }
        },
    );

    TokenStream::from(quote! {
        #item_fn
    })
}
