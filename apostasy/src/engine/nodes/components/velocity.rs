use crate::engine::editor::inspectable::Inspectable;
use crate::{self as apostasy, engine::nodes::components::transform::Transform};
use apostasy_macros::{Component, InspectValue, Inspectable, SerializableComponent};
use cgmath::{InnerSpace, Vector3, Zero};
use serde::{Deserialize, Serialize};

#[derive(
    Component, Clone, Inspectable, InspectValue, SerializableComponent, Serialize, Deserialize,
)]
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

impl Velocity {
    pub fn add_velocity(&mut self, strength: Vector3<f32>) {
        let len = strength.magnitude();
        if len > 0.0 {
            self.direction = strength / len;
        } else {
            self.direction = Vector3::zero();
        }
    }
}
