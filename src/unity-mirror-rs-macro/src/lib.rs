extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use std::time::SystemTime;
use syn::parse::{Parse, ParseStream};
use syn::*;

#[proc_macro_attribute]
pub fn mirror(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // 解析输入的 TokenStream 到 ItemImpl 结构
    let mut input = parse_macro_input!(item as ItemFn);

    // 只能在main方法上使用
    if input.sig.ident != "main" {
        panic!("Only main method can use mirror attribute");
    }

    // 在main方法上添加一个方法，用于注册命令
    input.block.stmts.insert(
        0,
        parse_quote! {
            unsafe {
                for register_function in REGISTER_FUNCTIONS.iter() {
                    register_function()
                }
            }
        },
    );

    let stream = TokenStream::from(quote! {

        pub static mut REGISTER_FUNCTIONS: Vec<fn()> = vec![];

        #input

    });

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    std::fs::write(format!("{}_main.rs", timestamp), stream.to_string())
        .expect("write file failed");

    stream
}

struct ComponentArgs {
    namespace: String,
}

// 实现属性解析器
impl Parse for ComponentArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut namespace: String = "".to_string();

        // 解析多个 `key = value` 或 `key` 形式的参数
        while !input.is_empty() {
            let name_value: MetaNameValue = input.parse()?; // 解析每个 `key = value`
            let key = name_value.path.to_token_stream().to_string();
            let value = name_value.value.to_token_stream();
            match key.as_str() {
                "namespace" => {
                    namespace = value.to_string().trim_matches('"').to_string();
                }
                _ => {}
            }

            // 如果有逗号，则跳过，否则结束
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(ComponentArgs { namespace })
    }
}

/// 定义 component 宏
#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 解析属性参数
    let args = parse_macro_input!(attr as ComponentArgs);
    let namespace = args.namespace;

    // 解析输入的 TokenStream 到 ItemImpl 结构
    let mut item_impl = parse_macro_input!(item as ItemImpl);

    // 获取结构体的标识符
    let struct_ident = match item_impl.self_ty.as_ref() {
        Type::Path(path) => path.path.segments.last().unwrap().ident.clone(),
        _ => panic!("Unsupported type for impl block"),
    };

    // 生成并且需要被注册的 invoke_user_code 方法
    let mut invoke_user_code_methods: Vec<proc_macro2::TokenStream> = vec![];
    // 注册语句列表
    let mut reg_statements: Vec<proc_macro2::TokenStream> = vec![];

    // 遍历impl的方法
    for item_impl_item in &mut item_impl.items {
        // 如果是方法
        if let ImplItem::Fn(impl_item_method) = item_impl_item {
            // 方法的第一个参数必须有 `self`
            match impl_item_method.sig.inputs.first() {
                None => {
                    panic!(
                        "The func {} first argument of the method must be have `self`",
                        impl_item_method.sig.ident
                    );
                }
                Some(first_arg) => {
                    match first_arg {
                        // 参数是self
                        FnArg::Receiver(_) => {
                            // 遍历方法的属性
                            for attr in impl_item_method.attrs.iter() {
                                match attr.path().get_ident().unwrap().to_string().as_str() {
                                    // command属性
                                    "command" => {
                                        // 被标注的方法 ident
                                        let method_ident = &impl_item_method.sig.ident;
                                        // 获取方法入参TokenStream
                                        let mut method_params = vec![];
                                        for arg in impl_item_method.sig.inputs.iter() {
                                            match arg {
                                                FnArg::Typed(pat_type) => {
                                                    let param_type = type_to_csharp(&pat_type.ty);
                                                    method_params.push(param_type);
                                                }
                                                _ => {}
                                            }
                                        }

                                        // 方法全名  宏参：function_full_name
                                        // let mut function_full_name: Option<String> = Some("".to_string());
                                        // 是否需要权限  宏参：requires_authority
                                        let mut requires_authority = false;

                                        // 解析属性参数
                                        match attr.parse_args::<CommandArgs>() {
                                            Ok(command_args) => {
                                                // function_full_name = command_args.function_full_name;
                                                requires_authority =
                                                    command_args.requires_authority;
                                            }
                                            Err(_) => {}
                                        }

                                        let mut params = Vec::new();
                                        // 遍历方法的所有参数
                                        for arg in
                                            impl_item_method.sig.inputs.iter().filter_map(|input| {
                                                match input {
                                                    FnArg::Typed(pat_type) => Some(pat_type),
                                                    _ => None,
                                                }
                                            })
                                        {
                                            let param_type = type_to_csharp(&arg.ty);
                                            params.push(param_type);
                                        }

                                        // 构造invoke_user_code方法名称标识符
                                        let invoke_user_code_method_name = format_ident!(
                                            "__invoke_user_code__{}", // 模板
                                            method_ident.to_string()  // 方法名称
                                        );

                                        let mut params_into = vec![];
                                        for _ in 0..params.len() {
                                            params_into.push(quote! {reader.read_blittable()})
                                        }
                                        // 构造invoke_user_code方法 并添加到列表
                                        invoke_user_code_methods.push(quote! {
                                            fn #invoke_user_code_method_name(obj: &mut crate::mirror::core::network_behaviour::NetworkBehaviourType,reader: &mut crate::mirror::core::network_reader::NetworkReader , sender_connection: &mut crate::mirror::core::network_connection_to_client::NetworkConnectionToClient) {
                                                // TODO: 删除 打印调用的方法名称
                                                println!("invoke_user_code: {}", stringify!(#method_ident));
                                                obj.as_any_mut()
                                                   .downcast_mut::<Self>()
                                                   .unwrap()
                                                   .#method_ident(#(#params_into,)*);
                                            }
                                        });

                                        let method_name = snake_to_pascal(
                                            &impl_item_method.sig.ident.to_string(),
                                        );

                                        let params_str = params.join(",");

                                        // 构造csharp签名
                                        let csharp_signature = match method_name.as_str() {
                                            "" => {
                                                format!(
                                                    "System.Void {}::{}()",
                                                    struct_ident.to_string(),
                                                    method_name
                                                )
                                            }
                                            _ => {
                                                format!(
                                                    "System.Void {}.{}::{}({})",
                                                    namespace, // 固定的命名空间
                                                    struct_ident.to_string(),
                                                    method_name,
                                                    params_str
                                                )
                                            }
                                        };

                                        // 添加注册语句
                                        reg_statements.push(quote! {
                                            // 调用注册函数
                                            crate::mirror::core::remote_calls::RemoteProcedureCalls::register_command_delegate::<#struct_ident>(
                                                &#csharp_signature,
                                                #struct_ident::#invoke_user_code_method_name,
                                                #requires_authority,
                                            );
                                        });
                                    }
                                    "rpc" => { // rpc属性
                                    }
                                    _ => {}
                                }
                            }
                        }
                        // 参数是普通的类型，不含 `self`
                        FnArg::Typed(_) => {
                            // 参数是普通的类型，不含 `self`
                            panic!(
                                "The func {} first argument of the method must be have `self`",
                                impl_item_method.sig.ident
                            );
                        }
                    }
                }
            }
        }
    }

    let token_stream = TokenStream::from(quote! {

        #item_impl
        impl #struct_ident {
            // 构造invoke_user_code方法
            #(#invoke_user_code_methods)*
        }

        // 构造 注册方法
        #[ctor::ctor]
        fn __register() {
            #(#reg_statements)*
        }
    });

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // 删除 tmp/ 目录下的所有文件
    std::fs::remove_dir_all("tmp").unwrap_or_default();
    // 创建 tmp/ 目录
    std::fs::create_dir("tmp").unwrap_or_default();
    std::fs::write(format!("tmp/{}_.rs", timestamp), token_stream.to_string())
        .expect("write file failed");

    token_stream
}

