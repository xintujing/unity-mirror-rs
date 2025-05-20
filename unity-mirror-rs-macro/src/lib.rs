mod namespace;
mod network_behaviour;
mod string_case;

use proc_macro::TokenStream;

mod metadata_settings;
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


// #[proc_macro_attribute]
// pub fn network_behaviour(attr: TokenStream, item: TokenStream) -> TokenStream {
//     let mut item_struct = parse_macro_input!(item as syn::ItemStruct);
//
//     match &mut item_struct.fields {
//         Fields::Named(fields_named) => fields_named.named.extend(Vec::<Field>::from(vec![
//             parse_quote! { #[metadata("syncDirection")] sync_direction: SyncDirection },
//             parse_quote! { #[metadata("syncMode")] sync_mode: SyncMode },
//             parse_quote! { #[metadata("syncInterval")] sync_interval: f32 },
//             parse_quote! { last_sync_time: f64 },
//             parse_quote! { net_id: u32 },
//             parse_quote! { network_identity: Reference<NetworkIdentity> },
//             parse_quote! { component_index: u8 },
//             parse_quote! { sync_var_dirty_bits: u64 },
//             parse_quote! { game_object: std::rc::Weak<std::cell::UnsafeCell<crate::unity_engine::game_object::GameObject>> },
//         ])),
//         _ => {
//             return syn::Error::new_spanned(
//                 item_struct.fields,
//                 "The network behaviour struct must have named fields",
//             )
//             .to_compile_error()
//             .into();
//         }
//     }
//     item_struct.attrs.push(parse_quote!(
//         #[derive(unity_mirror_macro::NetworkBehaviour)]
//     ));
//     TokenStream::from(quote! {
//         #item_struct
//     })
// }
//
// #[proc_macro_derive(NetworkBehaviour, attributes(sync_var, sync_object, metadata))]
// pub fn derive_network_behaviour(input: TokenStream) -> TokenStream {
//     let derive_input = parse_macro_input!(input as syn::DeriveInput);
//
//     let struct_ident = &derive_input.ident;
//
//     // let mut sync_var_field_idents = vec![];
//     let mut sync_var_hook_trait_fn_slots = vec![];
//     // let mut sync_obj_field_idents = vec![];
//     // let mut metadata_field_idents = vec![];
//
//     if let Data::Struct(data_struct) = &derive_input.data {
//         for field in data_struct.fields.iter() {
//             field.attrs.iter().for_each(|attr| {
//                 if attr.path().is_ident("sync_var") {
//                     // sync_var_field_idents.push(field.ident.clone().unwrap());
//                     let hook_fn_ident = format_ident!("{}_changed", field.ident.clone().unwrap());
//                     let field_type = &field.ty;
//                     sync_var_hook_trait_fn_slots.push(quote! {
//                         fn #hook_fn_ident(&mut self, old_value: &#field_type, new_value: &#field_type) {}
//                     })
//                 } else if attr.path().is_ident("sync_object") {
//                     // sync_obj_field_idents.push(field.ident.clone().unwrap());
//                 } else if attr.path().is_ident("metadata") {
//                     // metadata_field_idents.push(field.ident.clone().unwrap());
//                 }
//             });
//         }
//     }
//
//     let hooks_trait_ident = format_ident!("{}Hooks", struct_ident);
//
//     TokenStream::from(quote! {
//
//         trait #hooks_trait_ident{
//             #(
//                 #sync_var_hook_trait_fn_slots
//             )*
//         }
//     })
// }
//
// #[proc_macro_attribute]
// pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
//     let mut impl_item_fn = parse_macro_input!(item as syn::ImplItemFn);
//
//     if let Some(first_input_arg) = impl_item_fn.sig.inputs.first() {
//         match first_input_arg {
//             FnArg::Receiver(_) => {}
//             _ => {
//                 return syn::Error::new_spanned(
//                     first_input_arg,
//                     "The first parameter of the command function must be &self or &mut self",
//                 )
//                 .to_compile_error()
//                 .into();
//             }
//         }
//     }
//
//     TokenStream::from(quote! {})
// }

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
