#![allow(dead_code)]
use nalgebra::{Quaternion, Vector3, Vector4};
use std::f32::consts::FRAC_1_SQRT_2;

pub struct Compress;

impl Compress {
    pub const QUATERNION_MIN_RANGE: f32 = -FRAC_1_SQRT_2;
    pub const QUATERNION_MAX_RANGE: f32 = FRAC_1_SQRT_2;
    pub const TEN_BITS_MAX: u32 = 1023;
    fn largest_absolute_component_index(q: &Quaternion<f32>) -> (usize, f32, Vector3<f32>) {
        let abs = Vector4::new(q.i.abs(), q.j.abs(), q.k.abs(), q.w.abs());

        let mut largest_abs = abs.x;
        let mut without_largest = Vector3::new(q.j, q.k, q.w);
        let mut largest_index = 0;

        if abs.y > largest_abs {
            largest_index = 1;
            largest_abs = abs.y;
            without_largest = Vector3::new(q.i, q.k, q.w);
        }
        if abs.z > largest_abs {
            largest_index = 2;
            largest_abs = abs.z;
            without_largest = Vector3::new(q.i, q.j, q.w);
        }
        if abs.w > largest_abs {
            largest_index = 3;
            largest_abs = abs.w;
            without_largest = Vector3::new(q.i, q.j, q.k);
        }
        (largest_index, largest_abs, without_largest)
    }

    // 将 `u16` 值缩放到指定的浮点范围
    fn scale_ushort_to_float(
        &self,
        value: u16,
        min_value: u32,
        max_value: u32,
        min_target: f32,
        max_target: f32,
    ) -> f32 {
        let target_range: f32 = max_target - min_target;
        let value_range = (max_value - min_value) as f32;
        let value_relative = (value as u32 - min_value) as f32;
        min_target + value_relative / value_range * target_range
    }

    // 将浮点值缩放到 `u16` 范围
    fn scale_float_to_ushort(
        &self,
        value: f32,
        min_value: f32,
        max_value: f32,
        min_target: u16,
        max_target: u16,
    ) -> u16 {
        let target_range = (max_target - min_target) as f32;
        let value_range = max_value - min_value;
        let value_relative = value - min_value;
        min_target + (value_relative / value_range * target_range) as u16
    }

    // 安全地规范化四元数，即使输入包含无效值（如 NaN）
    fn quaternion_normalize_safe(&self, v4: Vector4<f32>) -> Quaternion<f32> {
        const FLT_MIN_NORMAL: f64 = 1.175494351e-38f64;
        let len = v4.dot(&v4);
        if len > FLT_MIN_NORMAL as f32 {
            v4.normalize().into()
        } else {
            Quaternion::identity()
        }
    }

    pub fn vector3float_to_vector3long(
        &self,
        value: Vector3<f32>,
        precision: f32,
    ) -> (bool, Vector3<i64>) {
        let (result, x, y, z) = self.vector3float_to_long3(value, precision);
        (result, Vector3::new(x, y, z))
    }

    fn vector3float_to_long3(&self, value: Vector3<f32>, precision: f32) -> (bool, i64, i64, i64) {
        let mut result = true;
        let (res, x) = self.float_to_long(value.x, precision);
        result &= res;
        let (res, y) = self.float_to_long(value.y, precision);
        result &= res;
        let (res, z) = self.float_to_long(value.z, precision);
        result &= res;
        (result, x, y, z)
    }

    fn float_to_long(&self, value: f32, precision: f32) -> (bool, i64) {
        if precision == 0.0 {
            println!("precision cannot be 0");
        }
        let quantized = (value / precision) as i64;
        (true, quantized)
    }

    fn long_to_float(&self, value: i64, precision: f32) -> f32 {
        if precision == 0.0 {
            println!("precision cannot be 0");
        }
        value as f32 * precision
    }

    pub fn vector3long_to_vector3float(&self, value: Vector3<i64>, precision: f32) -> Vector3<f32> {
        self.long3_to_vector3float(value.x, value.y, value.z, precision)
    }

    fn long3_to_vector3float(&self, x: i64, y: i64, z: i64, precision: f32) -> Vector3<f32> {
        let mut v = Vector3::new(0.0, 0.0, 0.0);
        v.x = self.long_to_float(x, precision);
        v.y = self.long_to_float(y, precision);
        v.z = self.long_to_float(z, precision);
        v
    }

    pub fn var_uint_size(&self, value: u64) -> usize {
        if value <= 240 {
            return 1;
        }
        if value <= 2287 {
            return 2;
        }
        if value <= 67823 {
            return 3;
        }
        if value <= 16777215 {
            return 4;
        }
        if value <= 4294967295 {
            return 5;
        }
        if value <= 1099511627775 {
            return 6;
        }
        if value <= 281474976710655 {
            return 7;
        }
        if value <= 72057594037927935 {
            return 8;
        }
        9
    }
}
