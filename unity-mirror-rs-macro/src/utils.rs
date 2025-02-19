use std::time::SystemTime;
use syn::{
    GenericArgument, Path, PathArguments, PathSegment, Type, TypeArray, TypePath, TypeReference,
    TypeSlice,
};

pub trait StringUtils {
    fn to_snake_case(&self) -> String;
    fn to_camel_case(&self) -> String;
}

impl StringUtils for str {
    fn to_snake_case(&self) -> String {
        to_snake_case(self)
    }

    fn to_camel_case(&self) -> String {
        to_camel_case(self)
    }
}

impl StringUtils for String {
    fn to_snake_case(&self) -> String {
        to_snake_case(self)
    }

    fn to_camel_case(&self) -> String {
        to_camel_case(self)
    }
}

fn to_snake_case(val: &str) -> String {
    let mut result = String::new();
    for (i, c) in val.chars().enumerate() {
        if i > 0 && c.is_uppercase() {
            result.push('_');
        }
        result.push(c.to_ascii_lowercase());
    }
    result
}

fn to_camel_case(val: &str) -> String {
    val.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first_char) => first_char.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

pub fn write_to_file(prefix: &str, value: String) {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    std::fs::write(format!("tmp/{}_{}.rs", prefix, timestamp), value).expect("write file failed");
}

// pub fn type_to_csharp(r#type: &Type) -> Option<String> {
//     match r#type {
//         Type::Reference(TypeReference { elem, .. }) => {
//             // 处理引用类型
//             // todo!();
//             type_to_csharp(elem)
//
//             // let inner_type = type_to_csharp(elem);
//             // if inner_type == "str" {
//             //     return "System.String".to_string();
//             // }
//             // format!("ref {}", inner_type)
//         }
//         Type::Path(TypePath { path, .. }) => {
//             let last_segment = path.segments.last().unwrap();
//             // let type_name = last_segment.ident.to_string();
//
//             let full_type = path
//                 .segments
//                 .iter()
//                 .map(|seg| seg.ident.to_string())
//                 .collect::<Vec<_>>()
//                 .join("::");
//
//             match full_type.as_str() {
//                 "i8" => Some("System.SByte".to_string()),
//                 "i16" => Some("System.Int16".to_string()),
//                 "i32" => Some("System.Int32".to_string()),
//                 "i64" => Some("System.Int64".to_string()),
//                 "u8" => Some("System.Byte".to_string()),
//                 "u16" => Some("System.UInt16".to_string()),
//                 "u32" => Some("System.UInt32".to_string()),
//                 "u64" => Some("System.UInt64".to_string()),
//                 "f32" => Some("System.Single".to_string()),
//                 "f64" => Some("System.Double".to_string()),
//                 "bool" => Some("System.Boolean".to_string()),
//                 "char" => Some("System.Char".to_string()),
//                 "&str" | "String" => Some("System.String".to_string()),
//                 "isize" => Some("System.IntPtr".to_string()),
//                 "usize" => Some("System.UIntPtr".to_string()),
//                 "nalgebra::Vector3" => Some("UnityEngine.Vector3".to_string()),
//                 "Vec" => {
//                     // 处理 Vec<T>
//                     if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
//                         let generics = args
//                             .args
//                             .iter()
//                             .filter_map(|arg| match arg {
//                                 GenericArgument::Type(ty) => Some(type_to_csharp(ty)),
//                                 _ => None, // 过滤掉非类型参数
//                             })
//                             .map(|x| x.unwrap_or("".to_string()))
//                             .collect::<Vec<String>>()
//                             .join(",");
//                         Some(format!("System.Collections.Generic.List<{}>", generics))
//                     } else {
//                         None
//                     }
//                 }
//                 _ => {
//                     // // write_to_file("full_type", full_type);
//                     // // return None;
//                     //
//                     // if full_type.starts_with("nalgebra::Vector3") {
//                     //     return Some("UnityEngine.Vector3".to_string());
//                     // }
//
//                     None
//                     // // 泛型处理
//                     // if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
//                     //     let generics = args
//                     //         .args
//                     //         .iter()
//                     //         .filter_map(|arg| match arg {
//                     //             GenericArgument::Type(ty) => Some(type_to_csharp(ty)),
//                     //             _ => None, // 过滤掉非类型参数
//                     //         })
//                     //         .map(|x| x.unwrap_or("".to_string()))
//                     //         .collect::<Vec<String>>()
//                     //         .join(",");
//                     //     return format!("{}<{}>", type_name, generics);
//                     // }
//                     // if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
//                     //     let generics = args
//                     //         .args
//                     //         .iter()
//                     //         .filter_map(|arg| match arg {
//                     //             GenericArgument::Type(ty) => Some(type_to_csharp(ty)),
//                     //             _ => None, // 过滤掉非类型参数
//                     //         })
//                     //         .map(|x| x.unwrap_or("".to_string()))
//                     //         .collect::<Vec<String>>()
//                     //         .join(",");
//                     //     Some(format!("{}<{}>", type_name, generics))
//                     // } else {
//                     //     None
//                     // }
//                 }
//             }
//         }
//         Type::Array(TypeArray { elem, .. }) | Type::Slice(TypeSlice { elem, .. }) => {
//             Some(format!("{}[]", type_to_csharp(elem)?))
//         }
//         _ => None,
//     }
// }

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
                "nalgebra::Vector3" => Some("UnityEngine.Vector3".to_string()),
                "Vec" => process_generic_type(full_type, path, "System.Collections.Generic.List"),
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
        .map(|seg| seg.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn process_generic_type(type_name: String, path: &Path, csharp_prefix: &str) -> Option<String> {
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

pub(crate) fn generate_unique_string(len: usize) -> String {
    loop {
        let uuid = uuid::Uuid::new_v4().to_string();
        let mut chars = uuid.chars();
        if let Some(first_char) = chars.next() {
            if first_char.is_alphabetic() {
                return first_char.to_string() + chars.take(len - 1).collect::<String>().as_str();
            }
        }
    }
}
