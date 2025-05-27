use serde::Deserialize;

#[derive(Deserialize, Clone, Debug, Default)]
#[allow(unused)]
pub struct MetadataTransform {
    pub position: [f32; 3],
    #[serde(rename = "localPosition")]
    pub local_position: [f32; 3],
    pub rotation: [f32; 4],
    #[serde(rename = "localRotation")]
    pub local_rotation: [f32; 4],
    #[serde(rename = "localScale")]
    pub local_scale: [f32; 3],
    #[serde(rename = "instanceId")]
    pub instance_id: i32,
    pub r#type: String,
    #[serde(rename = "assetId")]
    pub asset_id: u32,
    #[serde(rename = "assetPath")]
    pub asset_path: String,
}
