use crate::engine::editor::inspectable::Inspectable;
use crate::{self as apostasy};
use apostasy_macros::{Component, Inspectable, SerializableComponent};
use cgmath::{Deg, Euler, One, Quaternion, Rotation, Vector3};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Component, Clone, Inspectable, Serialize, Deserialize, SerializableComponent)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub scale: Vector3<f32>,
    up: Vector3<f32>,
    forward: Vector3<f32>,
    right: Vector3<f32>,
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

impl Transform {
    pub fn calculate_rotation(&mut self) {
        self.rotation = Quaternion::from(Euler {
            x: Deg(0.0),
            y: Deg(self.yaw),
            z: Deg(0.0),
        }) * Quaternion::from(Euler {
            x: Deg(self.pitch),
            y: Deg(0.0),
            z: Deg(0.0),
        });
    }

    pub fn calculate_up(&self) -> Vector3<f32> {
        self.rotation.rotate_vector(self.up)
    }

    pub fn calculate_forward(&self) -> Vector3<f32> {
        self.rotation.rotate_vector(self.forward)
    }

    pub fn calculate_right(&self) -> Vector3<f32> {
        self.rotation.rotate_vector(self.right)
    }
}
