// use proc_macro::TokenStream;
// use quote::quote;
// use syn::{ImplItem, Type, parse_macro_input, parse_quote};
//
// pub(crate) fn handler(attr: TokenStream, item: TokenStream) -> TokenStream {
//     let mut item_impl = parse_macro_input!(item as syn::ItemImpl);
//     let impl_struct_ident = match &*item_impl.self_ty {
//         Type::Path(path) => path.path.segments.last().unwrap().ident.clone(),
//         _ => panic!("unsupported type for impl block"),
//     };
//     let impl_struct_name = impl_struct_ident.to_string();
//     item_impl.items.insert(
//         0,
//         parse_quote!(
//             const COMPONENT_NAME: &'static str = #impl_struct_name;
//         ),
//     );
//
//
//     TokenStream::from(quote! {
//         #item_impl
//     })
// }
