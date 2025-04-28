use serde::Deserialize;

#[derive(Deserialize,Default)]
pub struct MetadataAsset {
    #[serde(rename = "assetId")]
    pub asset_id: u32,
    #[serde(rename = "assetPath")]
    pub asset_path: String,
}
