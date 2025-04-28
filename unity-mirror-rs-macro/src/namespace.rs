use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Expr, ItemStruct, Lit, MetaNameValue};

pub(crate) struct NamespaceArgs {
    value: Option<String>,
    rename: Option<String>,
}

impl NamespaceArgs {
    pub(crate) fn get_namespace(&self, struct_ident: &syn::Ident) -> String {
        let current_name = if let Some(rename) = &self.rename {
            rename
        } else {
            &format!("{}", struct_ident)
        };

        match &self.value {
            None => format!("{}", current_name),
            Some(value) => format!("{}.{}", value, current_name),
        }
    }
}

impl Parse for NamespaceArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut value = None;
        let mut rename = None;

        if input.is_empty() {
            return Ok(NamespaceArgs { value, rename });
        }

        if let Expr::Lit(expr_lit) = input.parse::<Expr>()? {
            if let Lit::Str(lit_str) = &expr_lit.lit {
                value = Some(lit_str.value())
            } else {
                return Err(syn::Error::new_spanned(
                    expr_lit,
                    "value must be a string literal",
                ));
            }
        }

        if !input.is_empty() {
            let _ = input.parse::<syn::Token![,]>()?;

            if let Ok(MetaNameValue { path, value, .. }) = input.parse() {
                if path.is_ident("rename") {
                    if let Expr::Lit(lit) = value {
                        if let Lit::Str(lit_str) = &lit.lit {
                            rename = Some(lit_str.value())
                        } else {
                            return Err(syn::Error::new_spanned(
                                path,
                                "rename must be a string literal",
                            ));
                        }
                    }
                }
            }
        }

        Ok(NamespaceArgs { value, rename })
    }
}

pub(crate) fn handler(attr: TokenStream, input: TokenStream) -> TokenStream {
    let namespace_args = parse_macro_input!(attr as NamespaceArgs);
    let item_struct = parse_macro_input!(input as ItemStruct);
    let struct_ident = &item_struct.ident;

    let namespace = namespace_args.get_namespace(struct_ident);

    quote! {
        #item_struct

        impl crate::commons::namespace::Namespace for #struct_ident {
            fn get_namespace() -> &'static str {
                #namespace
            }
        }
    }
    .into()
}
