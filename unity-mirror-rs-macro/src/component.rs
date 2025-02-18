use crate::command::command_handler;
use crate::rpc::rpc_handler;
use crate::utils::generate_unique_string;
use crate::ComponentArgs;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, ImplItem, ItemImpl, Type};

pub(crate) struct ComponentBlock {
    invoke_user_code: Vec<proc_macro2::TokenStream>,
    register_statements: Vec<proc_macro2::TokenStream>,
}

impl ComponentBlock {
    fn new() -> Self {
        ComponentBlock {
            invoke_user_code: vec![],
            register_statements: vec![],
        }
    }

    pub(crate) fn add_register(&mut self, register: proc_macro2::TokenStream) {
        self.register_statements.push(register);
    }

    pub(crate) fn add_invoke_user_code(&mut self, output: proc_macro2::TokenStream) {
        self.invoke_user_code.push(output);
    }
}

pub(crate) fn component_attribute_handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 解析属性参数
    let args = parse_macro_input!(attr as ComponentArgs);
    let namespace = match args.namespace {
        None => {
            panic!("namespace is required")
        }
        Some(value) => value,
    };

    // 解析输入的 TokenStream 到 ItemImpl 结构
    let mut input = parse_macro_input!(item as ItemImpl);

    // 获取结构体的标识符
    let struct_ident = match input.self_ty.as_ref() {
        Type::Path(path) => path.path.segments.last().unwrap().ident.clone(),
        _ => panic!("Unsupported type for impl block"),
    };

    let mut component_block = ComponentBlock::new();

    // 遍历impl的方法
    for item_impl_item in &mut input.items {
        // 如果是方法
        if let ImplItem::Fn(impl_item_method) = item_impl_item {
            // 方法的第一个参数必须有 `self`
            if let Some(FnArg::Receiver(_)) = impl_item_method.sig.inputs.first() {
                // 遍历方法的属性
                for attr in impl_item_method.attrs.iter() {
                    match attr.path().get_ident().unwrap().to_string().as_str() {
                        "command" => {
                            command_handler(
                                &namespace,
                                &struct_ident,
                                attr,
                                &impl_item_method.sig,
                                &mut component_block,
                            );
                        }
                        "rpc" => {
                            rpc_handler(
                                &namespace,
                                &struct_ident,
                                attr,
                                &impl_item_method.sig,
                                &mut component_block,
                            );
                        }
                        _ => {}
                    }
                }
            } else {
                panic!(
                    "The func {} first argument of the method must be have `self`",
                    impl_item_method.sig.ident
                );
            }
        }
    }

    let register_ident = format_ident!("__register_{}", generate_unique_string(5).to_lowercase());

    let ComponentBlock {
        invoke_user_code,
        register_statements,
    } = component_block;

    let token_stream = TokenStream::from(quote! {
        #input

        impl #struct_ident {
            #(#invoke_user_code)*
        }

        #[ctor::ctor]
        fn #register_ident() {
            use crate::mirror::namespace::Namespace;

            #(#register_statements)*
        }
    });

    // write_to_file("component", token_stream.to_string());

    token_stream
}
