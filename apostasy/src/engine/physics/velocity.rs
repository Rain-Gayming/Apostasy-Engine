use crate::engine::editor::inspectable::Inspectable;
use crate::{self as apostasy, engine::nodes::components::transform::Transform};
use apostasy_macros::{Component, InspectValue, Inspectable, SerializableComponent};
use cgmath::{InnerSpace, Vector3};
use serde::{Deserialize, Serialize};

#[derive(
    Component, Clone, Inspectable, InspectValue, SerializableComponent, Serialize, Deserialize,
)]
pub struct Velocity {
    pub direction: Vector3<f32>,
    pub angular_direction: Vector3<f32>,
    pub friction: f32,
}

impl Default for Velocity {
    fn default() -> Self {
        Self {
            direction: Vector3::new(0.0, 0.0, 0.0),
            angular_direction: Vector3::new(0.0, 0.0, 0.0),
            friction: 1.0,
        }
    }
}

pub fn apply_velocity(velocity: &mut Velocity, transform: &mut Transform) {
    const PHYSICS_SCALE: f32 = 0.1;
    transform.position += velocity.direction * PHYSICS_SCALE;
}

impl Velocity {
    pub fn add_velocity(&mut self, strength: Vector3<f32>) {
        self.direction += strength;
    }
    pub fn set_velocity(&mut self, velocity: Vector3<f32>) {
        self.direction = velocity;
    }
    pub fn sync_linear_from_angular(&mut self, radius: f32, surface_normal: Vector3<f32>) {
        let r_contact = -surface_normal * radius;

        let rolling_velocity = r_contact.cross(self.angular_direction);

        let normal_component = surface_normal * self.direction.dot(surface_normal);
        let tangential_from_spin =
            rolling_velocity - surface_normal * rolling_velocity.dot(surface_normal);
        self.direction = normal_component + tangential_from_spin;
    }
}
