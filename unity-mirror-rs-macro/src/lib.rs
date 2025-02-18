extern crate proc_macro;
mod component;
mod m_sync;
mod network_message;
mod tools;

use crate::component::component_impl;
use crate::m_sync::m_sync_impl;
use crate::network_message::network_message_impl;
use proc_macro::TokenStream;
use quote::quote;
use std::time::SystemTime;
use syn::*;

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

#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    component_impl(attr, item)
}

/// 定义 command attribute 宏
#[proc_macro_attribute]
pub fn command(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// 定义 rpc attribute 宏
#[proc_macro_attribute]
pub fn rpc(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_derive(MSync, attributes(sync_var, sync_struct))]
pub fn m_sync(input: TokenStream) -> TokenStream {
    m_sync_impl(input)
}

#[proc_macro_derive(NetworkMessage)]
pub fn network_message(input: TokenStream) -> TokenStream {
    network_message_impl(input)
}
