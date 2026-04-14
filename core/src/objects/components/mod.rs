use apostasy_macros::Component;
use cgmath::{Quaternion, Vector3};

#[derive(Component, Clone)]
pub struct Transform {
    pub local_position: Vector3<f32>,
    pub local_euler_angles: Vector3<f32>,
    pub local_rotation: Quaternion<f32>,
    pub local_scale: Vector3<f32>,
    pub global_position: Vector3<f32>,
    pub global_rotation: Quaternion<f32>,
    pub global_euler_angles: Vector3<f32>,
    pub global_scale: Vector3<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            local_position: Vector3::new(0.0, 0.0, 0.0),
            local_euler_angles: Vector3::new(0.0, 0.0, 0.0),
            local_rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            local_scale: Vector3::new(1.0, 1.0, 1.0),
            global_position: Vector3::new(0.0, 0.0, 0.0),
            global_rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            global_euler_angles: Vector3::new(0.0, 0.0, 0.0),
            global_scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}
