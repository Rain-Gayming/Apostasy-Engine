use apostasy_macros::Component;
use cgmath::{Quaternion, Vector3};

#[derive(Component, Clone)]
pub struct Transform {
    pub local_position: Vector3<f32>,
    pub local_euler_angles: Vector3<f32>,
    pub local_rotation: Quaternion<f32>,
    pub local_scale: Vector3<f32>,
    global_position: Vector3<f32>,
    global_rotation: Quaternion<f32>,
    global_euler_angles: Vector3<f32>,
    global_scale: Vector3<f32>,
}
