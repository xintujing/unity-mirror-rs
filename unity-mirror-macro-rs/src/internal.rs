use crate::utils::attribute_contain::VecAttributeExpand;
use crate::utils::string_case::StringCase;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Token, Type, parse_macro_input, parse_quote};

// mod kw {
//     syn::custom_keyword!(set);
// }
//
// struct FieldInternalArgs {
//     set: bool,
// }

// impl Parse for FieldInternalArgs {
//     fn parse(input: ParseStream) -> syn::Result<Self> {
//         let mut set = false;
//         while !input.is_empty() {
//             if input.peek(kw::set) {
//                 input.parse::<kw::set>()?;
//                 set = true;
//             } else if input.peek(Token![,]) {
//                 input.parse::<Token![,]>()?;
//                 if input.is_empty() {
//                     break;
//                 }
//             } else {
//                 return Err(input.error("Unexpected argument"));
//             }
//         }
//         Ok(Self { set })
//     }
// }

pub(crate) fn handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(item as syn::ItemStruct);
    item_struct.attrs.push(parse_quote! {
        #[derive(unity_mirror_macro_rs::Internal)]
    });

    let mut internal_set_structs = vec![];
    for field in item_struct.fields.iter_mut() {
        if let Some(field_ident) = &field.ident {
            if let Type::Path(type_path) = &field.ty {
                let field_type_path = &type_path.path;
                if field.attrs.contain("set") {
                    let field_box_struct_ident =
                        format_ident!("{}Box", field_ident.to_string().to_camel_case());

                    // let mut deref_mut_slot = Some(quote! {
                    //     impl std::ops::DerefMut for #field_box_struct_ident {
                    //         fn deref_mut(&mut self) -> &mut Self::Target {
                    //             &mut self.0
                    //         }
                    //     }
                    // });

                    // if !set {
                    //     deref_mut_slot = None;
                    // }

                    internal_set_structs.push(quote! {
                        pub struct #field_box_struct_ident(#field_type_path);

                        impl std::ops::Deref for #field_box_struct_ident {
                            type Target = #field_type_path;

                            fn deref(&self) -> &Self::Target {
                                &self.0
                            }
                        }

                        impl #field_box_struct_ident {
                            pub(super) fn set(&mut self,value: #field_type_path) {
                                self.0 = value;
                            }
                        }

                        // #deref_mut_slot
                    });

                    field.ty = parse_quote! {
                        fileds_box::#field_box_struct_ident
                    };
                }
            }
        }
    }

    TokenStream::from(quote! {

        mod fileds_box {
            use super::*;
            #(
                #internal_set_structs
            )*
        }

        #item_struct

    })
}
pub(crate) fn derive_handler(item: TokenStream) -> TokenStream {
    TokenStream::from(quote! {})
}
