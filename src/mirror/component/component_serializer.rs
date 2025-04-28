use crate::mirror::component::component::Component;
use crate::mirror::component::component_basic::ComponentBasic;
use crate::mirror::components::network_behaviour::NetworkBehaviour;
use crate::mirror::network_writer::NetworkWriter;

#[allow(unused)]
pub trait ComponentSerializer: ComponentOnSerializer {
    fn serialize(&self, writer: &mut NetworkWriter, initial: bool) {
        let header_position = writer.get_position();
        writer.write_blittable::<u8>(0);
        let content_position = writer.get_position();

        self.on_serialize(writer, initial);

        let end_position = writer.get_position();
        writer.set_position(header_position);
        let size = (end_position - content_position) as u8;
        let safety = size & 0xFF;
        writer.write_blittable::<u8>(safety);
        writer.set_position(end_position);
    }
}

impl<T> ComponentSerializer for T where T: Component {}

pub trait ComponentOnSerializer {
    fn on_serialize(&self, writer: &mut NetworkWriter, initial: bool) {
        self.serialize_sync_objects(writer, initial);
        self.serialize_sync_variables(writer, initial);
    }
    fn serialize_sync_objects(&self, writer: &mut NetworkWriter, initial: bool) {
        // if initial {
        //     NetworkBehaviour::serialize_objects_all(&self.id(), writer)
        // } else {
        //     NetworkBehaviour::serialize_objects_delta(&self.id(), writer)
        // }
    }
    fn serialize_sync_variables(&self, writer: &mut NetworkWriter, initial: bool) {}
}
