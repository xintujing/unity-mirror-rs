#![allow(unused)]
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::unity::metadata_transform::MetadataTransform;
use crate::unity_engine::GameObject;
use nalgebra::{Matrix3, Matrix4, Quaternion, Translation3, UnitQuaternion, Vector3};

#[derive(Default, Clone)]
pub struct Transform {
    pub instance_id: i32,

    pub parent: RevelWeak<Transform>,
    pub children: Vec<RevelWeak<Transform>>,

    pub game_object: RevelWeak<GameObject>,
    pub position: Vector3<f32>,
    pub local_position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub local_rotation: Quaternion<f32>,
    pub local_scale: Vector3<f32>,
}

impl Transform {
    pub fn new_with_metadata(metadata: &MetadataTransform) -> Self {
        Transform {
            instance_id: metadata.instance_id,
            parent: RevelWeak::default(),
            children: vec![],
            game_object: RevelWeak::default(),
            position: Vector3::new(
                metadata.position[0],
                metadata.position[1],
                metadata.position[2],
            ),
            rotation: Quaternion::new(
                metadata.rotation[3],
                metadata.rotation[0],
                metadata.rotation[1],
                metadata.rotation[2],
            ),
            local_position: Vector3::new(
                metadata.local_position[0],
                metadata.local_position[1],
                metadata.local_position[2],
            ),
            local_rotation: Quaternion::new(
                metadata.local_rotation[3],
                metadata.local_rotation[0],
                metadata.local_rotation[1],
                metadata.local_rotation[2],
            ),
            local_scale: Vector3::new(
                metadata.local_scale[0],
                metadata.local_scale[1],
                metadata.local_scale[2],
            ),
        }
    }
}

impl Transform {
    /// 将弧度转换为角度，并限制到 [0, 360) 范围
    fn radians_to_degrees_positive(radians: f32) -> f32 {
        let degrees = radians.to_degrees();
        if degrees < 0.0 {
            degrees + 360.0
        } else {
            degrees
        }
    }
    /// 四元数转欧拉角 (ZYX 顺序)
    pub(crate) fn quaternion_to_euler_angles(quat: Quaternion<f32>) -> Vector3<f32> {
        let (x, y, z, w) = (quat.w, quat.i, quat.j, quat.k);
        let ysqr = y * y;
        // 计算 Roll (X-axis rotation)
        let t0 = 2.0 * (w * x + y * z);
        let t1 = 1.0 - 2.0 * (x * x + ysqr);
        let roll = t0.atan2(t1); // 绕 X 轴的旋转
        // 计算 Pitch (Y-axis rotation)
        let t2 = 2.0 * (w * y - z * x);
        let t2 = t2.clamp(-1.0, 1.0); // 限制在 [-1, 1] 范围
        let pitch = t2.asin(); // 绕 Y 轴的旋转
        // 计算 Yaw (Z-axis rotation)
        let t3 = 2.0 * (w * z + x * y);
        let t4 = 1.0 - 2.0 * (ysqr + z * z);
        let yaw = t3.atan2(t4); // 绕 Z 轴的旋转
        // 转换为角度并限制到 [0, 360)
        Vector3::new(
            Self::radians_to_degrees_positive(roll),
            Self::radians_to_degrees_positive(pitch),
            Self::radians_to_degrees_positive(yaw),
        )
    }

    // 辅助函数：将角度转换为弧度
    fn degrees_to_radians(degrees: f32) -> f32 {
        degrees * std::f32::consts::PI / 180.0
    }
    pub(crate) fn euler_angles_to_quaternion(euler: Vector3<f32>) -> Quaternion<f32> {
        // 将欧拉角从角度转换为弧度
        let roll = Self::degrees_to_radians(euler.x);
        let pitch = Self::degrees_to_radians(euler.y);
        let yaw = Self::degrees_to_radians(euler.z);
        // 计算半角
        let cy = (yaw * 0.5).cos();
        let sy = (yaw * 0.5).sin();
        let cp = (pitch * 0.5).cos();
        let sp = (pitch * 0.5).sin();
        let cr = (roll * 0.5).cos();
        let sr = (roll * 0.5).sin();
        // 按公式计算四元数分量
        let w = cr * cp * cy + sr * sp * sy;
        let x = sr * cp * cy - cr * sp * sy;
        let y = cr * sp * cy + sr * cp * sy;
        let z = cr * cp * sy - sr * sp * cy;
        Quaternion::new(w, x, y, z)
    }
}

impl Transform {
    /// 计算全局变换矩阵
    fn to_global_matrix(&self) -> Matrix4<f32> {
        let translation = Translation3::from(self.position).to_homogeneous();
        let rotation = UnitQuaternion::from_quaternion(self.rotation).to_homogeneous();
        let scale = Matrix4::new_nonuniform_scaling(&self.local_scale);
        translation * rotation * scale
    }

    /// 从变换矩阵中提取位置、旋转和缩放
    fn from_matrix(matrix: &Matrix4<f32>) -> Self {
        let translation = Vector3::from(matrix.fixed_view::<3, 1>(0, 3));
        let rotation =
            *UnitQuaternion::from_matrix(&Matrix3::from(matrix.fixed_view::<3, 3>(0, 0)));
        let scale = Vector3::new(
            matrix.fixed_view::<3, 1>(0, 0).norm(),
            matrix.fixed_view::<3, 1>(0, 1).norm(),
            matrix.fixed_view::<3, 1>(0, 2).norm(),
        );
        Transform {
            instance_id: 0,
            parent: Default::default(),
            children: vec![],
            game_object: Default::default(),
            position: translation,
            rotation,
            local_position: Vector3::zeros(),
            local_rotation: Quaternion::identity(),
            local_scale: scale,
        }
    }

    /// 计算相对变换
    pub fn relative_transform(parent: &Transform, child: &Transform) -> Transform {
        // 父级和子级的全局变换矩阵
        let parent_global_matrix = parent.to_global_matrix();
        let child_global_matrix = child.to_global_matrix();

        // 计算相对矩阵
        let relative_matrix = parent_global_matrix
            .try_inverse()
            .expect("Parent matrix is not invertible")
            * child_global_matrix;

        // 从相对矩阵提取新的 Transform
        Transform::from_matrix(&relative_matrix)
    }
}