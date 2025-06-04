use crate::commons::to_hex_string::ToHexString;
use crate::mirror::compress::Compress;
use std::fmt::{Display, Formatter};

pub trait WriteCompress {
    fn compress(&self, writer: &mut NetworkWriter);
}
impl WriteCompress for i32 {
    fn compress(&self, writer: &mut NetworkWriter) {
        <i64 as WriteCompress>::compress(&(*self as i64), writer)
    }
}
impl WriteCompress for i64 {
    fn compress(&self, writer: &mut NetworkWriter) {
        let zigzagged = ((*self >> 63) ^ (*self << 1)) as u64;
        <u64 as WriteCompress>::compress(&zigzagged, writer)
    }
}
impl WriteCompress for u32 {
    fn compress(&self, writer: &mut NetworkWriter) {
        <u64 as WriteCompress>::compress(&(*self as u64), writer)
    }
}
impl WriteCompress for u64 {
    fn compress(&self, writer: &mut NetworkWriter) {
        if *self <= 240 {
            writer.write_blittable::<u8>(*self as u8);
            return;
        }
        if *self <= 2287 {
            let a = ((*self - 240) >> 8) as u16 + 241;
            let b = (*self - 240) as u16;
            writer.write_blittable::<u16>((b << 8u16) | a);
            return;
        }
        if *self <= 67823 {
            let a = 249;
            let b = ((*self - 2288) >> 8) as u16;
            let c = (*self - 2288) as u16;
            writer.write_blittable::<u8>(a);
            writer.write_blittable::<u16>((c << 8u16) | b);
            return;
        }
        if *self <= 16777215 {
            let a = 250;
            let b = (*self << 8) as u32;
            writer.write_blittable::<u32>(b | a);
            return;
        }
        if *self <= 4294967295 {
            let a = 251;
            let b = *self as u32;
            writer.write_blittable::<u8>(a);
            writer.write_blittable::<u32>(b);
            return;
        }
        if *self <= 1099511627775 {
            let a = 252;
            let b = (*self & 0xFF) as u16;
            let c = (*self >> 8) as u32;
            writer.write_blittable::<u16>(b << 8 | a);
            writer.write_blittable::<u32>(c);
            return;
        }
        if *self <= 281474976710655 {
            let a = 253;
            let b = (*self & 0xFF) as u16;
            let c = ((*self >> 8) & 0xFF) as u16;
            let d = (*self >> 16) as u32;
            writer.write_blittable::<u8>(a);
            writer.write_blittable::<u16>(c << 8 | b);
            writer.write_blittable::<u32>(d);
            return;
        }
        if *self <= 72057594037927935 {
            let a = 254u64;
            let b = *self << 8;
            writer.write_blittable::<u64>(b | a);
            return;
        }

        writer.write_blittable::<u8>(255);
        writer.write_blittable::<u64>(*self);
    }
}
impl WriteCompress for nalgebra::Quaternion<f32> {
    fn compress(&self, writer: &mut NetworkWriter) {
        let (largest_index, _, mut without_largest) =
            Compress.largest_absolute_component_index(self);

        if self[largest_index] < 0.0 {
            without_largest = -without_largest;
        }

        let a_scaled = Compress.scale_float_to_ushort(
            without_largest.x,
            Compress::QUATERNION_MIN_RANGE,
            Compress::QUATERNION_MAX_RANGE,
            0,
            Compress::TEN_BITS_MAX as u16,
        );
        let b_scaled = Compress.scale_float_to_ushort(
            without_largest.y,
            Compress::QUATERNION_MIN_RANGE,
            Compress::QUATERNION_MAX_RANGE,
            0,
            Compress::TEN_BITS_MAX as u16,
        );
        let c_scaled = Compress.scale_float_to_ushort(
            without_largest.z,
            Compress::QUATERNION_MIN_RANGE,
            Compress::QUATERNION_MAX_RANGE,
            0,
            Compress::TEN_BITS_MAX as u16,
        );

        // 将它们打包到一个整数中
        let i = (largest_index as u32) << 30
            | (a_scaled as u32) << 20
            | (b_scaled as u32) << 10
            | (c_scaled as u32);
        writer.write_blittable(i);
    }
}

#[derive(Default)]
pub struct NetworkWriter {
    buffer: Vec<u8>,
    pub position: usize,
}

impl Display for NetworkWriter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] @ {} / {}",
            self.buffer.to_hex_string(" ", true),
            self.position,
            self.capacity()
        )
    }
}

