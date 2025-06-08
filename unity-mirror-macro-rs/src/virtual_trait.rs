use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{ToTokens, format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Fields, FnArg, Token, parse_macro_input, parse_quote};

struct VirtualTraitArgs {
    virtual_fns: Punctuated<VirtualTraitArgsSignature, Token![;]>,
}

struct VirtualTraitArgsSignature {
    ident: Ident,
    inputs: Punctuated<FnArg, Token![,]>,
    output: Option<syn::ReturnType>,
}

impl Parse for VirtualTraitArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // 解析内部的内容：{ fn foo(&self); fn bar(&self); }
        let content;
        syn::braced!(content in input);

        let mut virtual_fns = Punctuated::new();

        while !content.is_empty() {
            // 解析函数名
            let ident: Ident = content.parse()?;

            // 解析参数列表 (...)
            let paren;
            syn::parenthesized!(paren in content);
            let inputs: Punctuated<FnArg, Token![,]> =
                paren.parse_terminated(FnArg::parse, Token![,])?;

            let output = if content.peek(Token![->]) {
                Some(content.parse()?)
            } else {
                None
            };

            virtual_fns.push_value(VirtualTraitArgsSignature {
                ident,
                inputs,
                output,
            });
            virtual_fns.push_punct(content.parse()?); // 分号
        }

        Ok(Self { virtual_fns })
    }
}

// 可选：实现 ToTokens 来生成代码
impl ToTokens for VirtualTraitArgs {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for callback in &self.virtual_fns {
            let ident = &callback.ident;
            let inputs = &callback.inputs;
            let output = &callback.output;
            tokens.extend(
                quote! {
                    fn #ident(#inputs)#output;
                }
                .to_token_stream(),
            );
        }
    }
}

pub(crate) fn handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let virtual_trait_args = parse_macro_input!(attr as VirtualTraitArgs);

    let mut item_struct = parse_macro_input!(item as syn::ItemStruct);

    let struct_ident = &item_struct.ident;

    let virtual_trait_trait_ident = format_ident!("{}VirtualTrait", struct_ident);

    if let Fields::Named(fields_named) = &mut item_struct.fields {
        fields_named.named.push(parse_quote! {
            virtual_trait: crate::commons::revel_weak::RevelWeak<Box<dyn #virtual_trait_trait_ident>>
        })
    }

    TokenStream::from(quote! {
        #item_struct

        pub trait #virtual_trait_trait_ident {
            #virtual_trait_args
        }

        impl #struct_ident {
            pub fn set_virtual_trait(&mut self, virtual_trait: crate::commons::revel_weak::RevelWeak<Box<dyn #virtual_trait_trait_ident>>) {
                self.virtual_trait = virtual_trait;
            }
        }
    })
}
