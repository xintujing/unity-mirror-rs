use crate::namespace::NamespaceArgs;
use crate::string_case::StringCase;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{parse_quote, Field, Fields, Path};

struct ComponentArgs {
    pub state: Option<Path>, // 存储 state 对应的路径
    pub parent: Option<Path>,
    pub namespace: Option<NamespaceArgs>,
}

impl Parse for ComponentArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut state = None;
        let mut parent = None;
        let mut namespace = None;

        while !input.is_empty() {
            {
                match input.parse::<Ident>()?.to_string().as_str() {
                    "state" => {
                        let content;
                        syn::parenthesized!(content in input); // 捕获括号内的内容
                        if let Ok(path) = content.parse::<Path>() {
                            state = Some(path)
                        }
                    }
                    "parent" => {
                        let content;
                        syn::parenthesized!(content in input); // 捕获括号内的内容
                        if let Ok(path) = content.parse::<Path>() {
                            parent = Some(path)
                        }
                    }
                    "namespace" => {
                        let content;
                        syn::parenthesized!(content in input); // 捕获括号内的内容
                        namespace = Some(content.parse::<NamespaceArgs>()?)
                    }
                    _ => {}
                }
            }
            let _ = input.parse::<Comma>();
        }

        Ok(ComponentArgs {
            state,
            parent,
            namespace,
        })
    }
}

