use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use serde::Deserialize;
use unity_mirror_macro::{namespace, settings_wrapper_register};

#[namespace(prefix = "Mirror", rename = "NetworkRoomPlayer")]
#[derive(Deserialize, Clone)]
pub struct MetadataNetworkRoomPlayer {}
settings_wrapper_register!(MetadataNetworkRoomPlayer as MetadataNetworkBehaviourWrapper);
