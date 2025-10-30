use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

pub(crate) fn handler(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::ItemStruct);
    let struct_ident = &input.ident;

    // 收集字段
    let mut fields = Vec::new();
    for field in &input.fields {
        fields.push((field.ident.clone().unwrap(), field.ty.clone()));
    }

    let mut serialize_ts = Vec::new();
    let mut deserialize_ts = Vec::new();
    for (field, field_type) in fields {
        serialize_ts.push(quote! {
                self.#field.serialize(writer);
        });
        deserialize_ts.push(quote! {
                this.#field = <#field_type as DataTypeDeserializer>::deserialize(reader);
        });
    }


    //
    // impl MessageDeserializer for AddPlayerMessage {
    //     fn deserialize(reader: &mut NetworkReader) -> Self
    //     where
    //         Self: Sized,
    //     {
    //         let _ = reader;
    //         Self
    //     }
    // }

    let output = quote! {

        impl NetworkMessage for #struct_ident {

        }

        impl MessageSerializer for #struct_ident {
            fn serialize(&mut self, writer: &mut NetworkWriter)
            where
                Self: Sized,
            {
                writer.write_blittable(Self::get_full_name().hash16());
                #(#serialize_ts)*
            }
        }

        impl MessageDeserializer for #struct_ident {
            fn deserialize(reader: &mut NetworkReader) -> Self
            where
                Self: Sized,
            {
                let mut this = Self::default();
                #(#deserialize_ts)*
                this
            }
        }
    };

    output.into()
}
