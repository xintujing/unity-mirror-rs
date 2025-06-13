use crate::NamespaceArgs;
use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemStruct, parse_macro_input};
impl NamespaceArgs {
    pub(crate) fn get_full_name(&self, struct_ident: &syn::Ident) -> String {
        let prefix = match &self.prefix {
            None => String::new(),
            Some(value) => match value.chars().last().unwrap() {
                '.' | '+' | '\0' => value.clone(),
                _ => {
                    let mut value = value.clone();
                    value.push('.');
                    value
                }
            },
        };
        match &self.rename {
            None => {
                format!("{}{}", prefix, struct_ident)
            }
            Some(rename) => {
                format!("{}{}", prefix, rename)
            }
        }
    }
}

pub(crate) fn handler(attr: TokenStream, input: TokenStream) -> TokenStream {
    // 解析属性参数
    let namespace_args = parse_macro_input!(attr as NamespaceArgs);
    // 结构体
    let item_struct = parse_macro_input!(input as ItemStruct);
    // 结构体的标识符
    let struct_ident = &item_struct.ident;
    // 结构体的命名空间
    let full_name = namespace_args.get_full_name(struct_ident);

    quote! {
        #item_struct

        impl Object for #struct_ident {
            fn get_full_name() -> &'static str
            where
                Self: Sized,
            {
                #full_name
            }
        }
    }
    .into()
}
