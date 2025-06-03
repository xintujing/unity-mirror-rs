use crate::namespace::NamespaceArgs;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::token::{Comma, Paren, Pub};
use syn::{parse_macro_input, Field, FieldMutability, Fields, Path, TypePath, Visibility};
//
// mod kw {
//     syn::custom_keyword!(state);
//     syn::custom_keyword!(parent);
//     syn::custom_keyword!(namespace);
// }

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
                        let _paren_token: Paren = syn::parenthesized!(content in input); // 捕获括号内的内容
                        if let Ok(path) = content.parse::<Path>() {
                            state = Some(path)
                        }
                    }
                    "parent" => {
                        let content;
                        let _paren_token: Paren = syn::parenthesized!(content in input); // 捕获括号内的内容
                        if let Ok(path) = content.parse::<Path>() {
                            parent = Some(path)
                        }
                    }
                    "namespace" => {
                        let content;
                        let _paren_token: Paren = syn::parenthesized!(content in input); // 捕获括号内的内容
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
    } = parse_macro_input!(attr as ComponentArgs);

    let mut item_struct = parse_macro_input!(item as syn::ItemStruct);

    if item_struct.fields.len() > 0 {
        // 组件本体字段必须为空
        return syn::Error::new(item_struct.span(), "component fields must be empty")
            .to_compile_error()
            .into();
    }

    let struct_ident = &item_struct.ident;

    // 插槽
    let mut self_slot = vec![];
    let mut self_state_clear_slot = quote! {};
    let mut self_parent_slot = quote! { None };
    let mut parent_on_serialize_slot = quote! {};
    let mut parent_on_deserialize_slot = quote! {};

    let mut serialize_slot = quote! {};
    let mut deserialize_slot = quote! {};

    if let Some(namespace) = &namespace {
        let namespace = namespace.get_namespace(struct_ident);

        self_slot.push(quote! {
            impl crate::commons::namespace::Namespace for #struct_ident{
                fn get_namespace() -> &'static str {
                    #namespace
                }
            }
        });
    }

    if let Some(parent_path) = &parent {
        if let Fields::Named(fields_named) = &mut item_struct.fields {
            fields_named.named.push(Field {
                attrs: vec![],
                ident: Some(format_ident!("parent")),
                colon_token: None,

                ty: syn::Type::Path(TypePath {
                    qself: None,
                    path: parent_path.clone(),
                }),
                vis: Visibility::Public(Pub(Span::call_site())),
                mutability: FieldMutability::None,
            });
        }

        self_parent_slot = quote! {
           Some(Box::new(self.parent.clone()))
        };

        parent_on_serialize_slot = quote! {
            self.parent.on_serialize(writer, initial);
        };
        parent_on_deserialize_slot = quote! {
            self.parent.on_deserialize(reader, initial);
        };
    };

    if let Some(state_path) = &state {
        if let Fields::Named(fields_named) = &mut item_struct.fields {
            fields_named.named.push(Field {
                attrs: vec![],
                ident: Some(format_ident!("id")),
                colon_token: None,
                ty: syn::Type::Path(TypePath {
                    qself: None,
                    path: syn::parse_str("String").unwrap(),
                }),
                vis: Visibility::Public(Pub(Span::call_site())),
                mutability: FieldMutability::None,
            });
        }

        let instance_slot = match &parent {
            None => {
                quote! {
                    let id = uuid::Uuid::new_v4().to_string();
                    let self_instance = Self {
                        id,
                    };
                }
            }
            Some(parent_path) => {
                quote! {
                    let parent_instance = #parent_path::new(&settings);
                    use crate::mirror::component::component_basic::ComponentBasic;
                    let id = parent_instance.id().clone();
                    let self_instance = Self {
                        id,
                        parent: parent_instance,
                    };
                }
            }
        };

        self_slot.push(quote! {

            const _: fn() = || {
                fn check<T>()
                where
                    T: crate::mirror::component::state::State,
                {}
                check::<#state_path>();
            };

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

            impl crate::mirror::component::component::Component for #struct_ident {
                fn new(settings: &crate::metadata_settings::mirror::network_behaviour_settings::MetadataNetworkBehaviourWrapper) -> Self
                where
                    Self: Sized,
                {
                    let state = #state_path::initialize(&settings);

                    #instance_slot

                    #state_path::new(&self_instance.id, state);

                    self_instance
                }
            }
        });

        self_state_clear_slot = quote! {
            if #state_path::remove(&self.id).is_none() {
                use crate::commons::namespace::Namespace;
                println!("[{}]{} component status deletion failed ", self.id, Self::get_namespace());
            }
        };

        serialize_slot = quote! {
            if let Some(mut network_behaviour_state) = NetworkBehaviour::state_mut(&self.id) {
                let index = network_behaviour_state.index;
                if let Some(mut state) = Self::state_mut(&self.id) {
                    use crate::mirror::component::state::State;
                    state.on_serialize_sync_variable (
                        index,
                        &mut network_behaviour_state.sync_var_dirty_bit,
                        writer,
                        initial,
                    );
                    state.on_serialize_sync_object (
                        index,
                        &mut network_behaviour_state.sync_object_dirty_bit,
                        writer,
                        initial,
                    );
                }
            }
        };

        deserialize_slot = quote! {
            if let Some(mut network_behaviour_state) = NetworkBehaviour::state_mut(&self.id) {
                let index = network_behaviour_state.index;
                if let Some(mut state) = Self::state_mut(&self.id) {
                    use crate::mirror::component::state::State;
                    state.on_deserialize_sync_variable (
                        index,
                        &mut network_behaviour_state.sync_var_dirty_bit,
                        reader,
                        initial,
                    );
                    state.on_deserialize_sync_object (
                        index,
                        &mut network_behaviour_state.sync_object_dirty_bit,
                        reader,
                        initial,
                    );
                }
            }
        };
    };

    TokenStream::from(quote! {
        #[derive(Clone)]
        #item_struct

        #(
            #self_slot
        )*


        impl crate::mirror::component::component_basic::ComponentBasic for #struct_ident {
            fn id(&self) -> String {
                self.id.clone()
            }

            fn parent(&self) -> Option<Box<dyn crate::mirror::component::component::Component>> {
                 #self_parent_slot
            }


            fn state_clear(&self) {
                if let Some(parent) = self.parent() {
                    parent.state_clear();
                }
                #self_state_clear_slot
            }
        }

        impl crate::mirror::component::component_serializer::ComponentOnSerializer for #struct_ident {
            fn on_serialize(&self, writer: &mut crate::mirror::network_writer::NetworkWriter, initial: bool) {
                #parent_on_serialize_slot
                #serialize_slot
            }
        }

        impl crate::mirror::component::component_deserializer::ComponentOnDeserializer for #struct_ident {
            fn on_deserialize(&self, reader: &mut crate::mirror::network_reader::NetworkReader, initial: bool) {
                #parent_on_deserialize_slot
                #deserialize_slot
            }
        }

    })
}
