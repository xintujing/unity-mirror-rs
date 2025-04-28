use crate::mirror::component::component::Component;
use crate::mirror::component::component_basic::ComponentBasic;
use crate::mirror::components::network_behaviour::NetworkBehaviour;
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;

#[allow(unused)]
pub trait ComponentDeserializer: ComponentOnDeserializer {
    fn deserialize(&self, reader: &mut NetworkReader, initial: bool) -> bool {
        let mut result = true;
        let safety = reader.read_blittable::<u8>();
        let chunk_start = reader.get_position();

        self.on_deserialize(reader, initial);

        let size = reader.get_position() - chunk_start;
        let size_hash = (size as u8) & 0xFF;
        if size_hash != safety {
            log::error!(
                "{}",
                format!(
                    "Deserialize failed. Size mismatch. Expected: {}, Received: {}",
                    size_hash, safety
                )
            );
            let corrected_size =
                crate::mirror::components::network_behaviour::NetworkBehaviour::error_correction(
                    size, safety,
                );
            reader.set_position(chunk_start + corrected_size);
            result = false;
        }
        result
    }
}

impl<T> ComponentDeserializer for T where T: Component {}

pub trait ComponentOnDeserializer {
    fn on_deserialize(&self, reader: &mut NetworkReader, initial: bool) {
        self.deserialize_sync_objects(reader, initial);
        self.deserialize_sync_variables(reader, initial);
    }
    fn deserialize_sync_objects(&self, reader: &mut NetworkReader, initial: bool) {
        // if initial {
        //     NetworkBehaviour::deserialize_objects_all(&self.id(), reader)
        // } else {
        //     NetworkBehaviour::deserialize_objects_delta(&self.id(), reader)
        // }
    }
    fn deserialize_sync_variables(&self, reader: &mut NetworkReader, initial: bool) {}
}
