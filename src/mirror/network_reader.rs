#![allow(unused)]
use crate::commons::to_hex_string::ToHexString;
use crate::mirror::compress::Compress;
use crate::mirror::NetworkWriter;
use nalgebra::Vector4;
use std::error::Error;
use std::fmt::{Display, Formatter};

pub trait ReadCompress {
    fn decompress(reader: &mut NetworkReader) -> Self
    where
        Self: Sized;
}

impl ReadCompress for i32 {
    fn decompress(reader: &mut NetworkReader) -> Self
    where
        Self: Sized,
    {
        <i64 as ReadCompress>::decompress(reader) as i32
    }
}
impl ReadCompress for i64 {
    fn decompress(reader: &mut NetworkReader) -> Self
    where
        Self: Sized,
    {
        let value = <u64 as ReadCompress>::decompress(reader) as i64;
        (value >> 1) ^ -(value & 1)
    }
}
impl ReadCompress for u32 {
    fn decompress(reader: &mut NetworkReader) -> Self
    where
        Self: Sized,
    {
        <u64 as ReadCompress>::decompress(reader) as u32
    }
}
impl ReadCompress for u64 {
    fn decompress(reader: &mut NetworkReader) -> Self
    where
        Self: Sized,
    {
        let a0 = reader.read_blittable::<u8>() as u64;
        if a0 < 241 {
            return a0;
        }

        let a1 = reader.read_blittable::<u8>() as u64;
        if a0 <= 248 {
            return 240 + ((a0 - 241) << 8) + a1;
        }

        let a2 = reader.read_blittable::<u8>() as u64;
        if a0 == 249 {
            return 2288 + (a1 << 8) + a2;
        }

        let a3 = reader.read_blittable::<u8>() as u64;
        if a0 == 250 {
            return a1 + (a2 << 8) + (a3 << 16);
        }

        let a4 = reader.read_blittable::<u8>() as u64;
        if a0 == 251 {
            return a1 + (a2 << 8) + (a3 << 16) + (a4 << 24);
        }

        let a5 = reader.read_blittable::<u8>() as u64;
        if a0 == 252 {
            return a1 + (a2 << 8) + (a3 << 16) + (a4 << 24) + (a5 << 32);
        }

        let a6 = reader.read_blittable::<u8>() as u64;
        if a0 == 253 {
            return a1 + (a2 << 8) + (a3 << 16) + (a4 << 24) + (a5 << 32) + (a6 << 40);
        }

        let a7 = reader.read_blittable::<u8>() as u64;
        if a0 == 254 {
            return a1 + (a2 << 8) + (a3 << 16) + (a4 << 24) + (a5 << 32) + (a6 << 40) + (a7 << 48);
        }

        let a8 = reader.read_blittable::<u8>() as u64;
        if a0 == 255 {
            return a1
                + (a2 << 8)
                + (a3 << 16)
                + (a4 << 24)
                + (a5 << 32)
                + (a6 << 40)
                + (a7 << 48)
                + (a8 << 56);
        }
        log::error!("Invalid decompression value: {a0}",);
        0
    }
}
impl ReadCompress for nalgebra::Quaternion<f32> {
    fn decompress(reader: &mut NetworkReader) -> Self
    where
        Self: Sized,
    {
        let data = reader.read_blittable::<u32>();
        // 获取 cScaled（位 0..10）
        let c_scaled = (data & Compress::TEN_BITS_MAX) as u16;

        // 获取 bScaled（位 10..20）
        let b_scaled = ((data >> 10) & Compress::TEN_BITS_MAX) as u16;

        // 获取 aScaled（位 20..30）
        let a_scaled = ((data >> 20) & Compress::TEN_BITS_MAX) as u16;

        // 获取 largestIndex（位 30..32）
        let largest_index = (data >> 30) as usize;

        // 缩放回浮点数
        let a = Compress.scale_ushort_to_float(
            a_scaled,
            0,
            Compress::TEN_BITS_MAX,
            Compress::QUATERNION_MIN_RANGE,
            Compress::QUATERNION_MAX_RANGE,
        );
        let b = Compress.scale_ushort_to_float(
            b_scaled,
            0,
            Compress::TEN_BITS_MAX,
            Compress::QUATERNION_MIN_RANGE,
            Compress::QUATERNION_MAX_RANGE,
        );
        let c = Compress.scale_ushort_to_float(
            c_scaled,
            0,
            Compress::TEN_BITS_MAX,
            Compress::QUATERNION_MIN_RANGE,
            Compress::QUATERNION_MAX_RANGE,
        );

        // 计算省略的分量 d，基于 a² + b² + c² + d² = 1
        let d = (1.0 - a * a - b * b - c * c).sqrt();

        // 根据 largestIndex 重建四元数
        let v4 = match largest_index {
            0 => Vector4::new(d, a, b, c),
            1 => Vector4::new(a, d, b, c),
            2 => Vector4::new(a, b, d, c),
            _ => Vector4::new(a, b, c, d),
        };

        Compress.quaternion_normalize_safe(v4)
    }
}

