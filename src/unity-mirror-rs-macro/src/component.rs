use crate::tools::{snake_to_pascal, type_to_csharp};
use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use std::time::SystemTime;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, FnArg, ImplItem, ItemImpl, MetaNameValue, Token, Type};

// component 宏的属性参数
struct ComponentArgs {
    namespace: String,
}

// 实现属性解析器
impl Parse for ComponentArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
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

// command 宏的属性参数
struct CommandArgs {
    // function_full_name: Option<String>,
    requires_authority: bool,
}

// 实现属性解析器
impl Parse for CommandArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
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

/// component 宏实现
pub(crate) fn component_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
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
    // std::fs::remove_dir_all("tmp").unwrap_or_default();
    // 创建 tmp/ 目录
    std::fs::create_dir("tmp").unwrap_or_default();
    std::fs::write(
        format!("tmp/__component_{}.rs", timestamp),
        token_stream.to_string(),
    )
    .expect("write file failed");

    token_stream
}
