use std::collections::HashMap;
use crate::metadata_settings::mirror::metadata_network_manager::MetadataNetworkManagerWrapper;
use crate::metadata_settings::unity::metadata_prefab::MetadataPrefab;
use crate::metadata_settings::unity::metadata_scene::MetadataScene;
use indexmap::IndexMap;
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::fs;
use std::sync::Arc;

pub struct MetadataLoader;

const FILE_PATH: &str = "metadata_settings.json";

static mut METADATA: Lazy<Arc<Metadata>> = Lazy::new(|| MetadataLoader::load(FILE_PATH));

impl MetadataLoader {
    pub fn load(file_path: &str) -> Arc<Metadata> {
        match fs::read_to_string(file_path) {
            Ok(string_value) => match serde_json::from_str::<Metadata>(&string_value) {
                Ok(value) => Arc::new(value),
                Err(err) => {
                    panic!("Failed to parse the configuration file:\n{}", err);
                }
            },
            Err(err) => {
                panic!("Failed to open configuration file '{FILE_PATH}'\n{}", err)
            }
        }
    }
}

#[derive(Deserialize)]
pub struct Metadata {
    pub prefabs: HashMap<String, MetadataPrefab>,
    pub scenes: HashMap<String, HashMap<String, MetadataPrefab>>,
    #[serde(rename = "networkManagers")]
    pub network_managers: HashMap<String, MetadataNetworkManagerWrapper>,
}

impl Metadata {
    pub(crate) fn get_scene(path: &str) -> Option<&HashMap<String, MetadataPrefab>> {
        #[allow(static_mut_refs)]
        unsafe { METADATA.scenes.get(path) }
    }
}

impl Metadata {
    pub fn get_prefab(prefab_path: &str) -> Option<&MetadataPrefab> {
        #[allow(static_mut_refs)]
        unsafe { METADATA.prefabs.get(prefab_path) }
    }

    pub fn get_network_manager(prefab_path: &str) -> Option<&MetadataNetworkManagerWrapper> {
        #[allow(static_mut_refs)]
        unsafe { METADATA.network_managers.get(prefab_path) }
    }
}

#[cfg(test)]
mod metadata_test {

    // #[test]
    // fn test() {
    //     if let Some(prefab) = Metadata::get_prefab("Assets/Prefabs/Tank.prefab") {
    //         for metadata_network_identity_wrapper in
    //             prefab.components.list::<MetadataNetworkIdentityWrapper>()
    //         {
    //             let metadata_network_identity =
    //                 metadata_network_identity_wrapper.get::<MetadataNetworkIdentity>();
    //         }
    //     }
    //
    //     if let Some(prefab) = Metadata::get_prefab("Assets/Prefabs/Projectile.prefab") {
    //         for metadata_rigid_body_wrapper in prefab.components.list::<MetadataRigidBodyWrapper>()
    //         {
    //             let metadata_rigid_body = metadata_rigid_body_wrapper.get::<MetadataRigidBody>();
    //             println!("{}", metadata_rigid_body.angular_drag);
    //         }
    //     }
    //
    //     if let Some(prefab) = Metadata::get_prefab("Assets/Prefabs/Projectile.prefab") {
    //         let vec = prefab.components.list::<MetadataColliderWrapper>();
    //         for metadata_collider_wrapper in vec {
    //             let metadata_collider = metadata_collider_wrapper.get::<MetadataCapsuleCollider>();
    //             println!("{}", metadata_collider.height);
    //         }
    //     }
    //
    //     if let Some(network_manager) =
    //         Metadata::get_network_manager("Assets/Prefabs/NetworkManager.prefab")
    //     {
    //         let metadata_network_manager = network_manager.get::<MetadataNetworkManager>();
    //         println!("{}", metadata_network_manager.player_prefab.asset_path);
    //     }
    // }
}
