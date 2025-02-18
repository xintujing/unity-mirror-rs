use proc_macro::TokenStream;
use quote::quote;
use std::time::SystemTime;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

pub(crate) fn network_message_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // 获取结构体的名字
    let struct_ident = input.ident;

    let mut serialize_fields = Vec::new();
    let mut deserialize_fields = Vec::new();

    // 如果输入的是结构体
    if let Data::Struct(data) = input.data {
        // 用于标记字段的索引
        if let Fields::Named(fields_named) = data.fields {
            for field in fields_named.named.iter() {
                // 获取字段的名字
                let field_name = field.ident.as_ref().unwrap();
                // 获取字段的类型
                let field_type = &field.ty;
                // 获取字段的类型名字
                let field_type_str = match field_type {
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
                let serialize_ts = match field_type_str.as_str() {
                    "String" => {
                        // String 类型
                        quote! {writer.write_string(self.#field_name.clone());}
                    }
                    "str" => {
                        quote! {writer.write_str(self.#field_name);}
                    }
                    "i32" => {
                        quote! {writer.compress_var_int(self.#field_name);}
                    }
                    "u32" => {
                        quote! {writer.compress_var_uint(self.#field_name);}
                    }
                    "i64" => {
                        quote! {writer.compress_var_long(self.#field_name);}
                    }
                    "u64" => {
                        quote! {writer.compress_var_ulong(self.#field_name);}
                    }
                    "Vec" => {
                        quote! {writer.write_array_segment_and_size(self.#field_name.as_slice());}
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
                        quote! {_self.#field_name = reader.read_string();}
                    }
                    "str" => {
                        quote! {_self.#field_name = reader.read_str();}
                    }
                    "i32" => {
                        quote! {_self.#field_name = reader.decompress_var_int();}
                    }
                    "u32" => {
                        quote! {_self.#field_name = reader.decompress_var_uint();}
                    }
                    "i64" => {
                        quote! {_self.#field_name = reader.decompress_var_long();}
                    }
                    "u64" => {
                        quote! {_self.#field_name = reader.decompress_var_ulong();}
                    }
                    "Vec" => {
                        quote! { _self.#field_name = reader.read_bytes_and_size();}
                    }
                    _ => {
                        quote! {_self.#field_name = reader.read_blittable::<#field_type>();}
                    }
                };

                // Generate logic for SerializeSyncVars
                serialize_fields.push(quote! {
                    #serialize_ts
                });

                // Generate logic for DeserializeSyncVars
                deserialize_fields.push(quote! {
                    #deserialize_ts
                });
            }
        }
    }

    // Generate SerializeSyncVars and DeserializeSyncVars functions
    let generated = quote! {
        impl crate::mirror::core::messages::NetworkMessagePreTrait for #struct_ident {
            fn serialize(&mut self, writer: &mut crate::mirror::core::network_writer::NetworkWriter) {
                use crate::mirror::core::network_writer::NetworkWriterTrait;
                use crate::mirror::core::tools::stable_hash::StableHash;
                writer.write_ushort(Self::get_full_name().get_stable_hash_code16());
                #(#serialize_fields)*
            }

            fn deserialize(reader: &mut crate::mirror::core::network_reader::NetworkReader) -> Self {
                use crate::mirror::core::network_reader::NetworkReaderTrait;
                let mut _self = Self::default();
                #(#deserialize_fields)*
                _self
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
        format!("tmp/__network_message_{}.rs", timestamp),
        token_stream.to_string(),
    )
    .expect("write file failed");

    token_stream
}
