use crate::NamespaceArgs;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

pub(crate) fn namespace_attribute_handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let attrs = parse_macro_input!(attr as NamespaceArgs);

    let strict_ident = &input.ident;

    let namespace = match attrs.value {
        None => panic!("namespace attribute must have a value"),
        Some(value) => value.clone(),
    };

    let full_path = match attrs.full_path {
        None => String::new(),
        Some(mut full_path) => match full_path.chars().last().unwrap() {
            '.' | '+' | '\0' => full_path.clone(),
            _ => {
                full_path.push('.');
                full_path.clone()
            }
        },
    };

    let output = TokenStream::from(quote! {
        #input

        impl crate::mirror::namespace::Namespace for #strict_ident {

            fn get_namespace() -> &'static str {
                #namespace
            }

            fn get_prefix() -> &'static str {
                #full_path
            }
        }
    });

    // write_to_file("namespace", output.to_string());

    output
}