pub(crate) fn handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ComponentArgs {
        state,
        parent,
        namespace,
    } = syn::parse_macro_input!(attr as ComponentArgs);

    let mut item_struct = syn::parse_macro_input!(item as syn::ItemStruct);

    let struct_ident = &item_struct.ident;

    match item_struct.fields {
        Fields::Unit => {}
        _ => {
            return syn::Error::new_spanned(
                item_struct.fields,
                "Component macro only supports unit structs",
            )
            .to_compile_error()
            .into();
        }
    }

    item_struct.attrs.push(parse_quote!(
        #[derive(Debug, Clone, Eq, PartialEq)]
    ));

    let namespace_slot = match namespace {
        None => {
            quote! {}
        }
        Some(namespace_args) => {
            let namespace_string = namespace_args.get_namespace(struct_ident);
            quote! {
                impl crate::commons::namespace::Namespace for #struct_ident {
                    fn get_namespace() -> &'static str {
                        #namespace_string
                    }
                }
            }
        }
    };

    let mut named = Punctuated::<Field, Comma>::new();
    let mut instance_fields = Punctuated::<Field, Comma>::new();
    // 组件id字段
    named.push(parse_quote! { id: String });
    instance_fields.push(parse_quote! { id });

    // ---------------------------------------------------------
    // ---------------------------------------------------------

    let mut parent_instance_slot = quote! {
        let id: String = uuid::Uuid::new_v4().to_string().into();
    };
    let mut get_parent_slot = quote! { None };
    let mut parent_component_on_serialize_slot = None;
    let mut parent_component_on_deserialize_slot = None;

    let is_root = parent.is_none();

    if let Some(parent_path) = &parent {
        // 父组件字段
        named.push(parse_quote! { parent: #parent_path });
        instance_fields.push(parse_quote! { parent });

        parent_instance_slot = quote! {
            use crate::mirror::component::component_basic::ComponentBasic;
            let parent = #parent_path::new(settings, obj_start_offset, var_start_offset);
            let id = parent.id().clone();
        };

        get_parent_slot = quote! {
            Some(Box::new(self.parent.clone()))
        };

        parent_component_on_serialize_slot = Some(quote! {
            self.parent.on_serialize(writer, initial);
        });
        parent_component_on_deserialize_slot = Some(quote! {
            self.parent.on_deserialize(reader, initial);
        });
    }

    // ---------------------------------------------------------
    let mut state_instance_slot = None;

    let mut state_clear_slot = None;
    let mut component_state_impl_slot = None;

    let mut variable_serialize_slot = None;
    let mut variable_deserialize_slot = None;
    let mut object_serialize_slot = None;
    let mut object_deserialize_slot = None;

    let mut this_component_state_trait_slot = None;
    if let Some(state_path) = state {
        // named.push(parse_quote! { state: #state_path });
        // instance_fields.push(parse_quote! { state });

        let this_component_state_trait_ident = format_ident!("{}StateTrait", struct_ident);

        this_component_state_trait_slot = Some(quote! {
            trait #this_component_state_trait_ident: crate::mirror::component::state::StateInitialize {}
            impl #this_component_state_trait_ident for #state_path {}
        });

        state_instance_slot = Some(quote! {
            use crate::mirror::component::state::StateInitialize;
            let mut state = #state_path::default();
            state.initialize(settings);
            #state_path::new(&id, state, obj_start_offset, var_start_offset);
        });

        state_clear_slot = Some(quote! {
            #state_path::remove(&self.id);
        });

        component_state_impl_slot = Some(quote! {
            impl #struct_ident {
                pub fn state(id: &str) -> Option<std::sync::RwLockReadGuard<#state_path>> {
                    #state_path::get(id)
                }
                pub fn state_mut(
                    id: &str,
                ) -> Option<std::sync::RwLockWriteGuard<#state_path>> {
                    #state_path::get_mut(id)
                }
            }
        });

        // component_serialize_slot = Some(quote! {
        //     if let Some(mut network_behaviour_state) = crate::mirror::components::network_behaviour::NetworkBehaviour::state_mut(&self.id) {
        //         if let Some(mut state) = Self::state_mut(&self.id) {
        //             use crate::mirror::component::state::State;
        //             state.on_serialize_sync_variable (
        //                 &mut network_behaviour_state.sync_var_dirty_bit,
        //                 writer,
        //                 initial,
        //             );
        //             state.on_serialize_sync_object (
        //                 &mut network_behaviour_state.sync_object_dirty_bit,
        //                 writer,
        //                 initial,
        //             );
        //         }
        //     }
        // });

        if parent.is_none() {
            object_serialize_slot = Some(quote! {
                if !initial {
                    if let Some(mut network_behaviour_state) = crate::mirror::components::network_behaviour::NetworkBehaviour::state_mut(&self.id) {
                         writer.write_blittable::<u64>(network_behaviour_state.sync_var_dirty_bit);
                    }
                }
            });
            // object_deserialize_slot = Some(quote! {
            //
            // });
        } else {
            variable_serialize_slot = Some(quote! {
                if let Some(mut network_behaviour_state) = crate::mirror::components::network_behaviour::NetworkBehaviour::state_mut(&self.id) {
                    if let Some(mut state) = Self::state_mut(&self.id) {
                        use crate::mirror::component::state::State;
                        state.on_serialize_sync_variable (
                            network_behaviour_state.sync_var_dirty_bit,
                            writer,
                            initial,
                        );
                    }
                }
            });
            object_serialize_slot = Some(quote! {
                if let Some(mut network_behaviour_state) = crate::mirror::components::network_behaviour::NetworkBehaviour::state_mut(&self.id) {
                    if let Some(mut state) = Self::state_mut(&self.id) {
                        use crate::mirror::component::state::State;
                        state.on_serialize_sync_object (
                            network_behaviour_state.sync_object_dirty_bit,
                            writer,
                            initial,
                        );
                    }
                }
            });
            variable_deserialize_slot = Some(quote! {
                if let Some(mut state) = Self::state_mut(&self.id) {
                    use crate::mirror::component::state::State;
                    state.on_deserialize_sync_variable (reader,initial);
                }
            });
            object_deserialize_slot = Some(quote! {
                if let Some(mut network_behaviour_state) = crate::mirror::components::network_behaviour::NetworkBehaviour::state_mut(&self.id) {
                    if let Some(mut state) = Self::state_mut(&self.id) {
                        use crate::mirror::component::state::State;
                        state.on_deserialize_sync_object (
                            dirty_bit,
                            reader,
                            initial,
                        );
                    }
                }
            });
        }

        // component_deserialize_slot = Some(quote! {
        //     if let Some(mut network_behaviour_state) = crate::mirror::components::network_behaviour::NetworkBehaviour::state_mut(&self.id) {
        //         if let Some(mut state) = Self::state_mut(&self.id) {
        //             use crate::mirror::component::state::State;
        //             state.on_deserialize_sync_variable (
        //                 &mut network_behaviour_state.sync_var_dirty_bit,
        //                 reader,
        //                 initial,
        //             );
        //             state.on_deserialize_sync_object (
        //                 &mut network_behaviour_state.sync_object_dirty_bit,
        //                 reader,
        //                 initial,
        //             );
        //         }
        //     }
        // });
    }

    // ---------------------------------------------------------

    let mut instance_slot = quote! {
        #parent_instance_slot
        #state_instance_slot
        let instance = Self {
            #instance_fields
        };
        instance
    };
    // ---------------------------------------------------------

    item_struct.fields = Fields::Named(syn::FieldsNamed {
        brace_token: syn::token::Brace::default(),
        named,
    });

    let this_struct_private_mod_ident = format_ident!(
        "private_component_{}",
        struct_ident.to_string().to_snake_case().to_lowercase()
    );

    TokenStream::from(quote! {
        mod #this_struct_private_mod_ident {
            use super::*;

            #item_struct

            impl crate::mirror::component::component::Component for #struct_ident {
                fn new(
                    settings: &crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper,
                    obj_start_offset: &mut u8,
                    var_start_offset: &mut u8,
                ) -> Self
                where
                    Self: Sized,
                {
                    #instance_slot
                }
            }

            impl crate::mirror::component::component_basic::ComponentBasic for #struct_ident {
                fn id(&self) -> String {
                    self.id.clone()
                }

                fn parent(&self) -> Option<Box<dyn crate::mirror::component::component::Component>> {
                    #get_parent_slot
                }


                fn state_clear(&self) {
                    if let Some(parent) = self.parent() {
                        parent.state_clear();
                    }
                    #state_clear_slot
                }
            }

            #component_state_impl_slot





            impl crate::mirror::component::component_serializer::ComponentOnSerializer for #struct_ident {
                fn serialize_sync_objects(&self, writer: &mut crate::mirror::network_writer::NetworkWriter, initial: bool) {
                    use crate::mirror::component::component_basic::ComponentBasic;
                    if let Some(parent) = self.parent() {
                        parent.serialize_sync_objects(writer, initial);
                    }
                    #object_serialize_slot
                }

                fn serialize_sync_variables(&self, writer: &mut crate::mirror::network_writer::NetworkWriter, initial: bool) {
                    use crate::mirror::component::component_basic::ComponentBasic;
                    if let Some(parent) = self.parent() {
                        parent.serialize_sync_variables(writer, initial);
                    }
                    #variable_serialize_slot
                }
            }

            impl crate::mirror::component::component_deserializer::ComponentOnDeserializer for #struct_ident {
                fn deserialize_sync_objects(&self, reader: &mut crate::mirror::network_reader::NetworkReader, initial: bool) -> u64 {
                    use crate::mirror::component::component_basic::ComponentBasic;
                    let dirty_bit = if let Some(parent) = self.parent() {
                        parent.deserialize_sync_objects(reader, initial)
                    } else {
                        reader.read_blittable::<u64>()
                    };
                    #object_deserialize_slot

                    dirty_bit
                }

                fn deserialize_sync_variables(&self, reader: &mut crate::mirror::network_reader::NetworkReader, initial: bool) {
                    use crate::mirror::component::component_basic::ComponentBasic;
                    if let Some(parent) = self.parent() {
                        parent.deserialize_sync_variables(reader, initial);
                    }
                    #variable_deserialize_slot
                }
            }
        }

        pub use #this_struct_private_mod_ident::#struct_ident;

        #namespace_slot

        #this_component_state_trait_slot

    })
}
