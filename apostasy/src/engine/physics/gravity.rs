use crate::engine::nodes::world::World;
use crate::engine::physics::velocity::Velocity;
use crate::engine::physics::constants::source_physics;
use crate::{self as apostasy};
use apostasy_macros::fixed_update;

#[fixed_update]
pub fn apply_gravity(world: &mut World, delta_time: f32) {
    // First collect velocity data
    let mut velocity_data = Vec::new();
    {
        let nodes = world.get_all_nodes();
        for node in nodes {
            if let Some(velocity) = node.get_component::<Velocity>() {
                velocity_data.push((node.id, velocity.direction, velocity.is_grounded));
            }
        }
    }

    // Then apply gravity and update positions
    for (node_id, mut velocity, is_grounded) in velocity_data {
        // Only apply gravity if not on the ground
        if !is_grounded {
            velocity.y += source_physics::GRAVITY * delta_time;

            // Clamp to terminal velocity
            if velocity.y < -source_physics::TERMINAL_VELOCITY {
                velocity.y = -source_physics::TERMINAL_VELOCITY;
            }
        } else {
            // Clamp vertical velocity to zero when grounded to prevent jittering
            if velocity.y < 0.0 {
                velocity.y = 0.0;
            }
        }

        // Update velocity component only
        if let Some(node) = world.get_all_nodes_mut().iter_mut().find(|n| n.id == node_id) {
            if let Some(velocity_comp) = node.get_component_mut::<Velocity>() {
                velocity_comp.direction = velocity;
            }
        }
    }
}
