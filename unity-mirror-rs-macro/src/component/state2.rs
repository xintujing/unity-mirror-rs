use crate::string_case::StringCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, Fields};

pub(crate) fn handler(#[allow(unused)] attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(item as syn::ItemStruct);

    item_struct.attrs.push(parse_quote!(
        #[derive(Default, Debug, unity_mirror_rs_macro::InnerState)]
    ));

    if let Fields::Named(fields_named) = &mut item_struct.fields {
        fields_named.named.push(parse_quote!(
            id: String
        ));
        fields_named.named.push(parse_quote!(
            var_start_offset: u8
        ));
        fields_named.named.push(parse_quote!(
            obj_start_offset: u8
        ));
    };

    // 修改而非添加结构体字段animator_speed为私有
    if let Fields::Named(fields_named) = &mut item_struct.fields {
        for field in fields_named.named.iter_mut() {
            if field.ident == Some(format_ident!("animator_speed")) {
                field.vis = syn::Visibility::Inherited;
            }
        }
    };

    // let fields = match &mut item_struct.fields {
    //     Fields::Named(fields) => fields,
    //     _ => panic!("仅支持命名字段的结构体"),
    // };

    let struct_ident = &item_struct.ident;
    let on_change_callback = format_ident!("{}OnChangeCallback", struct_ident);

    let state_condition_ident = format_ident!("Base{}", struct_ident);

    let static_state_ident = format_ident!(
        "{}",
        struct_ident.to_string().to_snake_case().to_uppercase()
    );

    let mut sync_variable_index = 0u8;
    let mut sync_object_index = 0u8;

    let mut sync_variable_serialize_slots = vec![];
    let mut sync_variable_deserialize_slots = vec![];
    let mut sync_objects_serialize_slots = vec![];
    let mut sync_objects_deserialize_slots = vec![];

    let mut on_change_callback_slots = vec![];

    let mut get_set_sync_fns = vec![];

    for field in item_struct.fields.iter() {
        for field_attr in field.attrs.iter() {
            if let syn::Type::Path(type_path) = &field.ty {
                if type_path.path.segments.len() > 0 {
                    let field_type = &type_path.path.segments;

                    // let field_type_ident = &segment.ident;
                    // 获取字段名称
                    let field_ident = &field.ident.clone().unwrap();
                    let get_sync_field_ident = format_ident!("get_{}", field_ident);
                    let set_sync_field_ident = format_ident!("set_{}", field_ident);

                    let on_change_callback_ident = format_ident!("on_{}_changed", field_ident);

                    if field_attr.meta.path().is_ident("sync_variable") {
                        sync_variable_serialize_slots.push(quote! {
                            if initial || *dirty_bit & (1u64 << (#sync_variable_index + index)) != 0 {
                                // self.value.serialize(writer);
                                crate::mirror::network_writer::DataTypeSerializer::serialize(&self.#field_ident, writer);
                            }
                        });
                        sync_variable_deserialize_slots.push(quote! {
                            if initial || *dirty_bit & (1u64 << (#sync_variable_index + index)) != 0 {
                                self.#set_sync_field_ident(<#type_path as crate::mirror::network_reader::DataTypeDeserializer>::deserialize(reader));
                            }
                        });

                        get_set_sync_fns.push(quote! {
                            pub fn #get_sync_field_ident(&self) -> &#field_type {
                                &self.#field_ident
                            }

                            pub fn #set_sync_field_ident(&mut self, value: #field_type) {
                                let old_value = self.#field_ident.clone();
                                self.#field_ident = value.clone();
                                if let Some(mut state) = NetworkBehaviour::state_mut(&self.id) {
                                    state.sync_var_dirty_bit |= 1u64 << (#sync_variable_index + self.var_start_offset);
                                }
                                self.#on_change_callback_ident(&old_value, &value)
                            }
                        });

                        sync_variable_index += 1;
                    }

                    if field_attr.meta.path().is_ident("sync_object") {
                        sync_objects_serialize_slots.push(quote! {
                            if initial || *dirty_bit & (1u64 << (#sync_object_index + index)) != 0 {
                                // self.value.serialize(writer);
                                // crate::mirror::network_writer::DataTypeSerializer::serialize(&#field_name, writer);
                            }
                        });
                        sync_objects_deserialize_slots.push(quote! {
                            if initial || *dirty_bit & (1u64 << (#sync_object_index + index)) != 0 {
                                // self.#set_sync_field_ident(crate::mirror::network_reader::DataTypeDeserializer::deserialize(reader));
                            }
                        });

                        get_set_sync_fns.push(quote! {
                            pub fn #get_sync_field_ident(&self) -> &#field_type {
                                &self.#field_ident
                            }

                            pub fn #set_sync_field_ident(&mut self, value: #field_type) {
                                let old_value = self.#field_ident.clone();
                                self.#field_ident = value.clone();
                                if let Some(mut state) = NetworkBehaviour::state_mut(&self.id) {
                                    state.sync_object_dirty_bit |= 1u64 << (#sync_object_index + self.var_start_offset);
                                }
                                self.#on_change_callback_ident(&old_value, &value)
                            }
                        });

                        sync_object_index += 1;
                    }

                    on_change_callback_slots.push(quote! {
                            fn #on_change_callback_ident(&mut self, old_value: &#type_path, new_value: &#type_path) {}
                    });
                }
            }
        }
    }

    TokenStream::from(quote! {
        // #item_struct

        trait #state_condition_ident: #on_change_callback {}

        trait #on_change_callback {
            #(#on_change_callback_slots)*
        }

        impl #state_condition_ident for #struct_ident {}

        mod private {
            use super::*;

            static mut #static_state_ident: once_cell::sync::Lazy<
                std::collections::HashMap<String, std::sync::Arc<std::sync::RwLock<#struct_ident>>>
            > = once_cell::sync::Lazy::new(|| std::collections::HashMap::new());


            #item_struct

            impl #struct_ident {
                #(#get_set_sync_fns)*
            }

            impl #struct_ident {
                #[allow(unused)]
                pub(super) fn new(id: &str, mut state: #struct_ident, obj_start_offset: &mut u8, var_start_offset: &mut u8) {
                    state.id = id.to_string();
                    state.obj_start_offset = *obj_start_offset;
                    state.var_start_offset = *var_start_offset;

                    *obj_start_offset += #sync_object_index;
                    *var_start_offset += #sync_variable_index;

                    #[allow(static_mut_refs)]
                    unsafe {
                        #static_state_ident.insert(id.to_string(), std::sync::Arc::new(std::sync::RwLock::new(state)));
                    }
                }
                #[allow(unused)]
                pub(super) fn get(id: &str) -> Option<std::sync::RwLockReadGuard<#struct_ident>> {
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

                #[allow(unused)]
                pub(super) fn get_mut(
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

                #[allow(unused)]
                pub(super) fn remove(id: &str) -> Option<std::sync::Arc<std::sync::RwLock<#struct_ident>>> {
                    #[allow(static_mut_refs)]
                    unsafe { #static_state_ident.remove(id) }
                }
            }

            impl crate::mirror::component::state::State for #struct_ident {
                fn on_serialize_sync_variable(
                    &mut self,
                    index: u8,
                    dirty_bit: &mut u64,
                    writer: &mut crate::mirror::network_writer::NetworkWriter,
                    initial: bool,
                ) {
                    #(#sync_variable_serialize_slots)*
                }

                fn on_serialize_sync_object(
                    &mut self,
                    index: u8,
                    dirty_bit: &mut u64,
                    writer: &mut crate::mirror::network_writer::NetworkWriter,
                    initial: bool,
                ) {
                    #(#sync_objects_serialize_slots)*
                }

                fn on_deserialize_sync_variable(
                    &mut self,
                    index: u8,
                    dirty_bit: &mut u64,
                    reader: &mut crate::mirror::network_reader::NetworkReader,
                    initial: bool,
                ) {
                    #(#sync_variable_deserialize_slots)*
                }

                fn on_deserialize_sync_object(
                    &mut self,
                    index: u8,
                    dirty_bit: &mut u64,
                    reader: &mut crate::mirror::network_reader::NetworkReader,
                    initial: bool,
                ) {
                    #(#sync_objects_deserialize_slots)*
                }
            }
        }


        pub use private::#struct_ident;

    })
}
