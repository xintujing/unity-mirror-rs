use crate::utils::string_case::StringCase;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Field, Fields, Path, VisRestricted, Visibility, parse_quote};

struct NetworkBehaviourArgs {
    pub parent: Option<Path>,
    pub metadata: Option<Path>,
    pub not_impl_nos: bool,
}

impl Parse for NetworkBehaviourArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut parent = None;
        let mut metadata = None;
        let mut not_impl_nos = false;

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
                    "metadata" => {
                        let content;
                        syn::parenthesized!(content in input); // 捕获括号内的内容
                        if let Ok(path) = content.parse::<Path>() {
                            metadata = Some(path)
                        }
                    }
                    "not_impl_nos" => {
                        not_impl_nos = true;
                    }
                    _ => {}
                }
            }
            let _ = input.parse::<Comma>();
        }

        Ok(NetworkBehaviourArgs {
            parent,
            metadata,
            not_impl_nos,
        })
    }
}

pub(crate) fn handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let NetworkBehaviourArgs {
        parent,
        metadata,
        not_impl_nos,
    } = syn::parse_macro_input!(attr as NetworkBehaviourArgs);

    if parent.is_none() {
        panic!("`handler` attribute can only be applied to parent network behaviour");
    }

    if metadata.is_none() {
        panic!("`handler` attribute can only be applied to metadata network behaviour");
    }

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
        #[derive(Default, Debug, unity_mirror_macro_rs::SyncState)]
    ));

    let mut on_serialize_ts = Vec::new();
    if !not_impl_nos {
        on_serialize_ts.push(quote! {
            // impl crate::mirror::NetworkBehaviourOnSerializer for #struct_ident {
            impl crate::mirror::NetworkBehaviourOnSerializer for #struct_ident {
                fn on_serialize(&mut self, writer: &mut crate::mirror::NetworkWriter, initial_state: bool) {
                    if let Some(mut parent) = self.parent.get() {
                        use crate::mirror::NetworkBehaviourOnSerializer;
                        parent.on_serialize(writer, initial_state);
                    }
                    use crate::mirror::NetworkBehaviourSerializer;
                    self.serialize_sync_objects(writer, initial_state);
                    self.serialize_sync_vars(writer, initial_state);
                }
            }

            // impl crate::mirror::NetworkBehaviourOnDeserializer for #struct_ident
            impl crate::mirror::NetworkBehaviourOnDeserializer for #struct_ident {
                fn on_deserialize(&mut self, reader: &mut crate::mirror::NetworkReader, initial_state: bool) {
                    if let Some(mut parent) = self.parent.get() {
                        use crate::mirror::NetworkBehaviourOnDeserializer;
                        parent.on_deserialize(reader, initial_state);
                    }
                    use crate::mirror::NetworkBehaviourDeserializer;
                    self.deserialize_sync_objects(reader, initial_state);
                    self.deserialize_sync_vars(reader, initial_state);
                }
            }

        });
    }

    // 收集同步对象
    let mut sync_obj_fields = Vec::new();
    // 收集同步变量
    let mut sync_var_fields = Vec::new();
    // 遍历 struct 的 fields
    for field in &mut item_struct.fields {
        if let Visibility::Inherited = &field.vis {
            // 修改为 pub(super)
            let vr = VisRestricted {
                pub_token: Default::default(),
                paren_token: Default::default(),
                in_token: None,
                path: Box::new(syn::Path::from(Ident::new(
                    "super",
                    proc_macro2::Span::call_site(),
                ))),
            };
            field.vis = Visibility::Restricted(vr);
        }

        for attr in &field.attrs {
            if attr.path().is_ident("sync_object") {
                sync_obj_fields.push(field.ident.clone().unwrap());
                break;
            }
            if attr.path().is_ident("sync_variable") {
                // 修改字段的可见性
                field.vis = Visibility::Inherited;
                sync_var_fields.push((field.ident.clone().unwrap(), field.ty.clone()));
                break;
            }
        }
    }
    let sync_var_count = sync_var_fields.len();
    let sync_obj_count = sync_obj_fields.len();

    let mut init_sync_objs = Vec::new();
    let mut serialize_sync_objs_all_ts = Vec::new();
    let mut serialize_sync_objs_delta_ts = Vec::new();
    let mut deserialize_sync_objs_all_ts = Vec::new();
    let mut deserialize_sync_objs_delta_ts = Vec::new();
    let mut clear_sync_objs_changes_ts = Vec::new();

    for (field_index, field) in sync_obj_fields.iter().enumerate() {
        init_sync_objs.push(quote! {
            this.#field.set_network_behaviour(this.ancestor.clone());
            this.#field.set_index(#field_index as u8 + this.obj_start_offset);
        });

        serialize_sync_objs_all_ts.push(quote! {
            self.#field.on_serialize_all(writer);
        });

        serialize_sync_objs_delta_ts.push(quote! {
            self.#field.on_serialize_delta(writer);
        });

        deserialize_sync_objs_all_ts.push(quote! {
            self.#field.on_deserialize_all(reader);
        });

        deserialize_sync_objs_delta_ts.push(quote! {
            self.#field.on_deserialize_delta(reader);
        });

        clear_sync_objs_changes_ts.push(quote! {
            self.#field.clear_changes();
        });
    }

    let mut serialize_sync_var_ts = Vec::new();
    let mut deserialize_sync_var_ts = Vec::new();
    let mut sync_variable_getter_setter = vec![];
    let mut on_change_callback_ts = Vec::new();
    let mut parent_slot = None;

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
                self.#set_sync_field_ident(<#field_type as crate::mirror::DataTypeDeserializer>::deserialize(reader));
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
        pub(super) ancestor: crate::commons::revel_weak::RevelWeak<Box<crate::mirror::NetworkBehaviour>>
    ));

    // 它的父组件
    if let Some(parent_path) = &parent {
        // 父组件字段
        ext_fields.push(parse_quote! {
            pub(super) parent: crate::commons::revel_weak::RevelWeak<Box<#parent_path>>
        });

        parent_slot = Some(quote! {
            impl core::ops::Deref for #struct_ident {
                type Target = Box<#parent_path>;

                fn deref(&self) -> &Self::Target {
                    self.parent.get().unwrap()
                }
            }

            impl core::ops::DerefMut for #struct_ident {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    self.parent.get().unwrap()
                }
            }
        })
    }

    // obj偏移
    ext_fields.push(parse_quote!(
        obj_start_offset: u8
    ));

    // var偏移
    ext_fields.push(parse_quote!(
        var_start_offset: u8
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

            #parent_slot

            impl #struct_ident {
                pub fn factory(
                    weak_game_object: crate::commons::revel_weak::RevelWeak<crate::unity_engine::GameObject>,
                    metadata: &crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper,
                    weak_network_behaviour: &mut crate::commons::revel_weak::RevelWeak<Box<crate::mirror::NetworkBehaviour>>,
                    sync_object_offset: &mut u8,
                    sync_var_offset: &mut u8,
                ) -> Vec<(crate::commons::revel_arc::RevelArc<Box<dyn crate::unity_engine::MonoBehaviour>>,std::any::TypeId)> {

                    let mut network_behaviour_chain = #parent::factory(weak_game_object.clone(), metadata, weak_network_behaviour, sync_object_offset, sync_var_offset);

                    let mut this = Self::new(weak_game_object.clone(), metadata);

                    // 同步偏移
                    {
                        this.obj_start_offset = *sync_object_offset;
                        this.var_start_offset = *sync_var_offset;

                        *sync_object_offset += #sync_obj_count as u8;
                        *sync_var_offset += #sync_var_count as u8;
                    }

                     // 祖先弱指针
                    if let Some((arc_nb, _)) = network_behaviour_chain.first() {
                        if let Some(weak_nb) = arc_nb.downgrade().downcast::<crate::mirror::NetworkBehaviour>() {
                            this.ancestor = weak_nb.clone();
                        }
                    }

                    // 父亲弱指针
                    if let Some((arc_nb, _)) = network_behaviour_chain.last() {
                        if let Some(weak_nb) = arc_nb.downgrade().downcast::<#parent>()
                        {
                            this.parent = weak_nb.clone();
                        }
                    }

                    // 初始化同步对象
                    {
                        use crate::mirror::sync_object::SyncObject;
                        #(#init_sync_objs)*
                    }

                    // 应用配置
                    {
                        let config = metadata.get::<#metadata>();
                    }

                    let arc_this = crate::commons::revel_arc::RevelArc::new(Box::new(this) as Box<dyn crate::mirror::TNetworkBehaviour> as Box<dyn crate::unity_engine::MonoBehaviour>);

                    network_behaviour_chain.push((arc_this, std::any::TypeId::of::<Self>()));

                    network_behaviour_chain
                }
            }

            // 注册工厂
            #[ctor::ctor]
            fn static_init() {
                crate::mirror::NetworkBehaviourFactory::register::<#struct_ident>(#struct_ident::factory);
            }

            // impl crate::mirror::NetworkBehaviourOnSerializer for #struct_ident {
            #(#on_serialize_ts)*


            // impl crate::mirror::NetworkBehaviourBase for #struct_ident {
            impl crate::mirror::NetworkBehaviourBase for #struct_ident {
                fn is_dirty(&self) -> bool {
                    if let Some(ancestor) = self.ancestor.get() {
                        return ancestor.is_dirty();
                    }
                    false
                }

                fn get_sync_direction(&self) -> &crate::mirror::SyncDirection {
                    if let Some(ancestor) = self.ancestor.get() {
                        return ancestor.get_sync_direction();
                    }
                    &crate::mirror::SyncDirection::ServerToClient
                }

                fn get_sync_mode(&self) -> &crate::mirror::SyncMode {
                    if let Some(ancestor) = self.ancestor.get() {
                        return ancestor.get_sync_mode();
                    }
                    &crate::mirror::SyncMode::Observers
                }

                fn clear_all_dirty_bits(&mut self) {
                    if let Some(mut parent) = self.parent.get() {
                        parent.clear_all_dirty_bits();
                    }
                    #(#clear_sync_objs_changes_ts)*
                }
            }

            // impl crate::mirror::NetworkBehaviourSerializer for #struct_ident {
            impl crate::mirror::NetworkBehaviourSerializer for #struct_ident {
                fn serialize_sync_objects(&mut self, writer: &mut crate::mirror::NetworkWriter, initial_state: bool) {
                    if initial_state {
                        self.serialize_objects_all(writer);
                    } else {
                        self.serialize_sync_object_delta(writer);
                    }
                }

                fn serialize_objects_all(&mut self, writer: &mut crate::mirror::NetworkWriter) {
                    use crate::mirror::sync_object::SyncObject;
                    #(#serialize_sync_objs_all_ts)*
                }

                fn serialize_sync_object_delta(&mut self, writer: &mut crate::mirror::NetworkWriter) {
                    use crate::mirror::sync_object::SyncObject;
                    #(#serialize_sync_objs_delta_ts)*
                }

                fn serialize_sync_vars(&mut self, writer: &mut crate::mirror::NetworkWriter, initial_state: bool) {
                    if #sync_var_count == 0 {
                        return;
                    }

                    if let Some(mut network_behaviour) = self.ancestor.get() {
                        use crate::mirror::DataTypeSerializer;
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

            // impl crate::mirror::NetworkBehaviourDeserializer for #struct_ident {
            impl crate::mirror::NetworkBehaviourDeserializer for #struct_ident {
                fn deserialize_sync_objects(&mut self, reader: &mut crate::mirror::NetworkReader, initial_state: bool) {
                    if initial_state {
                        self.deserialize_objects_all(reader);
                    } else {
                        self.deserialize_sync_object_delta(reader);
                    }
                }

                fn deserialize_objects_all(&mut self, reader: &mut crate::mirror::NetworkReader) {
                    use crate::mirror::sync_object::SyncObject;
                    #(#deserialize_sync_objs_all_ts)*
                }

                fn deserialize_sync_object_delta(&mut self, reader: &mut crate::mirror::NetworkReader) {
                    use crate::mirror::sync_object::SyncObject;
                    #(#deserialize_sync_objs_delta_ts)*
                }

                fn deserialize_sync_vars(&mut self, reader: &mut crate::mirror::NetworkReader, initial_state: bool) {
                    if #sync_var_count == 0 {
                        return;
                    }

                    if let Some(mut network_behaviour) = self.ancestor.get() {
                        use crate::mirror::DataTypeDeserializer;
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

        // impl crate::mirror::TBaseNetworkBehaviour for #struct_ident {
        impl crate::mirror::TBaseNetworkBehaviour for #struct_ident {
        }

    })
}

