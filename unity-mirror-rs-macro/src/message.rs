use crate::string_case::StringCase;
use crate::tools::generate_unique_string;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

pub(crate) fn message_registry_handler(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_ident = &input.ident;
    let register_ident = format_ident!(
        "__{}_register_{}",
        struct_ident.to_string().to_snake_case(),
        generate_unique_string(5).to_lowercase()
    );

    let output = quote! {

        impl crate::mirror::messages::message::Message for #struct_ident {

        }

        #[ctor::ctor]
        fn #register_ident() {
            crate::mirror::messages::message::register_messages::<#struct_ident>();
            use crate::commons::object::Object;
            use crate::mirror::stable_hash::StableHash;
            use colored::Colorize;
            log::info!("{} Registered for [{}] {} {}",
                "[Message]".bright_cyan().to_string(),
                #struct_ident::get_full_name().hash16(), stringify!(#struct_ident), #struct_ident::get_full_name());

        }
    };
    output.into()
}

pub(crate) fn message_handler(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_ident = &input.ident;

    let output = quote! {

        impl crate::mirror::messages::message::Message for #struct_ident {

        }
    };
    output.into()
}
