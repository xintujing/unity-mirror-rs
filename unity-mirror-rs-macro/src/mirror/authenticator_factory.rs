use crate::utils::random_string::random_string;
use crate::utils::string_case::StringCase;
use proc_macro::TokenStream;
use quote::format_ident;
use syn::Data;

pub(crate) fn handler(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);
    let struct_ident = &input.ident;

    match input.data {
        Data::Struct(_) => {}
        _ => {
            return syn::Error::new(
                input.ident.span(),
                "NetworkManagerFactory can only be derived for structs",
            )
            .to_compile_error()
            .into();
        }
    }

    let register_cotr_fn_ident = format_ident!(
        "register_{}_{}",
        struct_ident.to_string().to_snake_case().to_lowercase(),
        random_string(5).to_lowercase()
    );

    let out = quote::quote! {
        #[ctor::ctor]
        #[inline]
        fn #register_cotr_fn_ident() {
            crate::mirror::AuthenticatorFactory::register::<#struct_ident>();
        }
    };

    out.into()
}
