extern crate proc_macro;
use crate::component::component_attribute_handler;
use crate::namespace::namespace_attribute_handler;
mod component;
mod m_sync;
mod network_message;
mod tools;

use crate::component::component_impl;
use crate::m_sync::m_sync_impl;
use crate::network_message::network_message_impl;
use proc_macro::TokenStream;
use quote::quote;
use quote::{quote, ToTokens};
use std::time::SystemTime;
use syn::parse::{Parse, ParseStream};
use syn::*;

mod utils;

mod command;
mod component;
mod namespace;
mod rpc;

macro_rules! attribute_args {
    ($type_name:ident, $($field_name:ident),+) => {
        #[derive(Default)]
        #[allow(unused)]
        struct $type_name {
            $($field_name: Option<String>,)*
        }

        impl Parse for $type_name {
            fn parse(input: ParseStream) -> Result<Self> {
                $(let mut $field_name: String = "".to_string();)*

                let mut result= $type_name::default();

                while !input.is_empty() {
                    let name_value: MetaNameValue = input.parse()?;
                    let key = name_value.path.to_token_stream().to_string();
                    let value = name_value.value.to_token_stream();

                    match key.as_str() {
                        $(stringify!($field_name) => {
                            result.$field_name = Some(value.to_string().trim_matches('"').to_string());
                        },)*
                        _ => {}
                    }

                    if input.peek(Token![,]) {
                        input.parse::<Token![,]>()?;
                    }
                }

                Ok(result)
            }
        }
    };
}

#[proc_macro_attribute]
pub fn mirror(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // 解析输入的 TokenStream 到 ItemImpl 结构
    let mut input = parse_macro_input!(item as ItemFn);

    // 只能在main方法上使用
    if input.sig.ident != "main" {
        panic!("Only main method can use mirror attribute");
    }

    // 在main方法上添加一个方法，用于注册命令
    input.block.stmts.insert(
        0,
        parse_quote! {
            unsafe {
                for register_function in REGISTER_FUNCTIONS.iter() {
                    register_function()
                }
            }
        },
    );

    let stream = TokenStream::from(quote! {

        pub static mut REGISTER_FUNCTIONS: Vec<fn()> = vec![];

        #input

    });

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    std::fs::write(format!("{}_main.rs", timestamp), stream.to_string())
        .expect("write file failed");

    stream
}

attribute_args!(ComponentArgs, namespace);
#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    component_attribute_handler(attr, item)
}

/// 定义 command attribute 宏
attribute_args!(CommandArgs, requires_authority);
#[proc_macro_attribute]
pub fn command(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// 定义 rpc attribute 宏
#[proc_macro_attribute]
pub fn rpc(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

attribute_args!(NamespaceArgs, value, full_path);
#[proc_macro_attribute]
pub fn namespace(attr: TokenStream, item: TokenStream) -> TokenStream {
    namespace_attribute_handler(attr, item)
}
#[proc_macro_derive(MSync, attributes(sync_var, sync_struct))]
pub fn m_sync(input: TokenStream) -> TokenStream {
    m_sync_impl(input)
}

#[proc_macro_derive(NetworkMessage)]
pub fn network_message(input: TokenStream) -> TokenStream {
    network_message_impl(input)
}
