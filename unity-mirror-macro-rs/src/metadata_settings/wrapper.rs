use crate::utils::string_case::StringCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, parse_macro_input};

pub(crate) fn handler(input: TokenStream) -> TokenStream {
    let mut output = TokenStream::new();

    let input = parse_macro_input!(input as DeriveInput);

    // struct 名
    let struct_ident = &input.ident;

    // snake_case struct name
    // let register_ident = format_ident!("__{}_register", struct_ident.to_string().to_snake_case());

    // struct wrapper name
    let wrapper_struct_ident = format_ident!("{}Wrapper", struct_ident);

    let registers_ident = format_ident!(
        "{}_REGISTERS",
        struct_ident.to_string().to_camel_case().to_uppercase()
    );

    let ts: TokenStream = quote! {

        // 静态组件注册表
        static #registers_ident: once_cell::sync::Lazy<std::sync::Mutex<
            std::collections::HashMap<
                &'static str,
                fn(serde_json::Value) -> Result<Box<dyn crate::metadata_settings::wrapper::Settings>, serde_json::Error>
            >
        >> = once_cell::sync::Lazy::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));
        // 定义 Wrapper 结构体

        #[derive(Clone)]
        pub struct #wrapper_struct_ident {
            value: std::collections::BTreeMap<std::any::TypeId, Box<dyn crate::metadata_settings::wrapper::Settings>>,
            final_type_id: std::any::TypeId,
            final_full_name: String,
        }

        impl #wrapper_struct_ident {
            // 注册组件方法
            pub fn register<T: crate::metadata_settings::wrapper::Settings + 'static + for<'a> serde::Deserialize<'a>>() {
                let name = T::get_full_name();
                let parser = |value: serde_json::Value| -> Result<Box<dyn crate::metadata_settings::wrapper::Settings>, serde_json::Error> {
                    T::parse(value).map(|c| c as Box<dyn crate::metadata_settings::wrapper::Settings>)
                };
                if let Ok(mut component_registry) = #registers_ident.lock() {
                    if component_registry.contains_key(name) {
                        panic!("Settings already registered: {}", name);
                    }
                    component_registry.insert(name, parser);
                }
            }
            // 获取某个组件实例
            pub fn get<T: crate::metadata_settings::wrapper::Settings>(&self) -> &T {
                if let Some(component) = self.value.get(&std::any::TypeId::of::<T>()) {
                    return component.as_any().downcast_ref::<T>().unwrap();
                }
                panic!("Settings not found: {}", std::any::type_name::<T>());
            }
            // 获取最终组件实例
            pub fn get_finally(&self) -> (String, &Box<dyn crate::metadata_settings::wrapper::Settings>) {
                if let Some(component) = self.value.get(&self.final_type_id) {
                    return (self.final_full_name.clone(), component);
                }
                panic!("Settings not found");
            }
            pub fn get_final_full_name(&self) -> String {
                self.final_full_name.clone()
            }
        }
        // 自动实现 `serde::Deserialize`，支持 JSON 反序列化
        impl<'de> serde::Deserialize<'de> for #wrapper_struct_ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let mut wrapper = Self {
                    value: Default::default(),
                    final_type_id: std::any::TypeId::of::<Self>(),
                    final_full_name: "".to_string(),
                };
                // 动态反序列化
                let network_behaviour = std::vec::Vec::<std::collections::HashMap<String, serde_json::Value>>::deserialize(deserializer)?;
                for chain in network_behaviour.iter() {
                    for (key, value) in chain.iter() {
                        if let Ok(component_registry) = #registers_ident.lock() {
                            let parser = component_registry.get(key.as_str()).ok_or_else(|| {
                                serde::de::Error::custom(format!("Settings {} not found", key))
                            })?;
                            let component = parser(value.clone()).map_err(|err| {
                                serde::de::Error::custom(format!("Settings {} parse error: {}", key, err))
                            })?;
                            let id = component.as_any().type_id();
                            wrapper.final_type_id = id.clone();
                            wrapper.final_full_name = key.clone();
                            wrapper.value.insert(id, component);
                        }
                    }
                }
                Ok(wrapper)
            }
        }

        unity_mirror_macro_rs::settings_wrapper_register!(#struct_ident as #wrapper_struct_ident);

    }.into();

    output.extend(ts);

    output
}
