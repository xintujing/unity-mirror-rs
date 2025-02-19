use crate::utils::write_to_file;
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

    let package = match attrs.package {
        None => String::new(),
        Some(mut package) => match package.chars().last().unwrap() {
            '.' | '+' | '\0' => package.clone(),
            _ => {
                package.push('.');
                package.clone()
            }
        },
    };

    let output = TokenStream::from(quote! {
        #input

        impl crate::mirror::namespace::Namespace for #strict_ident {

            fn get_namespace() -> &'static str {
                #namespace
            }

            fn get_package() -> &'static str {
                #package
            }
        }
    });

    // write_to_file("namespace", output.to_string());

    output
}
