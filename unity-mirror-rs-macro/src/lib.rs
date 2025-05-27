mod namespace;
mod network_behaviour;

use proc_macro::TokenStream;

mod metadata_settings;
mod network_behaviour_state;

mod network_manager;

mod virtual_helper;

mod network_manager_factory;

pub(crate) mod utils;

mod callbacks;

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
pub fn derive_sync_state(input: TokenStream) -> TokenStream {
    network_behaviour_state::handler(input)
}

#[proc_macro_attribute]
pub fn network_manager(attr: TokenStream, item: TokenStream) -> TokenStream {
    network_manager::handler(attr, item)
}

#[proc_macro_attribute]
pub fn virtual_helper(attr: TokenStream, item: TokenStream) -> TokenStream {
    virtual_helper::handler(attr, item)
}

#[proc_macro_derive(NetworkManagerFactory)]
pub fn derive_network_manager_factory(item: TokenStream) -> TokenStream {
    network_manager_factory::handler(item)
}

#[proc_macro_attribute]
pub fn callbacks(attr: TokenStream, item: TokenStream) -> TokenStream {
    callbacks::handler(attr,item)
}
