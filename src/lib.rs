pub mod mirror;

use crate::mirror::core::network_behaviour::NetworkBehaviourTrait;
use crate::mirror::core::network_identity::network_identities;
use crate::mirror::core::network_reader::NetworkReader;
use nalgebra::Quaternion;
use serde::Deserialize;
use std::any::Any;
use std::collections::HashMap;
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

#[namespace(value = "qwer")]
#[derive(Debug, Default)]
struct Vector3;

#[derive(Default, Debug)]
struct MyType {
    a: i8,
    b: i16,
    c: String,
    d: Vec<u8>,
    e: HashMap<String, i64>,
    f: Vector3,
    g: nalgebra::Vector3<f64>,
    h: nalgebra::Vector4<f64>,
    i: Quaternion<f64>,
    j: (i8, u8, bool),
}

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

    #[command(requires_authority = true)]
    fn test_command1(
        &self,
        bs: &[u8],
        bs2: Vec<String>,
        /* b3: HashMap<String, i64>,*/ b4: &[u8; 32],
        b5: nalgebra::Vector3<f64>,
        b6: Vector3,
    ) {
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
    use crate::{MyStruct, MyType, Vector3};
    use std::collections::HashMap;

    #[test]
    fn a() {
        let mut a = MyType::default();

        let l = s(a);
        println!("{}", l) // 输出: 24
    }

    fn s<T>(t: T) -> usize {
        size_of::<T>()
        // size_of_val::<T>(&t)
    }

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

        writer.write_blittable(crate::MyType {
            a: 6,
            b: 7,
            c: "999".to_string(),
            d: vec![22, 255, 231],
            e: || -> HashMap<String, i64> {
                let mut m = HashMap::new();
                m.insert("key".to_string(), 123);
                m
            }(),
            f: Vector3 {},
            g: Default::default(),
            h: Default::default(),
            i: Default::default(),
            j: (6, 6, true),
        });

        let hex_str = writer
            .to_bytes()
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<String>>();

        println!("hex_str: {}", hex_str.join(" "));


        let mut reader = NetworkReader::new_with_bytes(writer.to_bytes());

        let _my_type: MyType = reader.read_blittable();
        println!("_my_type: {:?}", _my_type);

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
                // assert_eq!(is_invoke, true);
            }
        }
    }
}