impl NetworkWriter {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(Self::default_capacity() as usize),
            position: 0,
        }
    }
    pub fn reset(&mut self) {
        self.position = 0;
    }

    fn ensure_capacity(&mut self, value: usize) {
        if self.buffer.len() < value as usize {
            let capacity = usize::max(value, self.buffer.len() * 2);
            self.buffer.resize(capacity, 0);
        }
    }
    pub fn to_slice(&self) -> &[u8] {
        &self.buffer[0..self.position]
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.buffer[0..self.position].to_vec()
    }

    pub fn write_blittable<T>(&mut self, value: T)
    where
        T: DataTypeSerializer,
    {
        let size = size_of::<T>();
        self.ensure_capacity(self.position + size);
        unsafe {
            std::ptr::copy_nonoverlapping(
                &value as *const T as *const u8,
                self.buffer.as_mut_ptr().add(self.position),
                size,
            );
        }
        self.position += size;
    }

    pub fn write_blittable_compress<T>(&mut self, value: T)
    where
        T: DataTypeSerializer + WriteCompress,
    {
        value.compress(self);
    }

    pub fn write_byte(&mut self, value: u8) {
        self.write_blittable(value)
    }

    pub fn write_slice(&mut self, value: &[u8], offset: usize, count: usize) {
        self.ensure_capacity(self.position + count);
        unsafe {
            std::ptr::copy_nonoverlapping(
                value.as_ptr().add(offset),
                self.buffer.as_mut_ptr().add(self.position),
                count,
            );
        }
        self.position += count;
    }

    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    pub fn write_str(&mut self, value: &str) {
        if value.is_empty() {
            self.write_blittable(0u16);
            return;
        }

        let max_size = value.as_bytes().len();
        self.ensure_capacity(self.position + 2 + max_size);

        unsafe {
            let written = std::ffi::CString::new(value).unwrap().to_bytes().len();
            std::ptr::copy_nonoverlapping(
                value.as_bytes().as_ptr(),
                self.buffer.as_mut_ptr().add(self.position + 2),
                written,
            );
            if written > NetworkWriter::max_string_length() as usize {
                self.write_blittable(0u16);
                log::error!(
                    "NetworkWriter.WriteString - Value too long: {} bytes. Limit: {} bytes",
                    written,
                    NetworkWriter::max_string_length()
                );
                return;
            }
            self.write_blittable((written + 1) as u16);
            self.position += 2 + written;
        }
    }
    pub fn write_slice_and_size(&mut self, value: &[u8]) {
        let count = value.len();
        if count == 0 {
            self.write_blittable_compress(0);
            return;
        }
        self.write_blittable_compress(1 + count as u64);
        self.write_slice(value, 0, count);
    }
}

impl NetworkWriter {
    pub fn max_string_length() -> u16 {
        u16::MAX - 1
    }

    pub fn default_capacity() -> i32 {
        1500
    }
}

pub trait DataTypeSerializer {
    fn serialize(&self, writer: &mut NetworkWriter)
    where
        Self: Sized;
}

macro_rules! data_type_serialize {
    (($($typ:ty),*), $logic:expr) => {
        $(
            impl DataTypeSerializer for $typ {
                fn serialize(&self, writer: &mut NetworkWriter) {
                    let closure: &dyn Fn(&$typ, &mut NetworkWriter) = &$logic;
                    closure(&self, writer);
                }
            }
        )*
    };
}

data_type_serialize!((i32, u32, i64, u64), |value, writer| writer
    .write_blittable_compress(*value));
data_type_serialize!((String), |value, writer| writer.write_str(&value));
data_type_serialize!(
    (
        i8,
        i16,
        u8,
        u16,
        f32,
        f64,
        bool,
        nalgebra::Vector3<f32>,
        nalgebra::Vector4<f32>,
        nalgebra::Quaternion<f32>
    ),
    |value, writer| writer.write_blittable(*value)
);

impl<T: DataTypeSerializer> DataTypeSerializer for Vec<T> {
    fn serialize(&self, writer: &mut NetworkWriter) {
        if self.len() == 0 {
            writer.write_blittable_compress::<u64>(0);
            return;
        }

        writer.write_blittable_compress::<u64>(self.len() as u64 + 1);
        for item in self {
            item.serialize(writer);
        }
    }
}

impl<T: DataTypeSerializer> DataTypeSerializer for &[T] {
    fn serialize(&self, writer: &mut NetworkWriter) {
        writer.write_blittable_compress::<u64>(self.len() as u64 + 1);
        for item in *self {
            item.serialize(writer);
        }
    }
}
