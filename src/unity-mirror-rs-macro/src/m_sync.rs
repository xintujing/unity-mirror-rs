use proc_macro::TokenStream;
use quote::quote;
use std::time::SystemTime;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

// struct SyncVarArgs {
//     index: u64,
// }
//
// impl syn::parse::Parse for SyncVarArgs {
//     fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
//         let mut index = 0;
//         // 解析多个 `key = value` 或 `key` 形式的参数
//         while !input.is_empty() {
//             let name_value: MetaNameValue = input.parse()?; // 解析每个 `key = value`
//             let key = name_value.path.to_token_stream().to_string();
//             let value = name_value.value.to_token_stream();
//             match key.as_str() {
//                 "index" => {
//                     index = value.to_string().parse().unwrap();
//                 }
//                 _ => {}
//             }
//
//             // 如果有逗号，则跳过，否则结束
//             if input.peek(Token![,]) {
//                 input.parse::<Token![,]>()?;
//             }
//         }
//         Ok(SyncVarArgs { index })
//     }
// }

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
                        let field_name = field.ident.as_ref().unwrap();
                        let field_type = &field.ty;

                        // Generate logic for SerializeSyncVars
                        serialize_fields.push(quote! {
                            if force_all || (self.sync_var_dirty_bits() & (1 << #filed_index)) != 0 {
                                // TODO 压缩类型 特殊处理
                                writer.write_blittable::<#field_type>(self.#field_name);
                            }
                        });

                        // Generate logic for DeserializeSyncVars
                        deserialize_fields.push(quote! {
                            if initial_state || (reader.decompress_var_ulong() & (1 << #filed_index)) != 0 {
                                // TODO 压缩类型 特殊处理
                                self.#field_name = reader.read_blittable::<#field_type>();
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
