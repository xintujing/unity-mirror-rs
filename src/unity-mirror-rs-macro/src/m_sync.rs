use proc_macro::TokenStream;
use quote::quote;
use std::time::SystemTime;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

/// sync 宏实现
pub(crate) fn m_sync_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_ident = input.ident;

    let mut serialize_fields = Vec::new();
    let mut deserialize_fields = Vec::new();

    // 如果输入的是结构体
    if let Data::Struct(data) = input.data {
        // 用于标记字段的索引
        if let Fields::Named(fields_named) = data.fields {
            let mut sync_var_index = 0u64;
            for field in fields_named.named.iter() {
                for attr in field.attrs.iter() {
                    // 如果是 #[sync_var] 属性
                    if attr.path().is_ident("sync_var") {
                        let filed_index = sync_var_index;
                        sync_var_index += 1;
                        // 获取字段的名字
                        let field_name = field.ident.as_ref().unwrap();
                        // 获取字段的类型
                        let field_type = &field.ty;
                        // 获取字段的类型名字
                        let ref field_type_str = match field_type {
                            Type::Path(type_path) => {
                                // 获取类型标识符的名字
                                type_path.path.segments.last().unwrap().ident.to_string()
                            }
                            _ => {
                                // 暂时不支持复杂类型
                                panic!("Field type {} is not supported", field_name);
                            }
                        };

                        // 生成序列化的代码
                        // TODO 压缩类型 特殊处理
                        let serialize_ts = match field_type_str.as_str() {
                            "String" => {
                                // String 类型
                                quote! {writer.write_string(self.#field_name.clone());}
                            }
                            "str" => {
                                quote! {writer.write_str(self.#field_name);}
                            }
                            _ => {
                                quote! {writer.write_blittable::<#field_type>(self.#field_name);}
                            }
                        };

                        // 生成反序列化的代码
                        // TODO 压缩类型 特殊处理
                        let deserialize_ts = match field_type_str.as_str() {
                            "String" => {
                                // String 类型
                                quote! {self.#field_name = reader.read_string();}
                            }
                            "str" => {
                                quote! {self.#field_name = reader.read_str();}
                            }
                            _ => {
                                quote! {self.#field_name = reader.read_blittable::<#field_type>();}
                            }
                        };

                        // Generate logic for SerializeSyncVars
                        serialize_fields.push(quote! {
                            if force_all || (self.sync_var_dirty_bits() & (1 << #filed_index)) != 0 {
                                #serialize_ts
                            }
                        });

                        // Generate logic for DeserializeSyncVars
                        deserialize_fields.push(quote! {
                            if initial_state || (reader.decompress_var_ulong() & (1 << #filed_index)) != 0 {
                                #deserialize_ts
                            }
                        });
                    }
                }
            }
        }
    }

    // Generate SerializeSyncVars and DeserializeSyncVars functions
    let generated = quote! {
        impl crate::mirror::core::network_behaviour::NetworkBehaviourMSyncTrait for #struct_ident {
            fn serialize_sync_vars(&mut self, writer: &mut crate::mirror::core::network_writer::NetworkWriter, force_all: bool) {
                use crate::mirror::core::network_writer::NetworkWriterTrait;
                if force_all {
                    #(#serialize_fields)*
                    return;
                }
                // 写入脏数据位
                writer.compress_var_ulong(self.sync_var_dirty_bits());
                #(#serialize_fields)*
            }

            fn deserialize_sync_vars(&mut self, reader: &mut crate::mirror::core::network_reader::NetworkReader, initial_state: bool) {
                use crate::mirror::core::network_reader::NetworkReaderTrait;
                if initial_state {
                    #(#deserialize_fields)*
                    return;
                }

                // 读取脏数据位
                let dirty_bits = reader.decompress_var_ulong();
                #(#deserialize_fields)*
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    };

    let token_stream = TokenStream::from(generated);

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    std::fs::write(
        format!("tmp/__sync_{}.rs", timestamp),
        token_stream.to_string(),
    )
    .expect("write file failed");

    token_stream
}
