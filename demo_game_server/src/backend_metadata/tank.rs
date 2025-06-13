use serde::Deserialize;
use unity_mirror_rs::commons::Object;
use unity_mirror_rs::macro_namespace::*;
use unity_mirror_rs::metadata_settings::metadata_asset::MetadataAsset;
use unity_mirror_rs::metadata_settings::metadata_transform::MetadataTransform;
use unity_mirror_rs::metadata_settings::MetadataNetworkBehaviourWrapper;
use unity_mirror_rs::metadata_settings::Settings;
use unity_mirror_rs::settings_wrapper_register;

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
