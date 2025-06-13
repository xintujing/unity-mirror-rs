use serde::Deserialize;
use unity_mirror_rs::macro_namespace::*;
use unity_mirror_rs::metadata_settings::MetadataNetworkBehaviourWrapper;
use unity_mirror_rs::metadata_settings::Settings;
use unity_mirror_rs::settings_wrapper_register;

#[namespace(rename = "Projectile")]
#[derive(Deserialize, Clone)]
pub struct MetadataProjectile {}
settings_wrapper_register!(MetadataProjectile as MetadataNetworkBehaviourWrapper);
