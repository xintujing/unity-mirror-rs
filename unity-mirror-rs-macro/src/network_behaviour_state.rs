use proc_macro::TokenStream;
use syn::parse::Parse;
use syn::parse_macro_input;

pub(crate) fn handler(input: TokenStream) -> TokenStream {
    // DeriveInput
    let mut derive_input = parse_macro_input!(input as syn::DeriveInput);

    // struct 名
    let struct_ident = &derive_input.ident;

    // struct 的 fields
    let mut fields = match derive_input.data {
        syn::Data::Struct(ref mut data) => &mut data.fields,
        _ => panic!("Component can only be used in structures"),
    };

    // 遍历字段
    for field in &mut fields.iter_mut() {
        for attr in &field.attrs {
            if attr.path().is_ident("sync_variable") {
                // 修改字段的可见性
                field.vis = syn::Visibility::Inherited;
                break;
            }
        }
    }

    TokenStream::new()
}
