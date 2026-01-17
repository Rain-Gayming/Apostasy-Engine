use crate as apostasy;
use apostasy_macros::Component;
use cgmath::{One, Quaternion, Rotation, Vector3};

#[derive(Component)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
    pub up: Vector3<f32>,
    pub forward: Vector3<f32>,
    pub right: Vector3<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::one(),
            scale: Vector3::new(1.0, 1.0, 1.0),
            up: Vector3::new(0.0, 1.0, 0.0),
            forward: Vector3::new(0.0, 0.0, -1.0),
            right: Vector3::new(1.0, 0.0, 0.0),
        }
    }
}

pub fn calculate_up(transform: &Transform) -> Vector3<f32> {
    transform.rotation.rotate_vector(transform.up)
}

pub fn calculate_forward(transform: &Transform) -> Vector3<f32> {
    transform.rotation.rotate_vector(transform.forward)
}

pub fn calculate_right(transform: &Transform) -> Vector3<f32> {
    transform.rotation.rotate_vector(transform.right)
}
