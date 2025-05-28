use crate::commons::object::Object;
use crate::mirror::network_connection::NetworkConnection;
use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer, OnMessageHandler};
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use crate::mirror::stable_hash::StableHash;
use crate::mirror::transport::TransportChannel;
use unity_mirror_macro::{namespace, MessageRegistry};
use crate::commons::revel_arc::RevelArc;

#[namespace(prefix = "Mirror")]
#[derive(Debug, PartialEq, Clone, Default, MessageRegistry)]
pub struct CommandMessage {
    pub net_id: u32,
    pub component_index: u8,
    pub function_hash: u16,
    pub payload: Vec<u8>,
}

impl CommandMessage {
    #[allow(unused)]
    pub(crate) fn new(
        net_id: u32,
        component_index: u8,
        function_hash: u16,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            net_id,
            component_index,
            function_hash,
            payload,
        }
    }
    #[allow(unused)]
    pub fn get_payload_content(&self) -> Vec<u8> {
        self.payload[4..].to_vec()
    }
}
#[allow(unused)]
impl OnMessageHandler for CommandMessage {
    fn handle(&self, conn: &mut RevelArc<NetworkConnection>, channel: TransportChannel) {
        // let requires_authority =
        //     RemoteProcedureCalls::command_requires_authority(&self.function_hash);

        // // 将获取的 component 和 identity.name 提取到闭包函数外部
        // // 避免 Identity::handle_remote_call 内部无法获取identity强引用导致死锁
        // let mut component = None;
        // let mut identity_name = "".to_string();
        // if !NetworkServer::get_identity(&self.net_id, |mut identity| {
        //     identity_name = identity.name();
        //     component = identity.get_component(uc_conn, self.component_index);
        // }) {
        //     // log::error!(
        //     //     "Spawned object not found when handling Command message netId = {}",
        //     //     self.net_id
        //     // );
        //
        //     println!(
        //         "Spawned object not found when handling Command message netId = {}",
        //         self.net_id
        //     );
        //     return;
        // }
        //
        // match component {
        //     None => {
        //         println!(
        //             "Command message invoke failed, component not found! [conn_id={}] [netId={}] [component_index={}] [function_hash={}].",
        //             uc_conn.get().conn_id,
        //             self.net_id,
        //             self.component_index,
        //             self.function_hash
        //         );
        //     }
        //     Some(component) => {
        //         NetworkReaderPool::get_with_bytes_return(&self.payload, |reader| {
        //             Identity::handle_remote_call(
        //                 component,
        //                 uc_conn,
        //                 &self.function_hash,
        //                 RemoteCallType::Command,
        //                 reader,
        //                 self.net_id,
        //                 &identity_name,
        //             );
        //         });
        //     }
        // }
    }
}

impl MessageSerializer for CommandMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter)
    where
        Self: Sized,
    {
        writer.write_blittable(Self::get_full_name().hash16());
        writer.write_blittable_compress(self.net_id);
        writer.write_blittable(self.component_index);
        writer.write_blittable(self.function_hash);
        writer.write_slice_and_size(self.payload.as_slice());
    }
}

impl MessageDeserializer for CommandMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self
    where
        Self: Sized,
    {
        let net_id = reader.read_blittable_compress();
        let component_index = reader.read_blittable();
        let function_hash = reader.read_blittable();
        let payload = reader.read_slice_and_size();
        Self {
            net_id,
            component_index,
            function_hash,
            payload: payload.to_vec(),
        }
    }
}
