use nalgebra::{Quaternion, Vector3, Vector4};

pub struct NetworkWriter;

#[allow(unused)]
impl NetworkWriter {
    pub fn write_var_int(&mut self, value: i32) {}
    pub fn write_var_uint(&mut self, value: u32) {}
    pub fn write_var_long(&mut self, value: i64) {}
    pub fn write_var_ulong(&mut self, value: u64) {}
    pub fn write_string(&mut self, value: String) {}
    pub fn write_blittable<T>(&mut self, value: T) {}

    pub fn get_position(&self) -> usize {
        0
    }

    pub fn set_position(&mut self, value: usize) {}

    pub fn to_bytes(&self) -> Vec<u8> {
        vec![]
    }
}

pub trait DataTypeSerializer {
    fn serialize(&self, writer: &mut NetworkWriter)
    where
        Self: Sized;
}

macro_rules! data_type_serialize {
    ($($typ:ty),*, $logic:expr) => {
        $(
            impl DataTypeSerializer for $typ {
                fn serialize(&self, writer: &mut NetworkWriter) {
                    let closure: &dyn Fn(&$typ, &mut NetworkWriter) = &$logic;
                    closure(&self, writer);
                }
            }
        )*
    };
    // (<$($generic:ident),*>, $type_name:ty, $logic:expr) => {
    //     impl<$($generic),*> DataTypeSerializer for $type_name {
    //         fn serialize(&self, writer: &mut NetworkWriter) {
    //             let closure: &dyn Fn(&$typ, &mut NetworkWriter) = &$logic;
    //             closure(&self, writer);
    //         }
    //     }
    // };
    // (<$($generic:ident: $bound:tt),*>, $type_name:ty, $logic:expr) => {
    //     impl<$($generic:$bound),*> DataTypeSerializer for $type_name {
    //         fn serialize(&self, writer: &mut NetworkWriter) {
    //             let closure: &dyn Fn(&$typ, &mut NetworkWriter) = &$logic;
    //             closure(&self, writer);
    //         }
    //     }
    // };
}

data_type_serialize!(i32, |value, writer| writer.write_var_int(*value));
data_type_serialize!(u32, |value, writer| writer.write_var_uint(*value));
data_type_serialize!(i64, |value, writer| writer.write_var_long(*value));
data_type_serialize!(u64, |value, writer| writer.write_var_ulong(*value));
data_type_serialize!(String, |value, writer| writer.write_string(value.clone()));
// data_type_serialize!(
//     i8,
//     i16,
//     u8,
//     u16,
//     f32,
//     f64,
//     bool,
//     Vector3<f32>,
//     Vector4<f32>,
//     Quaternion<f32>,
//     |value, writer| writer.write_blittable(value)
// );

impl DataTypeSerializer for i8 {
    fn serialize(&self, writer: &mut NetworkWriter) {
        let closure: &dyn Fn(&i8, &mut NetworkWriter) =
            &(|value, writer| writer.write_blittable(value));
        closure(&self, writer);
    }
}
impl DataTypeSerializer for i16 {
    fn serialize(&self, writer: &mut NetworkWriter) {
        let closure: &dyn Fn(&i16, &mut NetworkWriter) =
            &(|value, writer| writer.write_blittable(value));
        closure(&self, writer);
    }
}
impl DataTypeSerializer for u8 {
    fn serialize(&self, writer: &mut NetworkWriter) {
        let closure: &dyn Fn(&u8, &mut NetworkWriter) =
            &(|value, writer| writer.write_blittable(value));
        closure(&self, writer);
    }
}
impl DataTypeSerializer for u16 {
    fn serialize(&self, writer: &mut NetworkWriter) {
        let closure: &dyn Fn(&u16, &mut NetworkWriter) =
            &(|value, writer| writer.write_blittable(value));
        closure(&self, writer);
    }
}
impl DataTypeSerializer for f32 {
    fn serialize(&self, writer: &mut NetworkWriter) {
        let closure: &dyn Fn(&f32, &mut NetworkWriter) =
            &(|value, writer| writer.write_blittable(value));
        closure(&self, writer);
    }
}
impl DataTypeSerializer for f64 {
    fn serialize(&self, writer: &mut NetworkWriter) {
        let closure: &dyn Fn(&f64, &mut NetworkWriter) =
            &(|value, writer| writer.write_blittable(value));
        closure(&self, writer);
    }
}
impl DataTypeSerializer for bool {
    fn serialize(&self, writer: &mut NetworkWriter) {
        let closure: &dyn Fn(&bool, &mut NetworkWriter) =
            &(|value, writer| writer.write_blittable(value));
        closure(&self, writer);
    }
}
impl DataTypeSerializer for Vector3<f32> {
    fn serialize(&self, writer: &mut NetworkWriter) {
        let closure: &dyn Fn(&Vector3<f32>, &mut NetworkWriter) =
            &(|value, writer| writer.write_blittable(value));
        closure(&self, writer);
    }
}
impl DataTypeSerializer for Vector4<f32> {
    fn serialize(&self, writer: &mut NetworkWriter) {
        let closure: &dyn Fn(&Vector4<f32>, &mut NetworkWriter) =
            &(|value, writer| writer.write_blittable(value));
        closure(&self, writer);
    }
}
impl DataTypeSerializer for Quaternion<f32> {
    fn serialize(&self, writer: &mut NetworkWriter) {
        let closure: &dyn Fn(&Quaternion<f32>, &mut NetworkWriter) =
            &(|value, writer| writer.write_blittable(value));
        closure(&self, writer);
    }
}

impl<T: DataTypeSerializer> DataTypeSerializer for Vec<T> {
    fn serialize(&self, writer: &mut NetworkWriter) {
        if self.len() == 0 {
            writer.write_var_ulong(0);
            return;
        }

        writer.write_var_ulong(self.len() as u64 + 1);
        for item in self {
            item.serialize(writer);
        }
    }
}
