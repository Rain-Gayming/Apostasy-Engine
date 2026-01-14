use crate as apostasy;
use apostasy_macros::Component;
use cgmath::{Quaternion, Vector3};

#[derive(Component)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
}
