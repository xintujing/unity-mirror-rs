use crate::mirror::core::network_behaviour::NetworkBehaviourType;
use dashmap::DashMap;
use std::collections::HashMap;
use std::ptr;
use std::sync::atomic::AtomicPtr;
use std::sync::Arc;

static mut NETWORK_IDENTITIES: AtomicPtr<HashMap<u32, NetworkIdentity>> =
    AtomicPtr::new(ptr::null_mut());

#[ctor::ctor]
fn __init__() {
    let map = Box::new(HashMap::new());
    let ptr = Box::into_raw(map);
    #[allow(static_mut_refs)]
    unsafe {
        NETWORK_IDENTITIES.store(ptr, std::sync::atomic::Ordering::SeqCst);
    }
}

pub fn insert_network_identity(net_id: u32, network_identity: NetworkIdentity) {
    #[allow(static_mut_refs)]
    unsafe {
        let ptr = NETWORK_IDENTITIES.load(std::sync::atomic::Ordering::SeqCst);
        let map = &mut *ptr;
        map.insert(net_id, network_identity);
    }
}

pub fn network_identities() -> &'static mut HashMap<u32, NetworkIdentity> {
    #[allow(static_mut_refs)]
    unsafe {
        let ptr = NETWORK_IDENTITIES.load(std::sync::atomic::Ordering::SeqCst);
        &mut *ptr
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum OwnedType {
    #[default]
    Client,
    Server,
}
#[derive(Debug, Default, PartialEq, Eq)]
pub enum Visibility {
    #[default]
    Default,
    ForceHidden,
    ForceShown,
}

#[derive(Debug, Default)]
pub struct NetworkIdentity {
    pub net_id: Arc<parking_lot::RwLock<u32>>,
    pub conn_id: Arc<parking_lot::RwLock<u64>>,
    /// The set of network connections (players) that can see this object.
    pub observers: Vec<u64>,

    pub had_authority: bool,

    /// Unique identifier for NetworkIdentity objects within a scene, used for spawning scene objects.
    // persistent scene id <sceneHash/32,sceneId/32> (see AssignSceneID comments)
    pub scene_id: u64,

    // assetId used to spawn prefabs across the network.
    // originally a Guid, but a 4 byte uint is sufficient
    // (as suggested by james)
    //
    // it's also easier to work with for serialization etc.
    // serialized and visible in inspector for easier debugging
    //
    // The AssetId trick:
    //   Ideally we would have a serialized 'Guid m_AssetId' but Unity can't
    //   serialize it because Guid's internal bytes are private
    //
    //   Using just the Guid string would work, but it's 32 chars long and
    //   would then be sent over the network as 64 instead of 16 bytes
    //
    // => The solution is to serialize the string internally here and then
    //    use the real 'Guid' type for everything else via .assetId
    pub asset_id: u32,

    /// isOwned is true on the client if this NetworkIdentity is one of the .owned entities of our connection on the server.
    // for example: main player & pets are owned. monsters & npcs aren't.
    pub is_owned: bool,

    // Set before Destroy is called so that OnDestroy doesn't try to destroy
    // the object again
    pub destroy_called: bool,

    // NetworkBehaviours are a list of components
    pub network_behaviours: Vec<NetworkBehaviourType>,

    // Keep track of all sceneIds to detect scene duplicates
    pub scene_ids: DashMap<u64, NetworkIdentity>,

    /// isOwned is true on the client if this NetworkIdentity is one of the .owned entities of our connection on the server.
    // for example: main player & pets are owned. monsters & npcs aren't.
    pub owned_type: OwnedType,

    // current visibility
    //
    // Default = use interest management
    // ForceHidden = useful to hide monsters while they respawn etc.
    // ForceShown = useful to have score NetworkIdentities that always broadcast
    // to everyone etc.
    pub visibility: Visibility,

    // broadcasting serializes all entities around a player for each player.
    // we don't want to serialize one entity twice in the same tick.
    // so we cache the last serialization and remember the timestamp so we
    // know which Update it was serialized.
    // (timestamp is the same while inside Update)
    // => this way we don't need to pool thousands of writers either.
    // => way easier to store them per object
    pub network_identity_serialization_tick: i32,
    // pub owner_writer: NetworkWriter,
    // pub observers_writer: NetworkWriter,
}

impl NetworkIdentity {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mirror::core::network_behaviour::NetworkBehaviourTrait;

    #[derive(Debug)]
    struct Test {
        id: u32,
    }

    impl NetworkBehaviourTrait for Test {
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    #[test]
    fn test_network_identities() {
        let mut network_identity = NetworkIdentity {
            net_id: Arc::new(parking_lot::RwLock::new(1)),
            conn_id: Arc::new(parking_lot::RwLock::new(1)),
            observers: vec![1],
            had_authority: true,
            scene_id: 1,
            asset_id: 1,
            is_owned: true,
            destroy_called: false,
            network_behaviours: vec![],
            scene_ids: DashMap::new(),
            owned_type: OwnedType::Client,
            visibility: Visibility::Default,
            network_identity_serialization_tick: 1,
        };

        let test = Test { id: 1 };

        network_identity.network_behaviours.push(Box::new(test));

        insert_network_identity(1, network_identity);

        network_identities()
            .get_mut(&1)
            .unwrap()
            .network_behaviours
            .iter_mut()
            .for_each(|v| {
                v.as_any_mut().downcast_mut::<Test>().unwrap().id += 1;
                println!(
                    "test - {}",
                    v.as_any_mut().downcast_mut::<Test>().unwrap().id
                );
                network_identities()
                    .get_mut(&1)
                    .unwrap()
                    .network_behaviours
                    .iter_mut()
                    .for_each(|v| {
                        v.as_any_mut().downcast_mut::<Test>().unwrap().id += 1;
                        println!(
                            "test - {}",
                            v.as_any_mut().downcast_mut::<Test>().unwrap().id
                        );
                    });
            });
    }
}
