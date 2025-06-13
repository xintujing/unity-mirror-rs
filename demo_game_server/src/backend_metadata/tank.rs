use serde::Deserialize;
use unity_mirror_rs::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use unity_mirror_rs::metadata_settings::unity::metadata_asset::MetadataAsset;
use unity_mirror_rs::metadata_settings::unity::metadata_transform::MetadataTransform;
use unity_mirror_rs::unity_mirror_macro_rs::{namespace, settings_wrapper_register};

#[namespace(rename = "Tank")]
#[derive(Deserialize, Clone)]
pub struct MetadataTank {
    pub turret: MetadataTransform,
    #[serde(rename = "projectilePrefab")]
    pub projectile_prefab: MetadataAsset,
    #[serde(rename = "projectileMount")]
    pub projectile_mount: MetadataTransform,
    pub health: i32,
}
settings_wrapper_register!(MetadataTank as MetadataNetworkBehaviourWrapper);
