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

    let struct_ident = &item_struct.ident;

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
                sync_var_fields.push(field.ident.clone().unwrap());
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

    for (field_index, field) in sync_var_fields.iter().enumerate() {
        let get_sync_field_ident = format_ident!("get_{}", field);
        let set_sync_field_ident = format_ident!("set_{}", field);
        serialize_sync_var_ts.push(quote! {
            if initial_state || (dirty_bits & (1u64 << (self.var_start_offset + #field_index as u8))) != 0 {
                self.#field.serialize(writer);
            }
        });

        deserialize_sync_var_ts.push(quote! {
            if initial_state || (dirty_bits & (1u64 << (self.var_start_offset + #field_index as u8))) != 0 {
                // self.#set_sync_field_ident(reader.deserialize());
            }
        });
    }

    // 扩展字段
    let mut ext_fields = Punctuated::<Field, Comma>::new();

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

    // ---------------------------------------------------------
    // let mut state_instance_slot = None;

    // let mut state_clear_slot = None;
    // let mut component_state_impl_slot = None;
    //
    // let mut variable_serialize_slot = None;
    // let mut variable_deserialize_slot = None;
    // let mut object_serialize_slot = None;
    // let mut object_deserialize_slot = None;
    //
    // let mut this_component_state_trait_slot = None;
    //
    //
    //
    // if let Some(state_path) = state {
    //     // named.push(parse_quote! { state: #state_path });
    //     // instance_fields.push(parse_quote! { state });
    //
    //     let this_component_state_trait_ident = format_ident!("{}StateTrait", struct_ident);
    //
    //     this_component_state_trait_slot = Some(quote! {
    //         trait #this_component_state_trait_ident: crate::mirror::component::state::StateInitialize {}
    //         impl #this_component_state_trait_ident for #state_path {}
    //     });
    //
    //     state_instance_slot = Some(quote! {
    //         use crate::mirror::component::state::StateInitialize;
    //         let mut state = #state_path::default();
    //         state.initialize(settings);
    //         #state_path::new(&id, state, obj_start_offset, var_start_offset);
    //     });
    //
    //     state_clear_slot = Some(quote! {
    //         #state_path::remove(&self.id);
    //     });
    //
    //     component_state_impl_slot = Some(quote! {
    //         impl #struct_ident {
    //             pub fn state(id: &str) -> Option<std::sync::RwLockReadGuard<#state_path>> {
    //                 #state_path::get(id)
    //             }
    //             pub fn state_mut(
    //                 id: &str,
    //             ) -> Option<std::sync::RwLockWriteGuard<#state_path>> {
    //                 #state_path::get_mut(id)
    //             }
    //         }
    //     });
    //
    //     // component_serialize_slot = Some(quote! {
    //     //     if let Some(mut network_behaviour_state) = crate::unity_engine::mirror::network_behaviour::i_network_behaviour::NetworkBehaviour::state_mut(&self.id) {
    //     //         if let Some(mut state) = Self::state_mut(&self.id) {
    //     //             use crate::mirror::component::state::State;
    //     //             state.on_serialize_sync_variable (
    //     //                 &mut network_behaviour_state.sync_var_dirty_bit,
    //     //                 writer,
    //     //                 initial,
    //     //             );
    //     //             state.on_serialize_sync_object (
    //     //                 &mut network_behaviour_state.sync_object_dirty_bit,
    //     //                 writer,
    //     //                 initial,
    //     //             );
    //     //         }
    //     //     }
    //     // });
    //
    //     if parent.is_none() {
    //         object_serialize_slot = Some(quote! {
    //             if !initial {
    //                 if let Some(mut network_behaviour_state) = crate::unity_engine::mirror::network_behaviour::i_network_behaviour::NetworkBehaviour::state_mut(&self.id) {
    //                      writer.write_blittable::<u64>(network_behaviour_state.sync_var_dirty_bit);
    //                 }
    //             }
    //         });
    //         // object_deserialize_slot = Some(quote! {
    //         //
    //         // });
    //     } else {
    //         variable_serialize_slot = Some(quote! {
    //             if let Some(mut network_behaviour_state) = crate::unity_engine::mirror::network_behaviour::i_network_behaviour::NetworkBehaviour::state_mut(&self.id) {
    //                 if let Some(mut state) = Self::state_mut(&self.id) {
    //                     use crate::mirror::component::state::State;
    //                     state.on_serialize_sync_variable (
    //                         network_behaviour_state.sync_var_dirty_bit,
    //                         writer,
    //                         initial,
    //                     );
    //                 }
    //             }
    //         });
    //         object_serialize_slot = Some(quote! {
    //             if let Some(mut network_behaviour_state) = crate::unity_engine::mirror::network_behaviour::i_network_behaviour::NetworkBehaviour::state_mut(&self.id) {
    //                 if let Some(mut state) = Self::state_mut(&self.id) {
    //                     use crate::mirror::component::state::State;
    //                     state.on_serialize_sync_object (
    //                         network_behaviour_state.sync_object_dirty_bit,
    //                         writer,
    //                         initial,
    //                     );
    //                 }
    //             }
    //         });
    //         variable_deserialize_slot = Some(quote! {
    //             if let Some(mut state) = Self::state_mut(&self.id) {
    //                 use crate::mirror::component::state::State;
    //                 state.on_deserialize_sync_variable (reader,initial);
    //             }
    //         });
    //         object_deserialize_slot = Some(quote! {
    //             if let Some(mut network_behaviour_state) = crate::unity_engine::mirror::network_behaviour::i_network_behaviour::NetworkBehaviour::state_mut(&self.id) {
    //                 if let Some(mut state) = Self::state_mut(&self.id) {
    //                     use crate::mirror::component::state::State;
    //                     state.on_deserialize_sync_object (
    //                         dirty_bit,
    //                         reader,
    //                         initial,
    //                     );
    //                 }
    //             }
    //         });
    //     }
    //
    //     // component_deserialize_slot = Some(quote! {
    //     //     if let Some(mut network_behaviour_state) = crate::unity_engine::mirror::network_behaviour::i_network_behaviour::NetworkBehaviour::state_mut(&self.id) {
    //     //         if let Some(mut state) = Self::state_mut(&self.id) {
    //     //             use crate::mirror::component::state::State;
    //     //             state.on_deserialize_sync_variable (
    //     //                 &mut network_behaviour_state.sync_var_dirty_bit,
    //     //                 reader,
    //     //                 initial,
    //     //             );
    //     //             state.on_deserialize_sync_object (
    //     //                 &mut network_behaviour_state.sync_object_dirty_bit,
    //     //                 reader,
    //     //                 initial,
    //     //             );
    //     //         }
    //     //     }
    //     // });
    // }

    // ---------------------------------------------------------

    // let mut instance_slot = quote! {
    //     #parent_instance_slot
    //     let instance = Self {
    //         #fields_instance
    //     };
    //     instance
    // };
    // ---------------------------------------------------------

    // item_struct.fields = Fields::Named(syn::FieldsNamed {
    //     brace_token: syn::token::Brace::default(),
    //     named,
    // });

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

            // 注册工厂
            #[ctor::ctor]
            fn static_init() {
                use crate::unity_engine::mirror::network_behaviour_trait::NetworkBehaviourInstance;
                crate::unity_engine::mirror::network_behaviour_factory::NetworkBehaviourFactory::register::<NetworkAnimator>(NetworkAnimator::instance);
            }

            // impl crate::unity_engine::mirror::network_behaviour_trait::NetworkBehaviourSerializer for #struct_ident {
            impl crate::unity_engine::mirror::network_behaviour_trait::NetworkBehaviourSerializer for #struct_ident {
                fn serialize_sync_objects(&mut self, writer: &mut crate::unity_engine::mirror::network_writer::NetworkWriter, initial_state: bool) {
                    if initial_state {
                        self.serialize_objects_all(writer);
                    } else {
                        self.serialize_sync_object_delta(writer);
                    }
                }

                fn serialize_objects_all(&mut self, writer: &mut crate::unity_engine::mirror::network_writer::NetworkWriter) {

                }

                fn serialize_sync_object_delta(&mut self, writer: &mut crate::unity_engine::mirror::network_writer::NetworkWriter) {

                }

                fn serialize_sync_vars(&mut self, writer: &mut crate::unity_engine::mirror::network_writer::NetworkWriter, initial_state: bool) {
                    if #sync_var_count == 0 {
                        return;
                    }

                    if let Some(mut network_behaviour) = self.parent.get() {
                        use crate::unity_engine::mirror::network_writer::DataTypeSerializer;
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

            // impl crate::unity_engine::mirror::network_behaviour_trait::NetworkBehaviourDeserializer for #struct_ident {
            impl crate::unity_engine::mirror::network_behaviour_trait::NetworkBehaviourDeserializer for #struct_ident {
                fn deserialize_sync_objects(&mut self, reader: &mut crate::unity_engine::mirror::network_reader::NetworkReader, initial_state: bool) {
                    if initial_state {
                        self.deserialize_objects_all(reader);
                    } else {
                        self.deserialize_sync_object_delta(reader);
                    }
                }

                fn deserialize_objects_all(&mut self, reader: &mut crate::unity_engine::mirror::network_reader::NetworkReader) {

                }

                fn deserialize_sync_object_delta(&mut self, reader: &mut crate::unity_engine::mirror::network_reader::NetworkReader) {

                }

                fn deserialize_sync_vars(&mut self, reader: &mut crate::unity_engine::mirror::network_reader::NetworkReader, initial_state: bool) {
                    if #sync_var_count == 0 {
                        return;
                    }

                    if let Some(mut network_behaviour) = self.parent.get() {
                        if initial_state{
                            return;
                        }
                        network_behaviour.sync_var_dirty_bits = reader.read_blittable::<u64>();
                    }
                }
            }
        }

        pub use #this_struct_private_mod_ident::#struct_ident;

        // impl crate::unity_engine::mirror::network_behaviour_trait::NetworkBehaviourInstance for #struct_ident {
        //     fn instance(weak_game_object: RevelWeak<GameObject>, metadata: &MetadataNetworkBehaviourWrapper) -> (Vec<(RevelArc<Box<dyn MonoBehaviour>>, TypeId)>, RevelWeak<crate::unity_engine::mirror::NetworkBehaviour>, u8, u8)
        //     where
        //         Self: Sized
        //     {
        //         todo!()
        //     }
        // }

        // #namespace_slot

    })
}
