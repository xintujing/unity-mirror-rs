pub mod mirror;

use crate::mirror::core::network_behaviour::NetworkBehaviourTrait;
use crate::mirror::core::network_identity::network_identities;
use crate::mirror::core::network_reader::NetworkReader;
use unity_mirror_rs_macro::{command, component, MSync};

#[derive(Debug, MSync)]
pub struct MyStruct {
    #[sync_var]
    name: String,
    age: u64,
    #[sync_var]
    health: u32,
}

// 实现 NetworkBehaviourTrait
impl NetworkBehaviourTrait for MyStruct {
    fn sync_var_dirty_bits(&self) -> u64 {
        0
    }
}

#[component(namespace = "Mirror")]
impl MyStruct {
    #[command(requires_authority = true)]
    fn existing_method(&mut self, id: u32) {
        println!("组件名字: {}  参数 1: {}", self.name, id);

        // 测试自己找自己

        self.name = "组件 2".to_string();

        match network_identities().get_mut(&99) {
            None => {}
            Some(identity) => {
                let component = &mut identity.network_behaviours[0];
                println!("{:?}", component);
            }
        }
    }

    fn sync_var_dirty_bits(&self) -> u64 {
        0
    }
}

#[cfg(test)]
mod tests {
    use crate::mirror::core::network_connection_to_client::NetworkConnectionToClient;
    use crate::mirror::core::network_identity::{
        insert_network_identity, network_identities, NetworkIdentity, OwnedType, Visibility,
    };
    use crate::mirror::core::network_reader::NetworkReader;
    use crate::mirror::core::network_writer::{NetworkWriter, NetworkWriterTrait};
    use crate::mirror::core::remote_calls::{RemoteCallType, RemoteProcedureCalls};
    use crate::mirror::core::tools::stable_hash::StableHash;
    use crate::MyStruct;

    #[test]
    fn bb() {
        match "u64" {
            "String" => {
                println!("writer.write_string(self.name.clone());")
            }
            "str" => {
                println!("writer.write_str(self.name);")
            }
            _ => {
                println!("writer.write_blittable::<u64>(self.name);")
            }
        }
    }

    #[test]
    fn test() {
        let net_id = 99;
        let index = 0;

        let my_struct = MyStruct {
            name: "组件 1".to_string(),
            age: 0,
            health: 0,
        };

        let mut network_identity = NetworkIdentity {
            net_id: std::sync::Arc::new(parking_lot::RwLock::new(1)),
            conn_id: std::sync::Arc::new(parking_lot::RwLock::new(1)),
            observers: vec![],
            had_authority: true,
            scene_id: 1,
            asset_id: 1,
            is_owned: true,
            destroy_called: false,
            network_behaviours: vec![],
            scene_ids: dashmap::DashMap::new(),
            owned_type: OwnedType::Server,
            visibility: Visibility::Default,
            network_identity_serialization_tick: 1,
        };

        // 添加组件
        network_identity
            .network_behaviours
            .push(Box::new(my_struct));

        // 添加 NetworkIdentity
        insert_network_identity(net_id, network_identity);

        // 函数签名的稳定哈希
        let func_hash =
            "System.Void Mirror.MyStruct::ExistingMethod(System.UInt32)".get_fn_stable_hash_code();
        // 远程调用类型
        let remote_call_type = RemoteCallType::Command;
        // NetworkWriter
        let mut writer = NetworkWriter::new();
        writer.write_int(22);
        let mut reader = NetworkReader::new_with_bytes(writer.to_bytes());
        // NetworkConnectionToClient
        let mut connection_to_client = NetworkConnectionToClient::default();

        match network_identities().get_mut(&net_id) {
            None => {}
            Some(identity) => {
                let component = &mut identity.network_behaviours[index];
                let is_invoke = RemoteProcedureCalls::invoke(
                    func_hash,
                    remote_call_type,
                    &mut reader,
                    component,
                    &mut connection_to_client,
                );
                assert_eq!(is_invoke, true);
            }
        }
    }
}
