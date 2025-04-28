use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use serde::Deserialize;
use unity_mirror_rs_macro::{namespace, settings_wrapper_register};

#[namespace("Mirror", rename = "NetworkRoomPlayer")]
#[derive(Deserialize)]
pub struct MetadataNetworkRoomPlayer {}
settings_wrapper_register!(MetadataNetworkRoomPlayer as MetadataNetworkBehaviourWrapper);
