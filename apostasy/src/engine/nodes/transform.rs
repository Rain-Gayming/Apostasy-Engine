use crate::engine::editor::inspectable::Inspectable;
use crate::{self as apostasy, engine::editor::inspectable::InspectValue};
use apostasy_macros::{Component, Inspectable};
use cgmath::{Deg, Euler, One, Quaternion, Rotation, Vector3};
use std::fmt::Debug;

#[derive(Component, Clone, Inspectable)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub scale: Vector3<f32>,
    pub up: Vector3<f32>,
    pub forward: Vector3<f32>,
    pub right: Vector3<f32>,
}

impl Debug for Transform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Transform")
            .field("position", &self.position)
            .field("rotation", &self.rotation)
            .field("yaw", &self.yaw)
            .field("pitch", &self.pitch)
            .field("scale", &self.scale)
            .field("up", &self.up)
            .field("forward", &self.forward)
            .field("right", &self.right)
            .finish()
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::one(),
            yaw: 0.0,
            pitch: 0.0,
            scale: Vector3::new(1.0, 1.0, 1.0),
            up: Vector3::new(0.0, 1.0, 0.0),
            forward: Vector3::new(0.0, 0.0, -1.0),
            right: Vector3::new(1.0, 0.0, 0.0),
        }
    }
}

pub fn calculate_rotation(transform: &mut Transform) {
    transform.rotation = Quaternion::from(Euler {
        x: Deg(0.0),
        y: Deg(transform.yaw),
        z: Deg(0.0),
    }) * Quaternion::from(Euler {
        x: Deg(transform.pitch),
        y: Deg(0.0),
        z: Deg(0.0),
    });
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
