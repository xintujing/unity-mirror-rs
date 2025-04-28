use crate::string_case::StringCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, GenericArgument, PathArguments};

pub(crate) fn handler(item: TokenStream) -> TokenStream {
    let mut derive_input = parse_macro_input!(item as syn::DeriveInput);

    let struct_ident = &derive_input.ident;

    let mut sync_variable_index = 0;
    let mut sync_object_index = 0;

    let mut sync_variable_slots = vec![];
    let mut sync_variable_field_idents = vec![];
    let mut sync_object_slots = vec![];
    let mut sync_object_field_idents = vec![];

    if let Data::Struct(data_struct) = &derive_input.data {
        for field in data_struct.fields.iter() {
            for field_attr in field.attrs.iter() {
                if let syn::Type::Path(type_path) = &field.ty {
                    if type_path.path.segments.len() > 0 {
                        let segment = &type_path.path.segments[0];

                        let mut general_type_path = None;
                        if let PathArguments::AngleBracketed(angle_bracketed) = &segment.arguments {
                            if let Some(generic_argument) = angle_bracketed.args.first() {
                                if let GenericArgument::Type(generic_argument_type) =
                                    &generic_argument
                                {
                                    general_type_path = Some(generic_argument_type.clone());
                                }
                            }
                        }

                        let field_type_ident = &segment.ident;
                        // 获取字段名称
                        let field_name = &field.ident.clone().unwrap();
                        let sync_field_get_ident = format_ident!("get_{}", field_name);
                        let sync_field_set_ident = format_ident!("get_{}", field_name);

                        if field_attr.meta.path().is_ident("sync_variable") {

                            sync_variable_slots.push(quote! {
                                fn sync_field_get_ident(){}
                                fn sync_field_set_ident(){}
                            });

                            // sync_variable_slots.push(quote! {
                            //     fn #field_name_new_ident(default_value: #general_type_path) -> #field_type_ident<#general_type_path> {
                            //         #field_type_ident::new(default_value, #sync_variable_index as u8)
                            //     }
                            // });

                            sync_variable_field_idents.push(field_name.clone());
                            sync_variable_index += 1;
                        }

                        if field_attr.meta.path().is_ident("sync_object") {
                            // sync_object_slots.push(quote! {
                            //     fn #field_name_new_ident(default_value: #general_type_path) -> #field_type_ident<#general_type_path> {
                            //         #field_type_ident::new(default_value, #sync_object_index as u8)
                            //     }
                            // });
                            sync_object_slots.push(quote! {});

                            sync_object_field_idents.push(field_name.clone());
                            sync_object_index += 1;
                        }
                    }
                }
            }

            // // 判断字段类型是否为 SyncVariable<X>
            // if let syn::Type::Path(type_path) = &field.ty {
            //     if type_path.path.segments.len() > 0 {
            //         let segment = &type_path.path.segments[0];
            //
            //         let mut general_type_path = None;
            //         if let PathArguments::AngleBracketed(angle_bracketed) = &segment.arguments {
            //             if let Some(generic_argument) = angle_bracketed.args.first() {
            //                 if let GenericArgument::Type(generic_argument_type) = &generic_argument
            //                 {
            //                     if let Type::Path(type_path) = generic_argument_type {
            //                         general_type_path = Some(generic_argument_type.clone());
            //                     }
            //                 }
            //             }
            //         }
            //
            //         let field_type_ident = &segment.ident;
            //         // 获取字段名称
            //         let field_name = &field.ident.clone().unwrap();
            //         let field_name_new_ident = format_ident!("{}_new", field_name);
            //         match segment.ident.to_string().as_str() {
            //             "SyncVariable" => {
            //                 sync_variable_slots.push(quote! {
            //                     fn #field_name_new_ident(default_value: #general_type_path) -> #field_type_ident<#general_type_path> {
            //                         #field_type_ident::new(default_value, #sync_variable_index as u8)
            //                     }
            //                 });
            //
            //                 sync_variable_field_idents.push(field_name.clone());
            //
            //                 sync_variable_index += 1;
            //             }
            //             "SyncObject" => {
            //                 sync_object_slots.push(quote! {
            //                     fn #field_name_new_ident(default_value: #general_type_path) -> #field_type_ident<#general_type_path> {
            //                         #field_type_ident::new(default_value, #sync_variable_index as u8)
            //                     }
            //                 });
            //
            //                 sync_object_field_idents.push(field_name.clone());
            //
            //                 sync_object_index += 1;
            //             }
            //
            //             &_ => {}
            //         }
            //     }
            // }
        }
    }

    // if let Data::Struct(data_struct) = &derive_input.data {
    //     for field in data_struct.fields.iter() {
    //         // 判断字段类型是否为 SyncVariable<X>
    //         if let syn::Type::Path(type_path) = &field.ty {
    //             if type_path.path.segments.len() > 0 {
    //                 let segment = &type_path.path.segments[0];
    //
    //                 let mut general_type_path = None;
    //                 if let PathArguments::AngleBracketed(angle_bracketed) = &segment.arguments {
    //                     if let Some(generic_argument) = angle_bracketed.args.first() {
    //                         if let GenericArgument::Type(generic_argument_type) = &generic_argument
    //                         {
    //                             if let Type::Path(type_path) = generic_argument_type {
    //                                 general_type_path = Some(generic_argument_type.clone());
    //                             }
    //                         }
    //                     }
    //                 }
    //
    //                 let field_type_ident = &segment.ident;
    //                 // 获取字段名称
    //                 let field_name = &field.ident.clone().unwrap();
    //                 let field_name_new_ident = format_ident!("{}_new", field_name);
    //                 match segment.ident.to_string().as_str() {
    //                     "SyncVariable" => {
    //                         sync_variable_slots.push(quote! {
    //                             fn #field_name_new_ident(default_value: #general_type_path) -> #field_type_ident<#general_type_path> {
    //                                 #field_type_ident::new(default_value, #sync_variable_index as u8)
    //                             }
    //                         });
    //
    //                         sync_variable_field_idents.push(field_name.clone());
    //
    //                         sync_variable_index += 1;
    //                     }
    //                     "SyncObject" => {
    //                         sync_object_slots.push(quote! {
    //                             fn #field_name_new_ident(default_value: #general_type_path) -> #field_type_ident<#general_type_path> {
    //                                 #field_type_ident::new(default_value, #sync_variable_index as u8)
    //                             }
    //                         });
    //
    //                         sync_object_field_idents.push(field_name.clone());
    //
    //                         sync_object_index += 1;
    //                     }
    //
    //                     &_ => {}
    //                 }
    //             }
    //         }
    //     }
    // }

    let static_state_ident = format_ident!(
        "{}",
        struct_ident.to_string().to_snake_case().to_uppercase()
    );

    TokenStream::from(quote::quote! {

            static mut #static_state_ident: once_cell::sync::Lazy<
                std::collections::HashMap<String, std::sync::Arc<std::sync::RwLock<#struct_ident>>>
            > = once_cell::sync::Lazy::new(|| std::collections::HashMap::new());

            impl #struct_ident {

                fn new(id: &str,state: #struct_ident) {
                    #[allow(static_mut_refs)]
                    unsafe {
                        #static_state_ident.insert(id.to_string(),std::sync::Arc::new(std::sync::RwLock::new(state)));
                    }
                }

                fn get(id: &str) -> Option<std::sync::RwLockReadGuard<#struct_ident>> {
                    #[allow(static_mut_refs)]
                    unsafe {
                        if let Some(state) = #static_state_ident.get(id) {
                            return state
                                .try_read()
                                .map_err(|err| crate::commons::trace::trace(5, err.into())).ok();
                        }
                    }
                    None
                }

                fn get_mut(
                    id: &str,
                ) -> Option<std::sync::RwLockWriteGuard<#struct_ident>> {
                    #[allow(static_mut_refs)]
                    unsafe {
                        if let Some(state) = #static_state_ident.get(id) {
                            return state
                                .try_write()
                                .map_err(|err| crate::commons::trace::trace(5, err.into())).ok();
                        }
                    }
                    None
                }

                fn remove(id: &str) -> Option<std::sync::Arc<std::sync::RwLock<#struct_ident>>> {
                    #[allow(static_mut_refs)]
                    unsafe { #static_state_ident.remove(id) }
                }
            }

        impl #struct_ident {
            #(#sync_variable_slots)*
            #(#sync_object_slots)*
        }

        impl crate::mirror::component::state::State for #struct_ident {
            fn on_serialize_sync_variable(
                &mut self,
                index: u8,
                dirty_bit: &mut u64,
                writer: &mut crate::mirror::network_writer::NetworkWriter,
                initial: bool,
            ) {
                use crate::mirror::network_writer::DataTypeSerializer;
                #(
                    self.#sync_variable_field_idents.serialize(dirty_bit, index, writer, initial);
                )*
            }

            fn on_serialize_sync_object(
                &mut self,
                index: u8,
                dirty_bit: &mut u64,
                writer: &mut crate::mirror::network_writer::NetworkWriter,
                initial: bool,
            ) {
                use crate::mirror::network_writer::DataTypeSerializer;
                #(
                    self.#sync_object_field_idents.serialize(dirty_bit, index, writer, initial);
                )*
            }

            fn on_deserialize_sync_variable(
                &mut self,
                index: u8,
                dirty_bit: &mut u64,
                reader: &mut crate::mirror::network_reader::NetworkReader,
                initial: bool,
            ) {
                #(
                    self.#sync_variable_field_idents.deserialize(dirty_bit, index, reader, initial);
                )*
            }

            fn on_deserialize_sync_object(
                &mut self,
                index: u8,
                dirty_bit: &mut u64,
                reader: &mut crate::mirror::network_reader::NetworkReader,
                initial: bool,
            ) {
                #(
                    self.#sync_object_field_idents.deserialize(dirty_bit, index, reader, initial);
                )*
            }
        }
    })
}
