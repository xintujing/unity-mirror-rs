#![allow(dead_code)]
use crate::utils::string_case::StringCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, GenericArgument, PathArguments, Type, TypePath, parse_macro_input};

pub(crate) fn callback_processor_handler(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;

    let static_callback_processor_ident = format_ident!(
        "{}_CALLBACK_PROCESSOR",
        struct_name.to_string().to_snake_case().to_uppercase()
    );

    let init_callback_processor_fn_ident = format_ident!(
        "init_{}_callback_processor",
        struct_name.to_string().to_snake_case()
    );

    let expanded = quote! {
        static mut #static_callback_processor_ident: once_cell::sync::Lazy<Option<crate::mirror::transport::CallbackProcessor>> = once_cell::sync::Lazy::new(|| None);

        fn #init_callback_processor_fn_ident(callback_processor: crate::mirror::transport::CallbackProcessor){
            unsafe {
                *#static_callback_processor_ident = Some(callback_processor);
            }
        }

        fn on_server_connected_with_address(connection_id: u64, address: &str) {
            unsafe{
                if let Some(ref callback_processor) = *#static_callback_processor_ident {
                    (callback_processor.on_server_connected_with_address)(connection_id, address);
                }
            }
        }
        fn on_server_connected(connection_id: u64) {
           unsafe{
                if let Some(ref callback_processor) = *#static_callback_processor_ident {
                    (callback_processor.on_server_connected)(connection_id);
                }
            }
        }
        fn on_server_data_received(connection_id: u64, data: &[u8], channel: TransportChannel) {
            unsafe{
                if let Some(ref callback_processor) = *#static_callback_processor_ident {
                    (callback_processor.on_server_data_received)(connection_id, data, channel);
                }
            }
        }
        fn on_server_error(connection_id: u64, error: TransportError, reason: &str) {
            unsafe{
                if let Some(ref callback_processor) = *#static_callback_processor_ident {
                    (callback_processor.on_server_error)(connection_id, error, reason);
                }
            }
        }
        fn on_server_data_sent(connection_id: u64, data: &[u8], channel: TransportChannel) {
            unsafe{
                if let Some(ref callback_processor) = *#static_callback_processor_ident {
                    (callback_processor.on_server_data_sent)(connection_id, data, channel);
                }
            }
        }
        fn on_server_transport_exception(connection_id: u64, error: Box<dyn std::error::Error>) {
            unsafe{
                if let Some(ref callback_processor) = *#static_callback_processor_ident {
                    (callback_processor.on_server_transport_exception)(connection_id, error);
                }
            }
        }
        fn on_server_disconnected(connection_id: u64) {
            unsafe{
                if let Some(ref callback_processor) = *#static_callback_processor_ident {
                    (callback_processor.on_server_disconnected)(connection_id);
                }
            }
        }
    };

    expanded.into()
}

fn is_option_callback_processor(ty: &Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        // 检查路径是否为 "Option"
        if path.segments.len() == 1 && path.segments[0].ident == "Option" {
            // 获取 Option 的泛型参数
            if let PathArguments::AngleBracketed(angle_bracketed) = &path.segments[0].arguments {
                if let Some(GenericArgument::Type(inner_ty)) = angle_bracketed.args.first() {
                    // 检查泛型参数是否为 CallbackProcessor
                    if let Type::Path(TypePath { path, .. }) = inner_ty {
                        return path.is_ident("CallbackProcessor");
                    }
                }
            }
        }
    }
    false
}
