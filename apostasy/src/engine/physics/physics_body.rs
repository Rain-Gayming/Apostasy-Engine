use crate::engine::editor::inspectable::Inspectable;
use crate::{self as apostasy};
use apostasy_macros::InspectValue;
use apostasy_macros::{Component, Inspectable, SerializableComponent};
use serde::{Deserialize, Serialize};

#[derive(
    Component, Clone, Inspectable, InspectValue, SerializableComponent, Serialize, Deserialize,
)]
pub struct PhysicsBody {
    pub mass: f32,
    pub friction: f32,
    pub drag: f32,
    pub bounce: f32,
    pub gravity: f32,
    pub is_gravity_enabled: bool,
    pub linear_damping: f32,
    pub angular_damping: f32,
}

impl Default for PhysicsBody {
    fn default() -> Self {
        Self {
            mass: 1.0,
            friction: crate::engine::physics::constants::source_physics::GROUND_FRICTION,
            drag: crate::engine::physics::constants::source_physics::PHYSICS_LINEAR_DAMPING,
            bounce: crate::engine::physics::constants::source_physics::BUNNY_HOP_FACTOR,
            gravity: crate::engine::physics::constants::source_physics::GRAVITY,
            is_gravity_enabled: true,
            linear_damping:
                crate::engine::physics::constants::source_physics::PHYSICS_LINEAR_DAMPING,
            angular_damping:
                crate::engine::physics::constants::source_physics::PHYSICS_ANGULAR_DAMPING,
        }
    }
}
