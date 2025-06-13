use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use serde::Deserialize;
use crate::{namespace, settings_wrapper_register};
use crate::commons::Object;
use crate::metadata_settings::Settings;

#[namespace(prefix = "Mirror", rename = "NetworkRoomPlayer")]
#[derive(Deserialize, Clone)]
pub struct MetadataNetworkRoomPlayer {}
settings_wrapper_register!(MetadataNetworkRoomPlayer as MetadataNetworkBehaviourWrapper);
