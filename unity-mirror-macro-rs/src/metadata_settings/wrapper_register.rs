#![allow(dead_code)]
use crate::utils::string_case::StringCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Path, Token};

// 输入解析部分
struct WrapperInput {
    struct_path: Path,
    as_token: Token![as],
    wrapper_path: Path,
}
impl Parse for WrapperInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(WrapperInput {
            struct_path: input.parse()?,
            as_token: input.parse()?,
            wrapper_path: input.parse()?,
        })
    }
}
pub(crate) fn handler(input: TokenStream) -> TokenStream {
    // 解析输入
    let WrapperInput {
        struct_path,
        wrapper_path,
        ..
    } = parse_macro_input!(input as WrapperInput);

    let register_ident = format_ident!(
        "__{}_register",
        struct_path.get_ident().unwrap().to_string().to_camel_case()
    );

    // 生成代码
    let expanded = quote! {
        impl Settings for #struct_path {}
        #[ctor::ctor]
        fn #register_ident(){
            #wrapper_path::register::<#struct_path>();
        }
    };
    // 返回生成的代码块
    TokenStream::from(expanded)
}
