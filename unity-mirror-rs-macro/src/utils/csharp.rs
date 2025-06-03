use quote::{format_ident, quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::{
    FnArg, GenericArgument, Path, PathArguments, PathSegment, Token, Type, TypeArray, TypePath,
    TypeReference, TypeSlice,
};

pub(crate) fn to_csharp_function_inputs(
    inputs: Punctuated<FnArg, Token![,]>,
) -> proc_macro2::TokenStream {
    let mut params: Vec<proc_macro2::TokenStream> = Vec::new();
    for arg in inputs.iter().filter_map(|input| match input {
        FnArg::Typed(pat_type) => Some(pat_type),
        _ => None,
    }) {
        match type_to_csharp(&arg.ty) {
            None => {
                let param_type_name = arg.ty.to_token_stream().to_string();
                let param_type = format_ident!("{}", param_type_name);
                params.push(quote! {
                    {
                        use crate::mirror::namespace::Namespace;
                        #param_type::get_full_path()
                    }
                });
            }
            Some(r#type) => {
                params.push(quote! { #r#type.to_string() });
            }
        }
    }

    if params.len() > 0 {
        quote! {
            vec![#(#params,)*].join(",")
        }
    } else {
        quote! {
            ""
        }
    }
}

pub fn type_to_csharp(r#type: &Type) -> Option<String> {
    match r#type {
        Type::Reference(TypeReference { elem, .. }) => type_to_csharp(elem),
        Type::Path(TypePath { path, .. }) => {
            let full_type = get_full_type(path);

            match full_type.as_str() {
                "i8" => Some("System.SByte".to_string()),
                "i16" => Some("System.Int16".to_string()),
                "i32" => Some("System.Int32".to_string()),
                "i64" => Some("System.Int64".to_string()),
                "u8" => Some("System.Byte".to_string()),
                "u16" => Some("System.UInt16".to_string()),
                "u32" => Some("System.UInt32".to_string()),
                "u64" => Some("System.UInt64".to_string()),
                "f32" => Some("System.Single".to_string()),
                "f64" => Some("System.Double".to_string()),
                "bool" => Some("System.Boolean".to_string()),
                "char" => Some("System.Char".to_string()),
                "&str" | "String" => Some("System.String".to_string()),
                "isize" => Some("System.IntPtr".to_string()),
                "usize" => Some("System.UIntPtr".to_string()),
                "crate::mirror::NetworkConnection" => Some("Mirror.NetworkConnectionToClient".to_string()),
                "nalgebra::Vector3" | "Vector3" => Some("UnityEngine.Vector3".to_string()),
                "nalgebra::Quaternion" | "Quaternion" => Some("UnityEngine.Quaternion".to_string()),
                "Vec" => process_generic_type(full_type, path, "System.Collections.Generic.List`1"),
                "RevelArc" | "RevelWeak" => {
                    if let PathSegment {
                        arguments: PathArguments::AngleBracketed(args),
                        ..
                    } = path.segments.last().unwrap() {
                        if let GenericArgument::Type(ref ty) = args.args[0] {
                            // panic!("RevelArc or RevelWeak cannot be used as a generic argument. {}", ty.to_token_stream());
                            return type_to_csharp(ty);
                        }
                    }
                    None
                }
                _ => None,
            }
        }
        Type::Array(TypeArray { elem, .. }) | Type::Slice(TypeSlice { elem, .. }) => {
            Some(format!("{}[]", type_to_csharp(elem)?))
        }
        _ => None,
    }
}

fn get_full_type(path: &Path) -> String {
    path.segments
        .iter()
        .map(|seg| seg.ident.to_string()) // 提取每个段的标识符并转换为字符串
        .reduce(|acc, item| acc + "::" + &item) // 直接连接字符串，避免使用 Vec
        .unwrap_or_default() // 如果路径为空，则返回一个空字符串
}

fn process_generic_type(_type_name: String, path: &Path, csharp_prefix: &str) -> Option<String> {
    if let PathSegment {
        arguments: PathArguments::AngleBracketed(args),
        ..
    } = path.segments.last().unwrap()
    {
        let generics: Vec<String> = args
            .args
            .iter()
            .filter_map(|arg| match arg {
                GenericArgument::Type(ty) => type_to_csharp(ty),
                _ => None,
            })
            .collect();

        if generics.len() != args.args.len() {
            return None; // If any generic argument was None, we can't proceed
        }

        let generics_str = generics.join(",");
        Some(format!("{}<{}>", csharp_prefix, generics_str))
    } else {
        None
    }
}
