use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, parse_quote, Fields, FnArg, Token};

struct CallbacksArgs {
    virtual_fns: Punctuated<CallbackSignature, Token![;]>,
}

struct CallbackSignature {
    ident: Ident,
    inputs: Punctuated<FnArg, Token![,]>,
}

impl Parse for CallbacksArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // 解析内部的内容：{ fn foo(&self); fn bar(&self); }
        let content;
        syn::braced!(content in input);

        let mut virtual_fns = Punctuated::new();

        while !content.is_empty() {
            // // 解析 fn
            // content.parse::<Token![fn]>()?;

            // 解析函数名
            let ident: Ident = content.parse()?;

            // 解析参数列表 (...)
            let paren;
            syn::parenthesized!(paren in content);
            let inputs: Punctuated<FnArg, Token![,]> =
                paren.parse_terminated(FnArg::parse, Token![,])?;

            // 忽略返回值部分

            // 解析分号
            // content.parse::<Semi>()?;

            virtual_fns.push_value(CallbackSignature { ident, inputs });
            virtual_fns.push_punct(content.parse()?); // 分号
        }

        Ok(CallbacksArgs { virtual_fns })
    }
}

// 可选：实现 ToTokens 来生成代码
impl ToTokens for CallbacksArgs {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for callback in &self.virtual_fns {
            let ident = &callback.ident;
            let inputs = &callback.inputs;
            tokens.extend(
                quote! {
                    fn #ident(#inputs);
                }
                .to_token_stream(),
            );
        }
    }
}

pub(crate) fn handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let callbacks_args = parse_macro_input!(attr as CallbacksArgs);

    // panic!("{}", callbacks_args.to_token_stream());

    let mut item_struct = parse_macro_input!(item as syn::ItemStruct);

    let struct_ident = &item_struct.ident;

    let callbacks_trait_ident = format_ident!("{}Callbacks", struct_ident);

    if let Fields::Named(fields_named) = &mut item_struct.fields {
        fields_named.named.push(parse_quote! {
            callbacks: crate::commons::revel_weak::RevelWeak<Box<dyn #callbacks_trait_ident>>
        })
    }

    // let a = quote! {pub trait #callbacks_trait_ident: crate::mirror::network_manager_trait::NetworkManager {
    //     #callbacks_args
    // }};
    //
    // write_to_file("qqQ", a.to_string());

    TokenStream::from(quote! {
        #item_struct

        pub trait #callbacks_trait_ident: crate::mirror::network_manager_trait::NetworkManager {
            #callbacks_args
        }

        impl #struct_ident {
            pub fn set_callbacks(&mut self, callbacks: crate::commons::revel_weak::RevelWeak<Box<dyn #callbacks_trait_ident>>) {
                self.callbacks = callbacks;
            }
        }
    })
}
