use crate::component::ComponentBlock;
use crate::utils::StringUtils;
use crate::{utils, CommandArgs};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use std::str::FromStr;
use syn::{Attribute, FnArg, Signature};

pub(crate) fn command_handler(
    namespace: &str,
    struct_ident: &proc_macro2::Ident,
    attr: &Attribute,
    sign: &Signature,
    comb: &mut ComponentBlock,
) {
    let method_ident = &sign.ident;
    // let mut method_params = vec![];
    // for arg in sign.inputs.iter() {
    //     match arg {
    //         FnArg::Typed(pat_type) => {
    //             let param_type = utils::type_to_csharp(&pat_type.ty).unwrap_or("".to_string());
    //             method_params.push(param_type);
    //         }
    //         _ => {}
    //     }
    // }

    // // 解析属性参数
    let mut authority = match attr.parse_args::<CommandArgs>() {
        Ok(command_args) => {
            // function_full_name = command_args.function_full_name;
            command_args.authority.is_some()
                && bool::from_str(&command_args.authority.unwrap()).unwrap_or(false)
        }
        Err(_) => false,
    };

    let mut params: Vec<TokenStream> = Vec::new();

    // 遍历方法的所有参数
    for arg in sign.inputs.iter().filter_map(|input| match input {
        FnArg::Typed(pat_type) => Some(pat_type),
        _ => None,
    }) {
        match utils::type_to_csharp(&arg.ty) {
            None => {
                let param_type_name = arg.ty.to_token_stream().to_string();

                // if param_type_name.contains("Vector3") {
                //     panic!("Asdeeee")
                // }
                let param_type = format_ident!("{}", param_type_name);

                // nalgebra :: Vector3 < f64 >
                // Vector3::<f64>

                params.push(
                    quote! {&format!("{}.{}{}",#param_type::get_namespace(),#param_type::get_package(), #param_type_name)},
                );
            }
            Some(r#type) => {
                let param_type_name = arg.ty.to_token_stream().to_string();
                // if param_type_name.contains("Vector3") {
                //     panic!("Asd {}", r#type)
                // }
                params.push(quote! { #r#type });
            }
        }
    }

    let invoke_user_code_method_name =
        format_ident!("__invoke_user_code__{}", method_ident.to_string());

    let params_read_blittable: Vec<TokenStream> = (0..params.len())
        .map(|_| quote! {reader.read_blittable()})
        .collect();

    comb.add_invoke_user_code(quote! {
        fn #invoke_user_code_method_name(
            obj: &mut crate::mirror::core::network_behaviour::NetworkBehaviourType,
            reader: &mut crate::mirror::core::network_reader::NetworkReader,
            sender_connection: &mut crate::mirror::core::network_connection_to_client::NetworkConnectionToClient
        ) {
            println!("invoke_user_code: {}", stringify!(#method_ident));
            // TODO: if network_server.active
            obj.as_any_mut().downcast_mut::<Self>().unwrap().#method_ident(#(#params_read_blittable,)*);
        }
    });

    let full_path_str = format!(
        "System.Void {}.{}::{}({{}})",
        namespace, // 固定的命名空间
        struct_ident.to_string(),
        sign.ident.to_string().to_camel_case()
    );

    comb.add_register(quote! {
        crate::mirror::core::remote_calls::RemoteProcedureCalls::register_command_delegate::<#struct_ident>(
            &format!(#full_path_str, vec![#(#params,)*].join(",")),
            #struct_ident::#invoke_user_code_method_name,
            #authority,
        );
    });
}
