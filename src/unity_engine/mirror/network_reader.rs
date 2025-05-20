use crate::commons::to_hex_string::ToHexString;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::string::FromUtf8Error;
use crate::unity_engine::mirror::network_writer::NetworkWriter;

trait ReadCompress {
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

const ALLOCATION_LIMIT: i32 = 1024 * 1024 * 16;

#[derive(Debug)]
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

    pub fn set_buffer(&mut self, segment: Vec<u8>) {
        self.buffer = segment;
        self.position = 0;
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

    pub fn read_bytes(&mut self, bytes: &mut Vec<u8>, count: usize) -> Result<(), Box<dyn Error>> {
        if count > bytes.len() {
            return Err(format!(
                "ReadBytes can't read {} bytes because the passed byte[] only has length {}",
                count,
                bytes.len()
            )
            .into());
        }

        if self.remaining() < count {
            return Err(format!(
                "ReadBytes can't read {} bytes because it would read past the end of the stream. {:?}",
                count,
                self
            )
            .into());
        }

        unsafe {
            let src = self.buffer.as_ptr().add(self.position);
            let dst = bytes.as_mut_ptr();
            std::ptr::copy_nonoverlapping(src, dst, count);
        }

        self.position += count;

        Ok(())
    }

    pub fn read_bytes_segment(&mut self, count: usize) -> Vec<u8> {
        if self.remaining() < count {
            log::error!(
                "ReadBytesSegment can't read {} bytes because it would read past the end of the stream. {}",
                count,
                self
            );
            return vec![];
        }

        let bytes_segment = self.buffer[self.position..self.position + count].to_vec();
        self.position += count;
        bytes_segment
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

        let bytes_segment = self.read_bytes_segment(real_size as usize);
        String::from_utf8(bytes_segment).unwrap_or_else(|err| {
            log::error!("NetworkReader.ReadString - Invalid UTF8: {}", err);
            "".to_string()
        })
    }

    pub fn remaining(&self) -> usize {
        self.buffer.len() - self.position
    }
    pub fn capacity(&self) -> usize {
        self.buffer.len()
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