const ALLOCATION_LIMIT: i32 = 1024 * 1024 * 16;

#[derive(Debug, Default)]
pub struct NetworkReader {
    buffer: Vec<u8>,
    pub position: usize,
}

impl Display for NetworkReader {
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

impl NetworkReader {
    pub fn new(segment: Vec<u8>) -> Self {
        Self {
            buffer: segment,
            position: 0,
        }
    }

    pub fn set_vec(&mut self, segment: Vec<u8>) {
        self.buffer = segment;
        self.position = 0;
    }

    pub fn set_slice(&mut self, segment: &[u8]) {
        self.buffer = segment.to_vec();
        self.position = 0;
    }

    pub fn to_slice(&self) -> &[u8] {
        &self.buffer[self.position..]
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.buffer[self.position..].to_vec()
    }

    pub fn read_blittable<T>(&mut self) -> T {
        let size = size_of::<T>();
        if self.remaining() < size {
            log::error!(
                "ReadBlittable<{}> not enough data in buffer to read {} bytes: {:?}",
                std::any::type_name::<T>(),
                size,
                self
            );
            return unsafe { std::mem::zeroed() };
        }

        let value = unsafe {
            let ptr = self.buffer.as_ptr().add(self.position) as *const T;
            ptr.read_unaligned()
        };
        self.position += size;

        value
    }

    pub fn read_blittable_compress<T>(&mut self) -> T
    where
        T: DataTypeDeserializer + ReadCompress,
    {
        T::decompress(self)
    }

    pub fn read_blittable_nullable<T>(&mut self) -> T
    where
        T: DataTypeDeserializer,
    {
        if self.read_byte() != 0 {
            return self.read_blittable();
        }
        unsafe { std::mem::zeroed() }
    }

    pub fn read_byte(&mut self) -> u8 {
        self.read_blittable()
    }
    pub fn read_slice(&mut self, count: usize) -> &[u8] {
        if self.remaining() < count {
            log::error!(
                "ReadBytesSegment can't read {} bytes because it would read past the end of the stream. {}",
                count,
                self
            );
            return &[];
        }
        let x = &self.buffer[self.position..self.position + count];
        self.position += count;
        x
    }
    pub fn read_string(&mut self) -> String {
        let size = self.read_blittable::<u16>();
        if size == 0 {
            return "".to_string();
        }
        let real_size = size - 1;

        if real_size > NetworkWriter::max_string_length() {
            log::error!(
                "NetworkReader.ReadString - Value too long: {} bytes. Limit is: {} bytes",
                real_size,
                NetworkWriter::max_string_length()
            );
            return "".to_string();
        }

        let bytes_segment = self.read_slice(real_size as usize);
        String::from_utf8(bytes_segment.to_vec()).unwrap_or_else(|err| {
            log::error!("NetworkReader.ReadString - Invalid UTF8: {}", err);
            "".to_string()
        })
    }
    pub fn read_slice_and_size(&mut self) -> &[u8] {
        let count: u32 = self.read_blittable_compress();
        if count == 0 {
            return &[];
        }
        self.read_slice(count as usize - 1)
    }
    pub fn remaining(&self) -> usize {
        self.buffer.len() - self.position
    }
    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }
    pub fn reset(&mut self) {
        self.position = 0;
    }
}

pub trait DataTypeDeserializer {
    fn deserialize(#[allow(unused)] reader: &mut NetworkReader) -> Self
    where
        Self: Sized;
}

macro_rules! data_type_deserialize {
    (($($typ:ty),*), {$closure:expr}) => {
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

data_type_deserialize!((i32, u32, i64, u64), {
    |reader| reader.read_blittable_compress()
});
data_type_deserialize!((String), { |reader| reader.read_string() });
data_type_deserialize!(
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
    { |reader| reader.read_blittable() }
);

impl<T: DataTypeDeserializer> DataTypeDeserializer for Vec<T> {
    fn deserialize(#[allow(unused)] reader: &mut NetworkReader) -> Self {
        let size = reader.read_blittable_compress::<u64>() as usize - 1;
        let mut result = Vec::with_capacity(size);
        for _ in 0..size {
            result.push(T::deserialize(reader));
        }
        result
    }
}

impl<T: DataTypeDeserializer> DataTypeDeserializer for &[T] {
    fn deserialize(reader: &mut NetworkReader) -> Self {
        let size = reader.read_blittable_compress::<u64>() as usize - 1;
        let mut result = Vec::with_capacity(size);
        for i in 0..size {
            let value = T::deserialize(reader);
            result.push(value);
        }
        unsafe { std::mem::transmute(result.as_slice()) }
    }
}
