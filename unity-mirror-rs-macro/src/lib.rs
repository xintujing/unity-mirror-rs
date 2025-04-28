use proc_macro::TokenStream;

mod metadata_settings;
mod namespace;

mod component;
mod string_case;
mod synced;

#[allow(unused)]
macro_rules! attribute_args {
    ($type_name:ident, $($field_name:ident),+) => {
        #[derive(Default)]
        #[allow(unused)]
        struct $type_name {
            $($field_name: Option<String>,)*
        }


        impl Parse for $type_name {
            fn parse(input: ParseStream) -> syn::Result<Self> {



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

#[proc_macro_derive(MetadataSettingsWrapper)]
pub fn derive_metadata_settings_wrapper(input: TokenStream) -> TokenStream {
    metadata_settings::wrapper::handler(input)
}

#[proc_macro]
pub fn settings_wrapper_register(input: TokenStream) -> TokenStream {
    metadata_settings::wrapper_register::handler(input)
}


#[proc_macro_attribute]
pub fn namespace(attr: TokenStream, item: TokenStream) -> TokenStream {
    namespace::handler(attr, item)
}

// #[proc_macro_derive(MirrorComponent, attributes(parent))]
// pub fn mirror_component(item: TokenStream) -> TokenStream {
//     component::component::handler(item)
// }

#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    component::component2::handler(attr, item)
}

// #[proc_macro_derive(State, attributes(sync_variable, sync_object))]
// pub fn derive_state(item: TokenStream) -> TokenStream {
//     component::state::handler(item)
// }

#[proc_macro_attribute]
pub fn state(attr: TokenStream, item: TokenStream) -> TokenStream {
    component::state2::handler(attr, item)
}

#[proc_macro_attribute]
pub fn mirror_synced(attr: TokenStream, item: TokenStream) -> TokenStream {
    synced::handler(attr, item)
}

#[proc_macro_derive(InnerState, attributes(sync_variable, sync_object))]
pub fn derive_inner_state(_: TokenStream) -> TokenStream {
    TokenStream::new()
}

// #[proc_macro_attribute]
// pub fn sync_variable(_: TokenStream, item: TokenStream) -> TokenStream {
//     item
// }
// #[proc_macro_attribute]
// pub fn sync_object(_: TokenStream, item: TokenStream) -> TokenStream {
//     item
// }
