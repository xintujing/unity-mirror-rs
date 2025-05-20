use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::token::Comma;
use syn::{parse_macro_input, GenericArgument, PathArguments};

struct SyncedArgs {
    pub custom: bool,
}

impl Parse for SyncedArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut synced_args = SyncedArgs { custom: false };

        while !input.is_empty() {
            {
                match input.parse::<Ident>()?.to_string().as_str() {
                    "custom" => synced_args.custom = true,
                    _ => {}
                }
            }
            let _ = input.parse::<Comma>();
        }

        Ok(synced_args)
    }
}

pub(crate) fn handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let SyncedArgs { custom } = parse_macro_input!(attr as SyncedArgs);
    let item_struct = parse_macro_input!(item as syn::ItemStruct);

    let struct_ident = &item_struct.ident;

    let mut sync_variable_index = 0;
    let mut sync_object_index = 0;

    let mut sync_variable_slots = vec![];
    let mut sync_variable_field_idents = vec![];
    let mut sync_object_slots = vec![];
    let mut sync_object_field_idents = vec![];

    for field in item_struct.fields.iter() {
        // 判断字段类型是否为 SyncVariable<X>
        if let syn::Type::Path(type_path) = &field.ty {
            if type_path.path.segments.len() > 0 {
                let segment = &type_path.path.segments[0];

                let mut general_type_path = None;
                if let PathArguments::AngleBracketed(angle_bracketed) = &segment.arguments {
                    if let Some(generic_argument) = angle_bracketed.args.first() {
                        if let GenericArgument::Type(generic_argument_type) = &generic_argument {
                            general_type_path = Some(generic_argument_type.clone());
                        }
                    }
                }

                let field_type_ident = &segment.ident;
                // 获取字段名称
                let field_name = &field.ident.clone().unwrap();
                let field_name_new_ident = format_ident!("{}_new", field_name);
                match segment.ident.to_string().as_str() {
                    "SyncVariable" => {
                        sync_variable_slots.push(quote! {
                                fn #field_name_new_ident(default_value: #general_type_path) -> #field_type_ident<#general_type_path> {
                                    #field_type_ident::new(default_value, None, #sync_variable_index as u8)
                                }
                        });

                        sync_variable_field_idents.push(field_name.clone());

                        sync_variable_index += 1;
                    }
                    "SyncObject" => {
                        sync_object_slots.push(quote! {
                                fn #field_name_new_ident(default_value: #general_type_path) -> #field_type_ident<#general_type_path> {
                                    #field_type_ident::new(default_value, None, #sync_object_index as u8)
                                }
                        });

                        sync_object_field_idents.push(field_name.clone());

                        sync_object_index += 1;
                    }

                    &_ => {}
                }
            }
        }
    }

    let mut state_trait_impl_slot = quote! {};

    if !custom {
        let component_state_ident = format_ident!("Component{struct_ident}");

        state_trait_impl_slot = quote! {
            impl #component_state_ident for #struct_ident {
                fn serialize_sync_variables(
                    &mut self,
                    dirty_bit: &mut u64,
                    index_offset: u8,
                    writer: &mut crate::mirror::network_writer::NetworkWriter,
                    initial: bool,
                ) {
                    #(
                        self.#sync_variable_field_idents.serialize(dirty_bit, index_offset, writer, initial);
                    )*
                }

                fn deserialize_sync_variables(
                    &mut self,
                    dirty_bit: &mut u64,
                    index_offset: u8,
                    reader: &mut crate::mirror::network_reader::NetworkReader,
                    initial: bool,
                ) {
                    #(
                        self.#sync_variable_field_idents.deserialize(dirty_bit, index_offset, reader, initial);
                    )*
                }

                fn serialize_sync_objects(
                    &mut self,
                    dirty_bit: &mut u64,
                    index_offset: u8,
                    writer: &mut crate::mirror::network_writer::NetworkWriter,
                    initial: bool,
                ) {
                    #(
                        self.#sync_object_field_idents.serialize(dirty_bit, index_offset, writer, initial);
                    )*
                }

                fn deserialize_sync_objects(
                    &mut self,
                    dirty_bit: &mut u64,
                    index_offset: u8,
                    reader: &mut crate::mirror::network_reader::NetworkReader,
                    initial: bool,
                ) {
                    #(
                        self.#sync_object_field_idents.deserialize(dirty_bit, index_offset, reader, initial);
                    )*
                }
            }
        }
    }

    TokenStream::from(quote::quote! {

        #item_struct

        impl #struct_ident {
            #(#sync_variable_slots)*
            #(#sync_object_slots)*
        }

        #state_trait_impl_slot

    })
}
