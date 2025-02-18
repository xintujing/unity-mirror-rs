use syn::{
    AngleBracketedGenericArguments, GenericArgument, PathArguments, Type, TypeArray, TypePath,
    TypeReference, TypeSlice, TypeTuple,
};

// 将蛇形命名转换为帕斯卡命名
pub(crate) fn snake_to_pascal(snake_case: &str) -> String {
    snake_case
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first_char) => first_char.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

// 将泛型参数转换为c#泛型参数
pub(crate) fn extract_generics(args: &AngleBracketedGenericArguments) -> String {
    args.args
        .iter()
        .filter_map(|arg| match arg {
            GenericArgument::Type(ty) => Some(type_to_csharp(ty)),
            _ => None, // 过滤掉非类型参数
        })
        .collect::<Vec<String>>()
        .join(", ")
}

// 将rust类型转换为c#类型
pub(crate) fn type_to_csharp(ty: &Type) -> String {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            let last_segment = path.segments.last().unwrap();
            let type_name = last_segment.ident.to_string();

            match type_name.as_str() {
                "u8" => "System.Byte".to_string(),  // 单个字节
                "i8" => "System.SByte".to_string(), // 有符号字节
                "u16" => "System.UInt16".to_string(),
                "i16" => "System.Int16".to_string(),
                "u32" => "System.UInt32".to_string(),
                "i32" => "System.Int32".to_string(),
                "u64" => "System.UInt64".to_string(),
                "i64" => "System.Int64".to_string(),
                "f32" => "System.Single".to_string(),
                "f64" => "System.Double".to_string(),
                "bool" => "System.Boolean".to_string(),
                "usize" => "System.UIntPtr".to_string(),
                "isize" => "System.IntPtr".to_string(),
                "&str" | "String" => "System.String".to_string(),
                "Vec" => {
                    // 处理 Vec<T>
                    if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
                        let generics = extract_generics(args);
                        return format!("System.Collections.Generic.List<{}>", generics);
                    }
                    "System.Collections.Generic.List<unknown>".to_string()
                }
                _ => {
                    // 泛型处理
                    if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
                        let generics = extract_generics(args);
                        return format!("{}<{}>", type_name, generics);
                    }
                    type_name
                }
            }
        }
        Type::Reference(TypeReference { elem, .. }) => {
            // 处理引用类型
            let inner_type = type_to_csharp(elem);
            if inner_type == "str" {
                return "System.String".to_string();
            }
            format!("ref {}", inner_type)
        }
        Type::Slice(TypeSlice { elem, .. }) => {
            // 处理切片类型，如 [u8]
            let elem_type = type_to_csharp(elem);
            format!("{}[]", elem_type)
        }
        Type::Tuple(TypeTuple { elems, .. }) => {
            // 处理元组类型
            let types = elems
                .iter()
                .map(type_to_csharp)
                .collect::<Vec<String>>()
                .join(", ");
            format!("({})", types)
        }
        Type::Array(TypeArray {
            elem, len: _len, ..
        }) => {
            // 处理数组类型
            let elem_type = type_to_csharp(elem);
            format!("{}[]", elem_type /*, len_str*/)
        }
        _ => "unknown".to_string(),
    }
}

/// 将*命名转换为蛇形命名
pub(crate) fn pascal_case_to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && c.is_uppercase() {
            result.push('_');
        }
        result.push(c.to_ascii_lowercase());
    }
    result
}
