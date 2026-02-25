use crate::engine::editor::inspectable::Inspectable;
use crate::{self as apostasy, engine::nodes::transform::Transform};
use apostasy_macros::{Component, Inspectable, SerializableComponent};
use cgmath::Vector3;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Inspectable, SerializableComponent, Serialize, Deserialize)]
pub struct Velocity {
    pub direction: Vector3<f32>,
}

impl Default for Velocity {
    fn default() -> Self {
        Self {
            direction: Vector3::new(0.0, 0.0, 0.0),
        }
    }
}

pub fn apply_velocity(velocity: &Velocity, transform: &mut Transform) {
    transform.position += velocity.direction;
}

pub fn add_velocity(velocity: &mut Velocity, strength: Vector3<f32>) {
    velocity.direction += strength;
}
