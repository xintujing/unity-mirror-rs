use nalgebra::{Quaternion, Vector3, Vector4};

pub struct NetworkReader;

#[allow(unused)]
impl NetworkReader {
    pub fn read_var_int(&mut self) -> i32 {
        todo!()
    }
    pub fn read_var_uint(&mut self) -> u32 {
        todo!()
    }
    pub fn read_var_long(&mut self) -> i64 {
        todo!()
    }
    pub fn read_var_ulong(&mut self) -> u64 {
        todo!()
    }
    pub fn read_string(&mut self) -> String {
        todo!()
    }
    pub fn read_blittable<T>(&mut self) -> T {
        todo!()
    }

    pub fn get_position(&self) -> usize {
        0
    }
    pub fn set_position(&mut self, value: usize) {}
}
// 组件内 Command/ClientRpc/ClientTarget/SyncVar 消息的参数反序列化
pub trait DataTypeDeserializer {
    fn deserialize(#[allow(unused)] reader: &mut NetworkReader) -> Self
    where
        Self: Sized;
}

macro_rules! data_type_deserialize {
    ($($typ:ty),* , {$closure:expr}) => {
        $(
            impl DataTypeDeserializer for $typ {
                fn deserialize(#[allow(unused)] reader: &mut NetworkReader) -> Self {
                    let closure: &dyn Fn(&mut NetworkReader)->Self = &$closure;
                    closure(reader)
                }
            }
        )*
    };
}

data_type_deserialize!(i32, { |reader| reader.read_var_int() });
data_type_deserialize!(u32, { |reader| reader.read_var_uint() });
data_type_deserialize!(i64, { |reader| reader.read_var_long() });
data_type_deserialize!(u64, { |reader| reader.read_var_ulong() });
data_type_deserialize!(String, { |reader| reader.read_string() });
data_type_deserialize!(
    i8,
    i16,
    u8,
    u16,
    f32,
    f64,
    bool,
    Vector3<f32>,
    Vector4<f32>,
    Quaternion<f32>,
    { |reader| reader.read_blittable() }
);

impl<T: DataTypeDeserializer> DataTypeDeserializer for Vec<T> {
    fn deserialize(#[allow(unused)] reader: &mut NetworkReader) -> Self {
        let size = reader.read_var_ulong() as usize - 1;
        let mut result = Vec::with_capacity(size);
        for _ in 0..size {
            result.push(T::deserialize(reader));
        }
        result
    }
}
