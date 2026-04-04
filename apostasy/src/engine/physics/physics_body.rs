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
}

impl Default for PhysicsBody {
    fn default() -> Self {
        Self {
            mass: 1.0,
            friction: 6.0,
            drag: 0.05,
            bounce: 0.0,
            gravity: -9.81,
            is_gravity_enabled: false,
        }
    }
}
