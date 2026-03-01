use crate::engine::editor::inspectable::Inspectable;
use crate::engine::nodes::World;
use crate::engine::nodes::components::transform::Transform;
use crate::engine::nodes::components::velocity::apply_velocity;
use crate::{self as apostasy, engine::nodes::components::velocity::Velocity};
use apostasy_macros::InspectValue;
use apostasy_macros::{Component, Inspectable, SerializableComponent, fixed_update};
use cgmath::Vector3;
use serde::{Deserialize, Serialize};

#[derive(
    Component, Clone, Inspectable, InspectValue, SerializableComponent, Serialize, Deserialize,
)]
pub struct Physics {
    pub mass: f32,
    pub friction: f32,
    pub gravity: f32,
    pub is_gravity_enabled: bool,
}

impl Default for Physics {
    fn default() -> Self {
        Self {
            mass: 1.0,
            friction: 0.0,
            gravity: -9.81,
            is_gravity_enabled: false,
        }
    }
}

#[fixed_update]
pub fn apply_gravity(world: &mut World, delta_time: f32) {
    for node in world.get_all_nodes_mut() {
        if let Some(physics) = node.get_component_mut::<Physics>() {
            let gravity = physics.gravity;
            if physics.is_gravity_enabled {
                let (transform, velocity) =
                    node.get_components_mut::<(&mut Transform, &mut Velocity)>();
                velocity.add_velocity(Vector3::new(0.0, gravity * delta_time, 0.0));
                apply_velocity(velocity, transform);
            }
        }
    }
}
