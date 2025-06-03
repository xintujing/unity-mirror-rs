use crate::mirror::network_reader::{DataTypeDeserializer, NetworkReader, ReadCompress};
use crate::mirror::network_writer::{DataTypeSerializer, NetworkWriter, WriteCompress};
use nalgebra::{Quaternion, UnitQuaternion, Vector3};
use std::fmt::Debug;
use std::ops::BitOrAssign;
use unity_mirror_macro::namespace;
use crate::unity_engine::Transform;

#[namespace(prefix = "Mirror")]
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct SyncData {
    // 改变的数据
    pub changed_data_byte: u8,
    // 位置
    pub position: Vector3<f32>,
    // 四元数
    pub quat_rotation: Quaternion<f32>,
    // 欧拉角
    pub vec_rotation: Vector3<f32>,
    // 缩放
    pub scale: Vector3<f32>,
}

impl DataTypeSerializer for SyncData {
    fn serialize(&self, writer: &mut NetworkWriter)
    where
        Self: Sized,
    {
        writer.write_blittable::<u8>(self.changed_data_byte);

        // 位置
        if (self.changed_data_byte & Changed::PosX.to_u8()) > 0 {
            writer.write_blittable::<f32>(self.position.x);
        }

        if (self.changed_data_byte & Changed::PosY.to_u8()) > 0 {
            writer.write_blittable::<f32>(self.position.y);
        }

        if (self.changed_data_byte & Changed::PosZ.to_u8()) > 0 {
            writer.write_blittable::<f32>(self.position.z);
        }

        // rotation
        if (self.changed_data_byte & Changed::CompressRot.to_u8()) > 0 {
            if (self.changed_data_byte & Changed::Rot.to_u8()) > 0 {
                self.quat_rotation.compress(writer);
            }
        } else {
            if (self.changed_data_byte & Changed::RotX.to_u8()) > 0 {
                writer.write_blittable::<f32>(self.vec_rotation.x);
            }

            if (self.changed_data_byte & Changed::RotY.to_u8()) > 0 {
                writer.write_blittable::<f32>(self.vec_rotation.y);
            }

            if (self.changed_data_byte & Changed::RotZ.to_u8()) > 0 {
                writer.write_blittable::<f32>(self.vec_rotation.z);
            }
        }

        // 缩放
        if (self.changed_data_byte & Changed::Scale.to_u8()) > 0 {
            writer.write_blittable::<f32>(self.scale.x);
            writer.write_blittable::<f32>(self.scale.y);
            writer.write_blittable::<f32>(self.scale.z);
        }
    }
}
impl DataTypeDeserializer for SyncData {
    fn deserialize(reader: &mut NetworkReader) -> Self
    where
        Self: Sized,
    {
        // 改变的数据
        let changed = reader.read_blittable::<u8>();

        // 位置
        let mut position = Vector3::new(0.0, 0.0, 0.0);
        if (changed & Changed::PosX.to_u8()) > 0 {
            position.x = reader.read_blittable::<f32>();
        }
        if changed & Changed::PosY.to_u8() > 0 {
            position.y = reader.read_blittable::<f32>();
        }
        if changed & Changed::PosZ.to_u8() > 0 {
            position.z = reader.read_blittable::<f32>();
        }

        // 欧拉角
        let mut vec_rotation = Vector3::new(0.0, 0.0, 0.0);
        // 四元数
        let mut quaternion = Quaternion::identity();

        if (changed & Changed::CompressRot.to_u8()) > 0 {
            if (changed & Changed::RotX.to_u8()) > 0 {
                let compress = Quaternion::decompress(reader);
            }
        } else {
            if changed & Changed::RotX.to_u8() > 0 {
                vec_rotation.x = reader.read_blittable::<f32>();
            }
            if changed & Changed::RotY.to_u8() > 0 {
                vec_rotation.y = reader.read_blittable::<f32>();
            }
            if changed & Changed::RotZ.to_u8() > 0 {
                vec_rotation.z = reader.read_blittable::<f32>();
            }
        }

        // 缩放
        let mut scale = Vector3::new(1.0, 1.0, 1.0);
        if changed & Changed::Scale.to_u8() == Changed::Scale.to_u8() {
            scale.x = reader.read_blittable::<f32>();
            scale.y = reader.read_blittable::<f32>();
            scale.z = reader.read_blittable::<f32>();
        }

        if (changed & Changed::CompressRot.to_u8()) > 0 {
            vec_rotation = Transform::quaternion_to_euler_angles(quaternion)
        } else {
            quaternion = Transform::euler_angles_to_quaternion(vec_rotation);
        }

        Self {
            changed_data_byte: changed,
            position,
            quat_rotation: quaternion,
            vec_rotation,
            scale,
        }
    }
}

