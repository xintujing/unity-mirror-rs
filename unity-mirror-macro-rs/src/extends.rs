use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_quote, Fields, Path};

struct ExtendsArgs {
    parent: Path,
}

impl Parse for ExtendsArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let parent = match input.parse() {
            Ok(path) => path,
            Err(_) => {
                return Err(input.error("#[extends] expects a single path argument"));
            }
        };
        Ok(ExtendsArgs { parent })
    }
}

pub(crate) fn handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ExtendsArgs { parent } = syn::parse_macro_input!(attr as ExtendsArgs);
    let mut item_struct = syn::parse_macro_input!(item as syn::ItemStruct);
    let struct_ident = &item_struct.ident;

    match &mut item_struct.fields {
        Fields::Named(fields_named) => fields_named.named.insert(
            0,
            parse_quote! {
                pub parent: RevelArc<Box<#parent>>
            },
        ),
        _ => {
            return syn::Error::new_spanned(
                item_struct,
                "#[extends] only supports structs with named fields",
            )
                .to_compile_error()
                .into();
        }
    }

    TokenStream::from(quote! {
        #item_struct

        impl std::ops::Deref for #struct_ident {
            type Target = #parent;
            fn deref(&self) -> &Self::Target {
                &self.parent
            }
        }

        impl std::ops::DerefMut for #struct_ident {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.parent
            }
        }
    })
}
