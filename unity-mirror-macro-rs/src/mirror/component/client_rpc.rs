use crate::utils::csharp::to_csharp_function_inputs;
use crate::utils::string_case::StringCase;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Expr, FnArg, LitStr, Pat, PatType, Token, parse_macro_input, parse_quote};

mod kw {
    syn::custom_keyword!(channel);
    syn::custom_keyword!(include_owner);
    syn::custom_keyword!(rename);
}

struct TargetRpcArgs {
    channel: Option<Expr>,
    include_owner: Option<Expr>,
    rename: Option<String>,
}

impl Parse for TargetRpcArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut channel = None;
        let mut include_owner = None;
        let mut rename = None;

        while !input.is_empty() {
            if input.peek(kw::channel) {
                input.parse::<kw::channel>()?;
                input.parse::<Token![=]>()?;
                channel = Some(input.parse()?);
            } else if input.peek(kw::include_owner) {
                input.parse::<kw::include_owner>()?;
                input.parse::<Token![=]>()?;
                include_owner = Some(input.parse()?);
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

        Ok(TargetRpcArgs {
            channel,
            include_owner,
            rename,
        })
    }
}

pub(crate) fn handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let TargetRpcArgs {
        mut channel,
        mut include_owner,
        rename,
    } = parse_macro_input!(attr as TargetRpcArgs);

    if channel.is_none() {
        channel = Some(parse_quote! { TransportChannel::Reliable })
    }
    if include_owner.is_none() {
        include_owner = Some(parse_quote! { true })
    }

    let mut item_fn = parse_macro_input!(item as syn::ItemFn);

    let mut arg_block: Vec<proc_macro2::TokenStream> = vec![];

    for (_, fn_arg) in item_fn.sig.inputs.iter().enumerate() {
        if let FnArg::Typed(PatType { pat, .. }) = fn_arg {
            if let Pat::Ident(a) = pat.as_ref() {
                let arg_name = &a.ident;
                arg_block.push(quote! {
                    DataTypeSerializer::serialize(&#arg_name, &mut writer);
                });
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

                NetworkWriterPool::get_by_closure(|mut writer|{
                    #(#arg_block)*

                    let full_path_str = format!(
                        "System.Void {}::{}({})",
                        Self::get_full_name(),
                        #fn_name,
                        #csharp_func_inputs,
                    );

                    self.send_rpc_internal(
                        &full_path_str,
                        full_path_str.fn_hash() as u16,
                        &mut writer,
                        #channel,
                        #include_owner
                    );
                });
            }
        },
    );

    TokenStream::from(quote! {
        #item_fn
    })
}
