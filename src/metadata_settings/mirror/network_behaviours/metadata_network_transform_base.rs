use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::metadata_settings::unity::metadata_transform::MetadataTransform;
use serde::Deserialize;
use serde_repr::Deserialize_repr;
use unity_mirror_macro::{namespace, settings_wrapper_register};

#[derive(Deserialize_repr, Debug, Clone)]
#[repr(u8)]
pub enum CoordinateSpace {
    Local = 0,
    World = 1,
}

#[namespace(prefix = "Mirror", rename = "NetworkTransformBase")]
#[derive(Deserialize, Debug, Clone)]
pub struct MetadataNetworkTransformBase {
    pub target: MetadataTransform,
    #[serde(rename = "syncPosition")]
    pub sync_position: bool,
    #[serde(rename = "syncRotation")]
    pub sync_rotation: bool,
    #[serde(rename = "syncScale")]
    pub sync_scale: bool,
    #[serde(rename = "onlySyncOnChange")]
    pub only_sync_on_change: bool,
    #[serde(rename = "compressRotation")]
    pub compress_rotation: bool,
    #[serde(rename = "interpolatePosition")]
    pub interpolate_position: bool,
    #[serde(rename = "interpolateRotation")]
    pub interpolate_rotation: bool,
    #[serde(rename = "interpolateScale")]
    pub interpolate_scale: bool,
    #[serde(rename = "coordinateSpace")]
    pub coordinate_space: CoordinateSpace,
    #[serde(rename = "timelineOffset")]
    pub timeline_offset: bool,
}
settings_wrapper_register!(MetadataNetworkTransformBase as MetadataNetworkBehaviourWrapper);
