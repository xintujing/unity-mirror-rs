use nalgebra::{Quaternion, Vector3};
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug, Default)]
#[allow(unused)]
pub struct MetadataTransform {
    position: [f32; 3],
    #[serde(rename = "localPosition")]
    local_position: [f32; 3],
    rotation: [f32; 4],
    #[serde(rename = "localRotation")]
    local_rotation: [f32; 4],
    #[serde(rename = "localScale")]
    local_scale: [f32; 3],
    #[serde(rename = "instanceId")]
    instance_id: i32,
    r#type: String,
    #[serde(rename = "assetId")]
    asset_id: u32,
    #[serde(rename = "assetPath")]
    asset_path: String,
}

// impl Into<Transform> for MetadataTransform {
//     fn into(self) -> Transform {
//         Transform {
//             position: Vector3::new(self.position[0], self.position[1], self.position[2]),
//             rotation: Quaternion::new(
//                 self.rotation[3],
//                 self.rotation[0],
//                 self.rotation[1],
//                 self.rotation[2],
//
//             ),
//             local_position: Vector3::new(
//                 self.local_position[0],
//                 self.local_position[1],
//                 self.local_position[2],
//             ),
//             local_rotation: Quaternion::new(
//                 self.local_rotation[3],
//                 self.local_rotation[0],
//                 self.local_rotation[1],
//                 self.local_rotation[2],
//             ),
//             local_scale: Vector3::new(
//                 self.local_scale[0],
//                 self.local_scale[1],
//                 self.local_scale[2],
//             ),
//         }
//     }
// }