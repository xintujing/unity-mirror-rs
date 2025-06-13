use crate::utils::random_string::random_string;
use crate::utils::string_case::StringCase;
use proc_macro::TokenStream;
use quote::format_ident;
use syn::Field;

pub(crate) fn handler(item: TokenStream) -> TokenStream {
    let mut input = syn::parse_macro_input!(item as syn::ItemStruct);
    let struct_ident = &input.ident;

    // 扩展字段
    let mut ext_fields: Vec<Field> = Vec::new();
    ext_fields.push(syn::parse_quote! {
        weak: RevelWeak<Box<Self>>
    });
    ext_fields.push(syn::parse_quote! {
        on_server_authenticated: SelfMutAction<(
            RevelArc<Box<NetworkConnectionToClient>>,),()>
    });

    // 检查结构体的字段是否为命名字段（即标准的 struct，而不是 tuple struct 或 unit struct）
    if let syn::Fields::Named(ref mut fields_named) = input.fields {
        // 将新字段插入到字段集合中
        fields_named.named.extend(ext_fields);
    }

    let register_cotr_fn_ident = format_ident!(
        "register_{}_{}",
        struct_ident.to_string().to_snake_case().to_lowercase(),
        random_string(5).to_lowercase()
    );

    let out = quote::quote! {

        // 修改后的 struct定义
        #input

        // impl of the Authenticator trait
        impl AuthenticatorBase for #struct_ident {
            fn set_weak_self(
                &mut self,
                weak_self: RevelWeak<Box<dyn Authenticator>>,
            ) {
                if let Some(weak_self) = weak_self.downcast::<Self>() {
                    self.weak = weak_self.clone();
                }
            }
            fn set_on_server_authenticated(
                &mut self,
                event: SelfMutAction<(RevelArc<Box<NetworkConnectionToClient>>,),(),>,
            ) {
                self.on_server_authenticated = event;
            }

            fn on_server_authenticated(
                &self,
            ) -> &SelfMutAction<(
                RevelArc<Box<NetworkConnectionToClient>,>,),(),> {
                &self.on_server_authenticated
            }
        }

        #[ctor::ctor]
        #[inline]
        fn #register_cotr_fn_ident() {
            AuthenticatorFactory::register::<#struct_ident>();
        }
    };

    out.into()
}
