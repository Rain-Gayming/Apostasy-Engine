use crate::engine::nodes::Node;
use crate::engine::nodes::components::transform::Transform;
use crate::engine::nodes::world::World;
use crate::engine::physics::raycast::Raycast;
use crate::engine::physics::velocity::Velocity;
use crate::{self as apostasy};
use apostasy_macros::fixed_update;
use cgmath::Vector3;

#[fixed_update]
pub fn update_ground_state(world: &mut World, delta_time: f32) {
    // First pass: collect all nodes that need ground checking
    let mut nodes_to_check = Vec::new();
    {
        let all_nodes = world.get_all_nodes();
        for node in all_nodes {
            if node.get_component::<Velocity>().is_some()
                && node.get_component::<Transform>().is_some()
            {
                nodes_to_check.push((
                    node.name.clone(),
                    node.get_component::<Transform>().unwrap().global_position,
                ));
            }
        }
    }

    // Second pass: check ground for each node
    for (node_name, position) in nodes_to_check {
        // Skip editor camera
        if node_name == "EditorCamera" {
            continue;
        }

        // Collect nodes for raycast first (avoid borrowing world while we have a mutable borrow)
        let all_nodes: Vec<Node> = world.get_all_nodes().iter().map(|n| (*n).clone()).collect();
        let node_refs: Vec<&Node> = all_nodes.iter().collect();

        let raycast = Raycast::new(Vector3::new(0.0, -1.0, 0.0), 1.25);
        let hits = raycast.cast_from(
            position,
            Vector3::new(0.0, -1.0, 0.0),
            &node_refs,
            &node_name, // This filters out self-detection
        );

        if let Some(node) = world.get_node_with_name_mut(&node_name) {
            if let Some(velocity) = node.get_component_mut::<Velocity>() {
                let mut is_grounded = false;
                if hits.is_some() {
                    // Only consider grounded if not moving upward
                    if velocity.direction.y <= 0.0 {
                        is_grounded = true;
                    }
                }

                velocity.update_ground_state(is_grounded, delta_time);

                // Apply friction if grounded
                if is_grounded {
                    velocity.apply_ground_friction(
                        crate::engine::physics::constants::source_physics::GROUND_FRICTION,
                        crate::engine::physics::constants::source_physics::STOP_SPEED,
                        delta_time,
                    );
                } else {
                    velocity.apply_air_friction(delta_time);
                }
            }
        }
    }
}
