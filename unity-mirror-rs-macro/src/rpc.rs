use crate::component::ComponentBlock;
use crate::utils::generate_unique_string;
use quote::{format_ident, quote};
use syn::{Attribute, Signature};

#[allow(unused)]
pub(crate) fn rpc_handler(
    namespace: &str,
    struct_ident: &proc_macro2::Ident,
    attr: &Attribute,
    sign: &Signature,
    comb: &mut ComponentBlock,
) {
    let str = format_ident!("{}", generate_unique_string(5).to_lowercase());

    comb.add_invoke_user_code(quote! {
        fn #str(){
            println!("hello");
        }
    })
}
