use crate::metadata_settings::unity::metadata_component::MetadataComponentWrapper;
use crate::metadata_settings::unity::metadata_transform::MetadataTransform;
use serde::Deserialize;

#[derive(Deserialize)]
#[allow(unused)]
pub struct MetadataPrefab {
    pub id: i32,
    pub name: String,
    pub tag: String,
    pub layer: i32,
    #[serde(rename = "isStatic")]
    pub is_static: bool,
    #[serde(rename = "isActive")]
    pub is_active: bool,
    pub transform: MetadataTransform,
    pub components: MetadataComponentWrapper,
    pub children: Vec<MetadataPrefab>,
    #[serde(rename = "assetId")]
    pub asset_id: u32,
    #[serde(rename = "assetPath")]
    pub asset_path: String,
}