#[allow(unused)]
impl SyncData {
    pub fn new(
        changed: u8,
        position: Vector3<f32>,
        quat_rotation: Quaternion<f32>,
        scale: Vector3<f32>,
    ) -> Self {
        let rotation = UnitQuaternion::from_quaternion(quat_rotation);
        Self {
            changed_data_byte: changed,
            position,
            quat_rotation,
            vec_rotation: Vector3::new(
                rotation.euler_angles().0,
                rotation.euler_angles().1,
                rotation.euler_angles().2,
            ),
            scale,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Changed {
    None = 0,
    PosX = 1,
    PosY = 2,
    PosZ = 4,
    CompressRot = 8,
    RotX = 16,   // 0x10
    RotY = 32,   // 0x20
    RotZ = 64,   // 0x40
    Scale = 128, // 0x80

    Pos = 0x07, // 0x07
    Rot = 0x70, // 0x70
}

impl Into<Changed> for u8 {
    fn into(self) -> Changed {
        match self {
            1 => Changed::PosX,
            2 => Changed::PosY,
            4 => Changed::PosZ,
            8 => Changed::CompressRot,
            16 => Changed::RotX,
            32 => Changed::RotY,
            64 => Changed::RotZ,
            128 => Changed::Scale,
            0x07 => Changed::Pos,
            0x70 => Changed::Rot,
            _ => Changed::None, // 默认值
        }
    }
}

impl Changed {
    pub fn to_u8(&self) -> u8 {
        *self as u8
    }
}

// 为 Changed 实现 BitOrAssign
impl BitOrAssign for Changed {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = (self.to_u8() | rhs as u8).into();
    }
}

#[cfg(test)]
mod tests {
    use crate::mirror::components::network_transform::transform_sync_data::SyncData;
    use crate::mirror::network_reader::{DataTypeDeserializer, NetworkReader};
    use crate::mirror::network_writer::NetworkWriter;
    use nalgebra::Quaternion;
    use crate::unity_engine::Transform;

    #[test]
    fn test_sync_data() {
        let mut writer = NetworkWriter::new();
        writer.write_blittable::<u8>(120);
        writer.write_blittable::<u8>(255);
        writer.write_blittable::<u8>(45);
        writer.write_blittable::<u8>(246);
        writer.write_blittable::<u8>(223);
        let mut reader = NetworkReader::new(writer.to_vec());
        let sync_data = SyncData::deserialize(&mut reader);
        println!("sync_data {:?}", sync_data);

        // 假设 Unity 四元数数据 (x, y, z, w)
        let (ux, uy, uz, uw) = (-0.00069f32, -0.16105, -0.00069, 0.98695);

        // 转换为 Rust 四元数（注意：Unity 中的顺序是 (x, y, z, w)）
        let quat = Quaternion::new(ux, uy, uz, uw);

        // 将四元数转换为欧拉角（ZXY 顺序）
        let a = Transform::quaternion_to_euler_angles(quat);

        println!(
            "欧拉角（单位：角度）: Yaw(Z) = {:.2}, Pitch(Y) = {:.2}, Roll(X) = {:.2}",
            a.x, a.y, a.z
        );

        // 将欧拉角转换回四元数
        let quat2 = Transform::euler_angles_to_quaternion(a);
        println!("转换回的四元数: {:?}", quat2);
    }
}