pub(crate) fn ancestor_on_serialize(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut item_fn = syn::parse_macro_input!(item as syn::ItemFn);

    item_fn.block.stmts.insert(
        0,
        parse_quote!({
            if let Some(mut ancestor) = self.ancestor.get() {
                use crate::mirror::NetworkBehaviourOnSerializer;
                ancestor.on_serialize(writer, initial_state);
            }
        }),
    );

    TokenStream::from(quote! {
        #item_fn
    })
}

pub(crate) fn ancestor_on_deserialize(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut item_fn = syn::parse_macro_input!(item as syn::ItemFn);

    item_fn.block.stmts.insert(
        0,
        parse_quote!({
            if let Some(mut ancestor) = self.ancestor.get() {
                use crate::mirror::NetworkBehaviourOnDeserializer;
                ancestor.on_deserialize(reader, initial_state);
            }
        }),
    );

    TokenStream::from(quote! {
        #item_fn
    })
}

pub(crate) fn parent_on_serialize(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut item_fn = syn::parse_macro_input!(item as syn::ItemFn);

    item_fn.block.stmts.insert(
        0,
        parse_quote!(if let Some(mut parent) = self.parent.get() {
            use crate::mirror::NetworkBehaviourOnSerializer;
            parent.on_serialize(writer, initial_state);
        }),
    );

    TokenStream::from(quote! {
        #item_fn
    })
}

pub(crate) fn parent_on_deserialize(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut item_fn = syn::parse_macro_input!(item as syn::ItemFn);

    item_fn.block.stmts.insert(
        0,
        parse_quote!(if let Some(mut parent) = self.parent.get() {
            use crate::mirror::NetworkBehaviourOnDeserializer;
            parent.on_deserialize(reader, initial_state);
        }),
    );

    TokenStream::from(quote! {
        #item_fn
    })
}
