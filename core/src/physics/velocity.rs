use anyhow::Result;
use apostasy_macros::{Component, update};
use cgmath::{Vector3, Zero};

use crate::objects::{components::transform::Transform, world::World};

#[derive(Component, Clone)]
pub struct Velocity {
    pub angular_velocity: Vector3<f32>,
    pub linear_velocity: Vector3<f32>,
    pub mass: f32,
}

impl Default for Velocity {
    fn default() -> Self {
        Self {
            angular_velocity: Vector3::zero(),
            linear_velocity: Vector3::zero(),
            mass: 1.0,
        }
    }
}

#[update]
fn velocity_process(world: &mut World) -> Result<()> {
    for node in world.get_objects_with_component_mut::<Velocity>() {
        let linear = { node.get_component::<Velocity>()?.linear_velocity };
        let transform = node.get_component_mut::<Transform>()?;

        transform.local_position += linear;
    }

    Ok(())
}
