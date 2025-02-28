pub mod mirror;

use crate::mirror::core::network_behaviour::NetworkBehaviourTrait;
use crate::mirror::core::network_identity::network_identities;
use crate::mirror::core::network_reader::NetworkReader;
use serde::Deserialize;
use std::any::Any;
use std::fmt::{Debug, Formatter};
use unity_mirror_rs_macro::{command, component, namespace, rpc, MSync, NetworkMessage};

#[derive(Debug, MSync)]
pub struct MyStruct {
    #[sync_var]
    name: String,
    age: u64,
    #[sync_var]
    health: u32,
}

// fn check_implements_my_trait<T: CustomDataType>(_: &T) -> bool {
//     true
// }
impl NetworkBehaviourTrait for MyStruct {
    fn sync_var_dirty_bits(&self) -> u64 {
        0
    }
}

// #[derive(Debug)]
// #[custom_data_type(namespace = "Mirror.Model.")]
#[namespace(value = "Mirror.Authenticators", full_path = "BasicAuthenticator+")]
pub struct AuthResponseMessage {
    q: bool,
}
// Mirror.Authenticators.BasicAuthenticator+AuthResponseMessage

// 实现 NetworkBehaviourTrait

#[component(namespace = "Mirror")]
impl MyStruct {
    #[command(requires_authority = true)]
    fn existing_method(&mut self, id: AuthResponseMessage, _pa: bool) {
        // println!("组件名字: {}  参数 1: {:?}", self.name, id);

        id.q;
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

    #[rpc]
    fn test_rpc1(&self) {}
    #[rpc]
    fn test_rpc2(&self) {}
    #[rpc]
    fn test_rpc3(&self) {}

    fn sync_var_dirty_bits(&self) -> u64 {
        0
    }
}

fn test2() {
    // AAA {}
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
    fn a() {}

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
            "System.Void Mirror.MyStruct::ExistingMethod(Mirror.Authenticators.BasicAuthenticator+AuthResponseMessage,System.Boolean)".get_fn_stable_hash_code();
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
                    component,
                    &mut reader,
                    &mut connection_to_client,
                );
                assert_eq!(is_invoke, true);
            }
        }
    }
}

// mod tests1 {
//
//     trait Behaviour {
//         fn get_base(&self) -> Option<Box<&mut dyn Behaviour>>;
//     }
//
//     struct NetworkRoomPlayer {
//         id: u64,
//     }
//
//     trait NetworkRoomPlayerTrait: Behaviour {
//         fn on_ready(&self);
//
//         fn on_stop(&self);
//     }
//
//     impl Behaviour for NetworkRoomPlayer {
//         fn get_base(&self) -> Option<Box<&mut dyn Behaviour>> {
//             None
//         }
//     }
//     impl NetworkRoomPlayerTrait for NetworkRoomPlayer {
//         fn on_ready(&self) {
//             println!("base on_ready read");
//             self.on_stop()
//         }
//
//         fn on_stop(&self) {
//             todo!()
//         }
//     }
//
//     struct MyNetworkRoomPlayer {
//         base: NetworkRoomPlayer,
//     }
//
//     impl Behaviour for MyNetworkRoomPlayer {
//         fn get_base(&mut self) -> Option<Box<&mut dyn Behaviour>> {
//             Some(Box::new(&mut self.base))
//         }
//     }
//
//     impl NetworkRoomPlayerTrait for MyNetworkRoomPlayer {
//         fn on_ready(&self) {
//             self.base.on_ready()
//         }
//
//         fn on_stop(&self) {
//             println!("MyNetworkRoomPlayer on_stop read");
//         }
//     }
// }

mod tests2 {

    trait A001T {
        fn a001(&self);

        fn a002(&self);
    }

    struct A001 {
        id: u64,
    }

    impl A001T for A001 {

        fn a001(&self) {
            self.a002();
        }

        fn a002(&self) {
            println!("a002 1");
        }
    }

    struct A002 {
        a001: A001,
        id: u64,
    }

    impl A001T for A002 {

        fn a001(&self) {
            self.a001.a001();
        }

        fn a002(&self) {
            println!("a002 2");
        }
    }

    #[test]
    fn test() {
        let a002 = A002 {
            a001: A001 { id: 1 },
            id: 1,
        };

        a002.a001();
    }
}
