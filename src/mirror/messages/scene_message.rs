use crate::commons::object::Object;
use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer};
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use crate::mirror::stable_hash::StableHash;
use unity_mirror_macro::{namespace, Message};

#[derive(Debug, PartialEq, Clone, Copy, Default)]
#[repr(u8)]
pub enum SceneOperation {
    #[default]
    Normal = 0,
    LoadAdditive = 1,
    UnloadAdditive = 2,
}
impl SceneOperation {
    pub fn from(value: u8) -> SceneOperation {
        match value {
            0 => SceneOperation::Normal,
            1 => SceneOperation::LoadAdditive,
            2 => SceneOperation::UnloadAdditive,
            _ => SceneOperation::Normal,
        }
    }
    pub fn to_u8(&self) -> u8 {
        *self as u8
    }
}

#[namespace(prefix = "Mirror")]
#[derive(Debug, PartialEq, Clone, Default, Message)]
pub struct SceneMessage {
    pub scene_name: String,
    pub operation: SceneOperation,
    pub custom_handling: bool,
}

impl SceneMessage {
    #[allow(unused)]
    pub fn new(
        scene_name: String,
        operation: SceneOperation,
        custom_handling: bool,
    ) -> SceneMessage {
        SceneMessage {
            scene_name,
            operation,
            custom_handling,
        }
    }
}

impl MessageSerializer for SceneMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter)
    where
        Self: Sized,
    {
        writer.write_blittable(Self::get_full_name().hash16());
        writer.write_str(self.scene_name.as_str());
        writer.write_blittable(self.operation.to_u8());
        writer.write_blittable(self.custom_handling);
    }
}

impl MessageDeserializer for SceneMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self
    where
        Self: Sized,
    {
        let scene_name = reader.read_string();
        let operation = SceneOperation::from(reader.read_blittable());
        let custom_handling = reader.read_blittable();
        Self {
            scene_name,
            operation,
            custom_handling,
        }
    }
}
