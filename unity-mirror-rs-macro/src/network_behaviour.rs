use crate::string_case::StringCase;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{parse_quote, Field, Fields, Path};

struct NetworkBehaviourArgs {
    pub parent: Option<Path>,
}

impl Parse for NetworkBehaviourArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut parent = None;

        while !input.is_empty() {
            {
                match input.parse::<Ident>()?.to_string().as_str() {
                    "parent" => {
                        let content;
                        syn::parenthesized!(content in input); // 捕获括号内的内容
                        if let Ok(path) = content.parse::<Path>() {
                            parent = Some(path)
                        }
                    }
                    _ => {}
                }
            }
            let _ = input.parse::<Comma>();
        }

        Ok(NetworkBehaviourArgs { parent })
    }
}

pub(crate) fn handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let NetworkBehaviourArgs { parent } = syn::parse_macro_input!(attr as NetworkBehaviourArgs);

    let mut item_struct = syn::parse_macro_input!(item as syn::ItemStruct);

    // struct_ident
    let struct_ident = &item_struct.ident;

    let state_condition_ident = format_ident!("Base{}", struct_ident);
    let on_change_callback = format_ident!("{}OnChangeCallback", struct_ident);

    match item_struct.fields {
        Fields::Named(_) => {}
        _ => {
            return syn::Error::new_spanned(
                item_struct.fields,
                "The component macro only supports named fields.",
            )
            .to_compile_error()
            .into();
        }
    }

    // 添加属性
    item_struct.attrs.push(parse_quote!(
        #[derive(Default, Debug, unity_mirror_macro::SyncState)]
    ));

    // 收集同步对象
    let mut sync_obj_fields = Vec::new();
    // 收集同步变量
    let mut sync_var_fields = Vec::new();
    // 遍历 struct 的 fields
    for field in &mut item_struct.fields {
        for attr in &field.attrs {
            if attr.path().is_ident("sync_object") {
                sync_obj_fields.push(field.ident.clone().unwrap());
                break;
            }
            if attr.path().is_ident("sync_variable") {
                // 修改字段的可见性
                field.vis = syn::Visibility::Inherited;
                sync_var_fields.push((field.ident.clone().unwrap(), field.ty.clone()));
                break;
            }
        }
    }
    let sync_var_count = sync_var_fields.len();

    // //
    // let mut serialize_sync_objs_all_ts = Vec::new();
    // let mut serialize_sync_objs_delta_ts = Vec::new();
    // let mut deserialize_sync_objs_all_ts = Vec::new();
    // let mut deserialize_sync_objs_delta_ts = Vec::new();
    // let mut clear_sync_objs_changes_ts = Vec::new();
    //
    // //
    let mut serialize_sync_var_ts = Vec::new();
    let mut deserialize_sync_var_ts = Vec::new();
    let mut sync_variable_getter_setter = vec![];
    let mut on_change_callback_ts = Vec::new();

    for (field_index, (field, field_type)) in sync_var_fields.iter().enumerate() {
        let get_sync_field_ident = format_ident!("get_{}", field);
        let set_sync_field_ident = format_ident!("set_{}", field);
        let on_change_callback_ident = format_ident!("on_{}_changed", field);
        serialize_sync_var_ts.push(quote! {
            if initial_state || (dirty_bits & (1u64 << (self.var_start_offset + #field_index as u8))) != 0 {
                self.#field.serialize(writer);
            }
        });

        deserialize_sync_var_ts.push(quote! {
            if initial_state || (dirty_bits & (1u64 << (self.var_start_offset + #field_index as u8))) != 0 {
                self.#set_sync_field_ident(<#field_type as crate::mirror::network_reader::DataTypeDeserializer>::deserialize(reader));
            }
        });

        sync_variable_getter_setter.push(quote! {
            pub fn #get_sync_field_ident(&self) -> &#field_type {
                &self.#field
            }

            pub fn #set_sync_field_ident(&mut self, value: #field_type) {

                 let old_value = unsafe {
                    let mut value_buffer = [0u8; size_of::<#field_type>()];
                    std::ptr::copy_nonoverlapping(
                        &self.#field as *const #field_type as *const u8,
                        value_buffer.as_mut_ptr(),
                        size_of::<#field_type>(),
                    );
                    std::mem::transmute::<[u8; size_of::<#field_type>()], #field_type>(value_buffer)
                };

                let new_value = unsafe {
                    let mut value_buffer = [0u8; size_of::<#field_type>()];
                    std::ptr::copy_nonoverlapping(
                        &value as *const #field_type as *const u8,
                        value_buffer.as_mut_ptr(),
                        size_of::<#field_type>(),
                    );
                    std::mem::transmute::<[u8; size_of::<#field_type>()], #field_type>(value_buffer)
                };

                self.#field = value;

                if let Some(mut network_behaviour) = self.ancestor.get() {
                    network_behaviour.sync_var_dirty_bits |= 1u64 << (self.var_start_offset + #field_index as u8);
                }

                self.#on_change_callback_ident(&old_value, &new_value)
            }
        });

        on_change_callback_ts.push(quote! {
            fn #on_change_callback_ident(&mut self, old_value: &#field_type, new_value: &#field_type){}
        });
    }

    // 扩展字段
    let mut ext_fields = Punctuated::<Field, Comma>::new();

    // 它的祖先 ancestor
    ext_fields.push(parse_quote!(
        ancestor: crate::commons::revel_weak::RevelWeak<crate::mirror::network_behaviour::NetworkBehaviour>
    ));

    // 它的父组件
    if let Some(parent_path) = &parent {
        // 父组件字段
        ext_fields.push(parse_quote! {
            parent: crate::commons::revel_weak::RevelWeak<#parent_path>
        });
    }

    // var偏移
    ext_fields.push(parse_quote!(
        var_start_offset: u8
    ));

    // obj偏移
    ext_fields.push(parse_quote!(
        obj_start_offset: u8
    ));

    // 扩展字段
    match &mut item_struct.fields {
        Fields::Named(fields_named) => {
            fields_named.named.extend(ext_fields);
        }
        _ => {}
    }

    // 私有模块
    let this_struct_private_mod_ident = format_ident!(
        "private_component_{}",
        struct_ident.to_string().to_snake_case().to_lowercase()
    );

    TokenStream::from(quote! {
        mod #this_struct_private_mod_ident {
            use super::*;

            #item_struct

            impl #struct_ident {
                pub fn factory(
                    weak_game_object: crate::commons::revel_weak::RevelWeak<crate::unity_engine::GameObject>,
                    metadata: &crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper,
                    weak_network_behaviour: &mut crate::commons::revel_weak::RevelWeak<
                        crate::mirror::network_behaviour::NetworkBehaviour,
                    >,
                    sync_object_offset: &mut u8,
                    sync_var_offset: &mut u8,
                ) -> Vec<(RevelArc<Box<dyn MonoBehaviour>>, TypeId)> {
                    use super::NetworkBehaviour;
                    Self::new(metadata);
                    Vec::new()
                }
            }

            // 注册工厂
            #[ctor::ctor]
            fn static_init() {
                crate::mirror::network_behaviour_factory::NetworkBehaviourFactory::register::<#struct_ident>(#struct_ident::factory);
            }

            // impl crate::mirror::network_behaviour_trait::NetworkBehaviourSerializer for #struct_ident {
            impl crate::mirror::network_behaviour_trait::NetworkBehaviourSerializer for #struct_ident {
                fn serialize_sync_objects(&mut self, writer: &mut crate::mirror::network_writer::NetworkWriter, initial_state: bool) {
                    if initial_state {
                        self.serialize_objects_all(writer);
                    } else {
                        self.serialize_sync_object_delta(writer);
                    }
                }

                fn serialize_objects_all(&mut self, writer: &mut crate::mirror::network_writer::NetworkWriter) {

                }

                fn serialize_sync_object_delta(&mut self, writer: &mut crate::mirror::network_writer::NetworkWriter) {

                }

                fn serialize_sync_vars(&mut self, writer: &mut crate::mirror::network_writer::NetworkWriter, initial_state: bool) {
                    if #sync_var_count == 0 {
                        return;
                    }

                    if let Some(mut network_behaviour) = self.ancestor.get() {
                        use crate::mirror::network_writer::DataTypeSerializer;
                        let dirty_bits = network_behaviour.sync_var_dirty_bits;
                        if initial_state{
                            #(#serialize_sync_var_ts)*
                            return;
                        }
                        writer.write_blittable::<u64>(dirty_bits);
                        #(#serialize_sync_var_ts)*
                    }
                }
            }

            // impl crate::mirror::network_behaviour_trait::NetworkBehaviourDeserializer for #struct_ident {
            impl crate::mirror::network_behaviour_trait::NetworkBehaviourDeserializer for #struct_ident {
                fn deserialize_sync_objects(&mut self, reader: &mut crate::mirror::network_reader::NetworkReader, initial_state: bool) {
                    if initial_state {
                        self.deserialize_objects_all(reader);
                    } else {
                        self.deserialize_sync_object_delta(reader);
                    }
                }

                fn deserialize_objects_all(&mut self, reader: &mut crate::mirror::network_reader::NetworkReader) {

                }

                fn deserialize_sync_object_delta(&mut self, reader: &mut crate::mirror::network_reader::NetworkReader) {

                }

                fn deserialize_sync_vars(&mut self, reader: &mut crate::mirror::network_reader::NetworkReader, initial_state: bool) {
                    if #sync_var_count == 0 {
                        return;
                    }

                    if let Some(mut network_behaviour) = self.ancestor.get() {
                        use crate::mirror::network_reader::DataTypeDeserializer;
                        let mut dirty_bits = 0;
                        if initial_state{
                            #(#deserialize_sync_var_ts)*
                            return;
                        }
                        network_behaviour.sync_var_dirty_bits = reader.read_blittable::<u64>();
                        dirty_bits = network_behaviour.sync_var_dirty_bits;
                        #(#deserialize_sync_var_ts)*
                    }
                }
            }

            // 同步变量 get/set
            impl #struct_ident {
                #(#sync_variable_getter_setter)*
            }
        }

        pub use #this_struct_private_mod_ident::#struct_ident;

        trait #state_condition_ident: #on_change_callback {}

        trait #on_change_callback {
            #(#on_change_callback_ts)*
        }

        impl #state_condition_ident for #struct_ident {}

        // impl crate::mirror::network_behaviour_trait::BaseNetworkBehaviourT for #struct_ident {
        impl crate::mirror::network_behaviour_trait::BaseNetworkBehaviourT for #struct_ident {
        }

        // impl crate::mirror::network_behaviour_trait::NetworkBehaviourInstance for #struct_ident {
        //     fn instance(weak_game_object: RevelWeak<GameObject>, metadata: &MetadataNetworkBehaviourWrapper) -> (Vec<(RevelArc<Box<dyn MonoBehaviour>>, TypeId)>, RevelWeak<crate::mirror::NetworkBehaviour>, u8, u8)
        //     where
        //         Self: Sized
        //     {
        //         todo!()
        //     }
        // }

        // #namespace_slot

    })
}
