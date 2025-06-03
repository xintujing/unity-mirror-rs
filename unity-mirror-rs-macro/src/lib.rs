#![allow(dead_code, unused)]

mod namespace;
mod network_behaviour;

use proc_macro::TokenStream;

mod callback_processor;
mod message;
mod metadata_settings;

mod network_manager;

mod network_manager_factory;

pub(crate) mod utils;

mod virtual_trait;

mod mirror;

macro_rules! attribute_args {
    ($type_name:ident, $($field_name:ident),+) => {
        #[derive(Default)]
        #[allow(unused)]
        struct $type_name {
            $($field_name: Option<String>,)*
        }


        impl syn::parse::Parse for $type_name {
            fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                use quote::ToTokens;

                let mut result= $type_name::default();

                while !input.is_empty() {
                    let name_value: syn::MetaNameValue = input.parse()?;
                    let key = name_value.path.to_token_stream().to_string();
                    let value = name_value.value.to_token_stream();

                    match key.as_str() {
                        $(stringify!($field_name) => {
                            result.$field_name = Some(value.to_string().trim_matches('"').to_string());
                        },)*
                        _ => {}
                    }

                    if input.peek(syn::Token![,]) {
                        input.parse::<syn::Token![,]>()?;
                    }
                }

                Ok(result)
            }
        }
    };
}

#[proc_macro_attribute]
pub fn network_behaviour(attr: TokenStream, item: TokenStream) -> TokenStream {
    network_behaviour::handler(attr, item)
}

#[proc_macro_attribute]
pub fn metadata(attr: TokenStream, item: TokenStream) -> TokenStream {
    TokenStream::new()
}

attribute_args!(NamespaceArgs, prefix, rename);
#[proc_macro_attribute]
pub fn namespace(attr: TokenStream, item: TokenStream) -> TokenStream {
    namespace::handler(attr, item)
}

#[proc_macro_derive(MetadataSettingsWrapper)]
pub fn derive_metadata_settings_wrapper(input: TokenStream) -> TokenStream {
    metadata_settings::wrapper::handler(input)
}

#[proc_macro]
pub fn settings_wrapper_register(input: TokenStream) -> TokenStream {
    metadata_settings::wrapper_register::handler(input)
}

#[proc_macro_derive(SyncState, attributes(sync_variable, sync_object))]
pub fn derive_sync_state(_: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_attribute]
pub fn ancestor_on_serialize(attr: TokenStream, item: TokenStream) -> TokenStream {
    network_behaviour::ancestor_on_serialize(attr, item)
}

#[proc_macro_attribute]
pub fn ancestor_on_deserialize(attr: TokenStream, item: TokenStream) -> TokenStream {
    network_behaviour::ancestor_on_deserialize(attr, item)
}

#[proc_macro_attribute]
pub fn parent_on_serialize(attr: TokenStream, item: TokenStream) -> TokenStream {
    network_behaviour::parent_on_serialize(attr, item)
}

#[proc_macro_attribute]
pub fn parent_on_deserialize(attr: TokenStream, item: TokenStream) -> TokenStream {
    network_behaviour::parent_on_deserialize(attr, item)
}

// #[proc_macro_derive(MessageRegistry)]
// pub fn message_registry(input: TokenStream) -> TokenStream {
//     message::message_registry_handler(input)
// }

#[proc_macro_derive(Message)]
pub fn message(input: TokenStream) -> TokenStream {
    message::message_handler(input)
}

#[proc_macro_derive(CallbackProcessor)]
pub fn callback_processor(input: TokenStream) -> TokenStream {
    callback_processor::callback_processor_handler(input)
}

#[proc_macro_attribute]
pub fn network_manager(attr: TokenStream, item: TokenStream) -> TokenStream {
    network_manager::handler(attr, item)
}

#[proc_macro_derive(NetworkManagerFactory)]
pub fn derive_network_manager_factory(item: TokenStream) -> TokenStream {
    network_manager_factory::handler(item)
}

#[proc_macro_attribute]
pub fn virtual_trait(attr: TokenStream, item: TokenStream) -> TokenStream {
    virtual_trait::handler(attr, item)
}
// #[proc_macro_attribute]
// pub fn virtual_trait(attr: TokenStream, item: TokenStream) -> TokenStream {
//     virtual_trait::handler(attr, item)
// }

#[proc_macro_derive(AuthenticatorFactory)]
pub fn derive_authenticator_factory(item: TokenStream) -> TokenStream {
    mirror::authenticator_factory::handler(item)
}

// #[proc_macro_attribute]
// pub fn rpc_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
//     mirror::component::rpc_impl::handler(attr, item)
// }

#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    mirror::component::command::handler(attr, item)
}

#[proc_macro_attribute]
pub fn client_rpc(attr: TokenStream, item: TokenStream) -> TokenStream {
    mirror::component::client_rpc::handler(attr, item)
}

#[proc_macro_attribute]
pub fn target_rpc(attr: TokenStream, item: TokenStream) -> TokenStream {
    mirror::component::target_rpc::handler(attr, item)
}
