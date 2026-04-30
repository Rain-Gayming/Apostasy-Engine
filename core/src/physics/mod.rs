use anyhow::Result;
use apostasy_macros::{Component, fixed_update};

use crate::{objects::world::World, physics::velocity::Velocity};

pub mod collider;
pub mod collision_system;
pub mod velocity;

#[derive(Component, Clone, Debug)]
pub struct Gravity {
    pub strength: f32,
}

impl Default for Gravity {
    fn default() -> Self {
        Self { strength: 9.81 }
    }
}

impl Gravity {
    pub fn deserialize(&mut self, _value: &serde_yaml::Value) -> anyhow::Result<()> {
        Ok(())
    }
}

#[fixed_update(priority = 10)]
pub fn apply_gravity(world: &mut World, delta: f32) -> Result<()> {
    let objects = world.get_objects_with_component_mut::<Velocity>();

    for object in objects {
        let velocity = object.get_component_mut::<Velocity>()?;
        if !velocity.is_grounded {
            velocity.linear_velocity.y -= 9.8 * delta;

            if velocity.linear_velocity.y < -50.0 {
                velocity.linear_velocity.y = -50.0;
            }
        }
    }

    Ok(())
}
