use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::token::Comma;
use syn::{Fields, Path, parse_macro_input, parse_quote};

struct ParentArgs {
    pub value: Path,
    pub callbacks: Option<Path>,
}

impl Parse for ParentArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let value = match input.parse::<Path>() {
            Ok(path) => {
                let _ = input.parse::<Comma>();
                path
            }
            Err(_) => {
                return Err(syn::Error::new(
                    input.span(),
                    "Expected a path for parent argument",
                ));
            }
        };

        let mut callbacks = None;
        while !input.is_empty() {
            match input.parse::<Ident>()?.to_string().as_str() {
                "callbacks" => {
                    input.parse::<syn::Token![=]>()?;
                    callbacks = input.parse().ok();
                }
                _ => {}
            }
            let _ = input.parse::<Comma>();
        }
        Ok(Self { value, callbacks })
    }
}

struct NetworkManagerArgs {
    pub parent: Option<ParentArgs>,
}

impl Parse for NetworkManagerArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut parent = None;

        while !input.is_empty() {
            match input.parse::<Ident>()?.to_string().as_str() {
                "parent" => {
                    let content;
                    syn::parenthesized!(content in input);
                    parent = Some(content.parse::<ParentArgs>()?);
                }
                _ => {}
            }
            let _ = input.parse::<Comma>();
        }

        Ok(NetworkManagerArgs { parent })
    }
}

pub(crate) fn handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let NetworkManagerArgs { parent } = parse_macro_input!(attr as NetworkManagerArgs);
    let mut item_struct = parse_macro_input!(item as syn::ItemStruct);

    item_struct.attrs.push(parse_quote! {
        #[derive(Default, NetworkManagerFactory)]
    });

    let struct_ident = &item_struct.ident;

    let mut parent_deref_slot = None;
    let mut parent_instance = None;
    let mut instance_field_slot = None;
    let mut instance_to_trait_slot = quote! {
        let arc_instance =  crate::commons::revel_arc::RevelArc::new(
            box_instance as Box<dyn crate::mirror::TNetworkManager>
        );
        if let Some(weak_instance) = arc_instance.downgrade().downcast::<Self>() {
            if let Some(real_instance) = weak_instance.get() {
                real_instance.initialize(metadata);
                real_instance.weak = weak_instance.clone();
            }
        }
    };

    match &mut item_struct.fields {
        Fields::Named(fields_named) => {
            fields_named.named.push(parse_quote! {
                game_object: crate::commons::revel_weak::RevelWeak<crate::unity_engine::GameObject>
            });
            fields_named.named.push(parse_quote! {
                weak: crate::commons::revel_weak::RevelWeak<Box<Self>>
            });
        }
        _ => {
            return syn::Error::new_spanned(
                item_struct.fields,
                "NetworkManager can only be used on structs with named fields",
            )
            .to_compile_error()
            .into();
        }
    };

    if let Some(parent) = &parent {
        let parent_path = &parent.value;

        if let Fields::Named(fields_named) = &mut item_struct.fields {
            fields_named.named.push(parse_quote! {
                parent: crate::commons::revel_weak::RevelWeak<Box<#parent_path>>
            })
        }

        parent_deref_slot = Some(quote! {
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
        });

        parent_instance = Some(quote! {
            use crate::mirror::NetworkManagerInstance;
            let parent = #parent_path::instance(weak_game_object.clone(), metadata);
            instances.extend(parent.clone());
        });

        instance_field_slot = Some(quote! {
            if let Some((arc_parent, _)) = parent.last() {
                instance.parent = arc_parent.downgrade().downcast::<NetworkManager>().unwrap().clone();
            }else{
                println!("Mirror: NetworkManager {} parent not found", stringify!(#struct_ident));
            }
        });

        if let Some(callbacks_path) = &parent.callbacks {
            instance_to_trait_slot = quote! {
                let arc_instance = crate::commons::revel_arc::RevelArc::new(
                    box_instance as Box<dyn #callbacks_path> as Box<dyn crate::mirror::TNetworkManager>,
                );

                let instance = unsafe {
                    &*(&arc_instance as *const dyn Any as *const crate::commons::revel_arc::RevelArc<Box<dyn #callbacks_path>>)
                };

                if let Some(weak_instance) = arc_instance.downgrade().downcast::<Self>() {
                    if let Some(real_instance) = weak_instance.get() {
                        if let Some(parent) = real_instance.parent.get() {
                            parent.set_virtual_trait(instance.downgrade())
                        }
                        real_instance.weak = weak_instance.clone();
                        real_instance.initialize(metadata);
                    }
                }
            }
        }
    }

    let struct_initialize_trait_ident = format_ident!("{}Initialize", struct_ident);

    TokenStream::from(quote! {
        #item_struct

        trait #struct_initialize_trait_ident {
            fn initialize(
                &mut self,
                metadata: &crate::metadata_settings::mirror::metadata_network_manager::MetadataNetworkManagerWrapper,
            );
        }

        const _: fn() = || {
            fn check<T>()
            where
                T: #struct_initialize_trait_ident,
            {}
            check::<#struct_ident>();
        };

        impl crate::mirror::TNetworkManager for #struct_ident {}

        #parent_deref_slot

        impl crate::mirror::NetworkManagerInstance for #struct_ident {
            fn instance(
                weak_game_object: crate::commons::revel_weak::RevelWeak<crate::unity_engine::GameObject>,
                metadata: &crate::metadata_settings::mirror::metadata_network_manager::MetadataNetworkManagerWrapper
            )
                -> Vec<(crate::commons::revel_arc::RevelArc<Box<dyn crate::mirror::TNetworkManager>>, std::any::TypeId)>
            where
                Self: Sized,
            {
                let mut instances = vec![];

                #parent_instance

                let mut instance = Self::default();

                instance.game_object = weak_game_object.clone();

                let type_id = std::any::TypeId::of::<#struct_ident>();

                #instance_field_slot

                let box_instance = Box::new(instance);

                #instance_to_trait_slot
                // #parent_callbacks_slot

                // let arc_instance =  crate::commons::revel_arc::RevelArc::new(
                //     box_instance as Box<dyn crate::mirror::TNetworkManager>
                // );

                // let weka_instance = arc_instance.downgrade().downcast::<Self>().unwrap().clone();
                // //
                //
                // if let Some(get_instance) = weka_instance.get() {
                //     get_instance.self_weak = weka_instance.clone();
                //     get_instance.initialize(metadata);
                // }

                // arc_instance.self_weak = weka_instance;

                instances.push((
                    arc_instance,
                    type_id,
                ));

                instances
            }
        }

    })
}