/// 将*命名转换为蛇形命名
fn pascal_case_to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && c.is_uppercase() {
            result.push('_');
        }
        result.push(c.to_ascii_lowercase());
    }
    result
}

struct CommandArgs {
    // function_full_name: Option<String>,
    requires_authority: bool,
}

// 实现属性解析器
impl Parse for CommandArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut requires_authority = false;

        // 解析多个 `key = value` 或 `key` 形式的参数
        while !input.is_empty() {
            let name_value: MetaNameValue = input.parse()?; // 解析每个 `key = value`
            let key = name_value.path.to_token_stream().to_string();
            let value = name_value.value.to_token_stream();
            match key.as_str() {
                "requires_authority" => {
                    requires_authority = value.to_string().parse().unwrap();
                }
                _ => {}
            }

            // 如果有逗号，则跳过，否则结束
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(CommandArgs { requires_authority })
    }
}

/// 定义 command attribute 宏
#[proc_macro_attribute]
pub fn command(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// 定义 rpc attribute 宏
#[proc_macro_attribute]
pub fn rpc(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

// 将蛇形命名转换为帕斯卡命名
fn snake_to_pascal(snake_case: &str) -> String {
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
fn extract_generics(args: &AngleBracketedGenericArguments) -> String {
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
fn type_to_csharp(ty: &Type) -> String {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            let last_segment = path.segments.last().unwrap();
            let type_name = last_segment.ident.to_string();

            match type_name.as_str() {
                "i32" => "System.Int32".to_string(),
                "i64" => "System.Int64".to_string(),
                "f32" => "System.Single".to_string(),
                "f64" => "System.Double".to_string(),
                "bool" => "System.Boolean".to_string(),
                "&str" | "String" => "System.String".to_string(),
                "usize" => "System.UIntPtr".to_string(),
                "isize" => "System.IntPtr".to_string(),
                "u8" => "System.Byte".to_string(), // 单个字节
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
