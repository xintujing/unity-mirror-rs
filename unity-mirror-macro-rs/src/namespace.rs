use crate::NamespaceArgs;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Item};
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
    // 解析输入的结构体或枚举
    let item = parse_macro_input!(input as Item);
    match item {
        // 如果是结构体
        Item::Struct(item_struct) => {
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
        // 如果是枚举
        Item::Enum(item_enum) => {
            // 枚举的标识符
            let enum_ident = &item_enum.ident;
            // 枚举的命名空间
            let full_name = namespace_args.get_full_name(enum_ident);
            quote! {
                #item_enum
                impl Object for #enum_ident {
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
        // 对其他类型报错
        _ => {
            let error = "This macro only supports structs and enums.".to_string();
            syn::Error::new_spanned(item, error).to_compile_error().into()
        }
    }
}
