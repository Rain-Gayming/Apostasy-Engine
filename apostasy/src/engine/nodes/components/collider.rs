use crate::engine::{
    editor::inspectable::Inspectable,
    nodes::{
        World,
        components::{transform::Transform, velocity::Velocity},
    },
};
use apostasy_macros::{Component, InspectValue, Inspectable, SerializableComponent, update};
use cgmath::{InnerSpace, Vector3, Zero};
use serde::{Deserialize, Serialize};

use crate as apostasy;

#[derive(
    Component, Clone, Inspectable, InspectValue, SerializableComponent, Serialize, Deserialize,
)]
pub struct Collider {
    pub half_extents: Vector3<f32>,
    pub offset: Vector3<f32>,
    pub is_static: bool,
    pub is_area: bool,
}

impl Default for Collider {
    fn default() -> Self {
        Self {
            half_extents: Vector3::new(0.5, 0.5, 0.5),
            offset: Vector3::new(0.0, 0.0, 0.0),
            is_static: false,
            is_area: false,
        }
    }
}

impl Collider {
    /// Creates a dynamic collider
    pub fn new(half_extents: Vector3<f32>, offset: Vector3<f32>) -> Self {
        Self {
            half_extents,
            offset,
            is_static: false,
            is_area: false,
        }
    }

    /// Creates a static collider
    pub fn new_static(half_extents: Vector3<f32>, offset: Vector3<f32>) -> Self {
        Self {
            half_extents,
            offset,
            is_static: true,
            is_area: false,
        }
    }

    /// Returns the minimum point of the collider
    pub fn world_min(&self, position: Vector3<f32>) -> Vector3<f32> {
        position - self.half_extents
    }

    /// Returns the maximum point of the collider
    pub fn world_max(&self, position: Vector3<f32>) -> Vector3<f32> {
        position + self.half_extents
    }

    /// Returns the translation vector between two colliders
    pub fn translation_vector_against(
        &self,
        pos_a: Vector3<f32>,
        other: &Collider,
        pos_b: Vector3<f32>,
    ) -> Option<Vector3<f32>> {
        let d = pos_a - pos_b;

        let ox = self.half_extents.x + other.half_extents.x - d.x.abs();
        let oy = self.half_extents.y + other.half_extents.y - d.y.abs();
        let oz = self.half_extents.z + other.half_extents.z - d.z.abs();

        // Separated on at least one axis → no collision
        if ox <= 0.0 || oy <= 0.0 || oz <= 0.0 {
            return None;
        }

        // Resolve on the axis with the smallest penetration depth
        if ox <= oy && ox <= oz {
            Some(Vector3::new(ox * d.x.signum(), 0.0, 0.0))
        } else if oy <= ox && oy <= oz {
            Some(Vector3::new(0.0, oy * d.y.signum(), 0.0))
        } else {
            Some(Vector3::new(0.0, 0.0, oz * d.z.signum()))
        }
    }

    /// Returns true when `point` lies inside (or on the surface of) the box.
    pub fn contains_point(&self, position: Vector3<f32>, point: Vector3<f32>) -> bool {
        let min = self.world_min(position);
        let max = self.world_max(position);
        point.x >= min.x
            && point.x <= max.x
            && point.y >= min.y
            && point.y <= max.y
            && point.z >= min.z
            && point.z <= max.z
    }
}

/// Contains information about a collision event
#[derive(Debug, Clone)]
pub struct CollisionEvent {
    pub node_a: String,
    pub node_b: String,
    pub translation_vector: Vector3<f32>,
    pub depth: f32,
    pub normal: Vector3<f32>,
}

#[derive(Debug, Clone, Default, Component)]
pub struct CollisionEvents {
    pub events: Vec<CollisionEvent>,
}

impl CollisionEvents {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Data cloned from each eligible node for the read-only detection pass.
#[derive(Clone)]
struct Snapshot {
    name: String,
    position: Vector3<f32>,
    collider: Collider,
}

/// Builds snapshots of all collidable nodes in the world
fn build_snapshot(world: &World) -> Vec<Snapshot> {
    world
        .get_all_nodes()
        .into_iter()
        .filter_map(|node| {
            let position = node.get_component::<Transform>()?.position;
            let collider = node.get_component::<Collider>()?.clone();
            Some(Snapshot {
                name: node.name.clone(),
                position,
                collider,
            })
        })
        .collect()
}

/// Detects collisions between all nodes
#[update]
pub fn collision_detection_system(world: &mut World) {
    let snapshot = build_snapshot(world);
    let n = snapshot.len();

    let mut events: Vec<CollisionEvent> = Vec::new();

    for i in 0..n {
        for j in (i + 1)..n {
            let a = &snapshot[i];
            let b = &snapshot[j];

            if let Some(translation_vector) =
                a.collider
                    .translation_vector_against(a.position, &b.collider, b.position)
            {
                let depth = translation_vector.magnitude2();
                let normal = translation_vector.normalize();
                events.push(CollisionEvent {
                    node_a: a.name.clone(),
                    node_b: b.name.clone(),
                    translation_vector,
                    depth,
                    normal,
                });
            }
        }
    }

    for event in &events {
        // get nodes, colldiers and information
        let a = world.get_node_with_name(&event.node_a);
        let b = world.get_node_with_name(&event.node_b);

        if let Some(a) = a
            && let Some(b) = b
            && let Some(a_collider) = a.get_component::<Collider>()
            && let Some(b_collider) = b.get_component::<Collider>()
        {
            let a_static = a_collider.is_static;
            let b_static = b_collider.is_static;

            let normal_a = event.normal;
            let normal_b = Vector3::new(-event.normal.x, -event.normal.y, -event.normal.z);

            match (a_static, b_static) {
                // both are dynamic, split the correction evenly
                (false, false) => {
                    let half = event.translation_vector * 0.5;
                    let neg_half = Vector3::new(-half.x, -half.y, -half.z);
                    resolve_node(world, &event.node_a, half, normal_a);
                    resolve_node(world, &event.node_b, neg_half, normal_b);
                }
                // a is static,push b the full amount
                (true, false) => {
                    let neg_translation_vector = Vector3::new(
                        -event.translation_vector.x,
                        -event.translation_vector.y,
                        -event.translation_vector.z,
                    );
                    resolve_node(world, &event.node_b, neg_translation_vector, normal_b);
                }
                // b is static, push a the full amount
                (false, true) => {
                    resolve_node(world, &event.node_a, event.translation_vector, normal_a);
                }
                // both static, do nothing
                (true, true) => {}
            }
        }
    }

    for global in world.global_nodes.iter_mut() {
        if let Some(ev) = global.get_component_mut::<CollisionEvents>() {
            ev.events = events;
            return;
        }
    }
}

/// Resolves collision events by pushing the node in the opposite direction
fn resolve_node(world: &mut World, name: &str, _offset: Vector3<f32>, normal: Vector3<f32>) {
    let node = world.get_node_with_name_mut(name);

    if let Some(node) = node
        && let Some(velocity) = node.get_component_mut::<Velocity>()
        && velocity.direction != Vector3::zero()
    {
        velocity.add_velocity(normal);
    }
}
