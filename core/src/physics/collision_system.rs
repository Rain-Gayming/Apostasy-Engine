use anyhow::Result;
use apostasy_macros::update;
use cgmath::Vector3;

use crate::{
    objects::{components::transform::Transform, systems::DeltaTime, world::World},
    physics::{collider::Collider, velocity::Velocity},
    voxels::{VoxelTransform, chunk::Chunk, voxel_raycast::sample_world},
};
fn get_overlapping_voxels(
    aabb_min: Vector3<f32>,
    aabb_max: Vector3<f32>,
    chunks: &[(Vector3<i32>, &Chunk)],
) -> Vec<Vector3<i32>> {
    let epsilon = 0.001;
    let min = Vector3::new(
        (aabb_min.x + epsilon).floor() as i32,
        (aabb_min.y + epsilon).floor() as i32,
        (aabb_min.z + epsilon).floor() as i32,
    );
    let max = Vector3::new(
        (aabb_max.x - epsilon).floor() as i32,
        (aabb_max.y - epsilon).floor() as i32,
        (aabb_max.z - epsilon).floor() as i32,
    );

    let mut solids = Vec::new();
    for x in min.x..=max.x {
        for y in min.y..=max.y {
            for z in min.z..=max.z {
                let voxel = Vector3::new(x, y, z);
                if sample_world(voxel, chunks).is_some() {
                    solids.push(voxel);
                }
            }
        }
    }
    solids
}

pub fn resolve_collisions(
    position: &mut Vector3<f32>,
    delta: &mut Vector3<f32>,
    half_extents: Vector3<f32>,
    chunks: &[(Vector3<i32>, &Chunk)],
) -> CollisionFlags {
    let mut flags = CollisionFlags::default();

    let axes: [usize; 3] = [1, 0, 2];

    for axis in axes {
        let axis_vel = match axis {
            0 => delta.x,
            1 => delta.y,
            _ => delta.z,
        };

        if axis_vel.abs() < 1e-6 {
            continue;
        }

        let axis_delta = match axis {
            0 => Vector3::new(delta.x, 0.0, 0.0),
            1 => Vector3::new(0.0, delta.y, 0.0),
            _ => Vector3::new(0.0, 0.0, delta.z),
        };

        let candidate = *position + axis_delta;
        let aabb_min = candidate - half_extents;
        let aabb_max = candidate + half_extents;

        let solids = get_overlapping_voxels(aabb_min, aabb_max, chunks);

        if solids.is_empty() {
            *position += axis_delta;
            continue;
        }

        let mut best_overlap = 0.0f32;

        for voxel in &solids {
            let vmin = Vector3::new(voxel.x as f32, voxel.y as f32, voxel.z as f32);
            let vmax = vmin + Vector3::new(1.0, 1.0, 1.0);

            let (a_min, a_max, v_min, v_max) = match axis {
                0 => (aabb_min.x, aabb_max.x, vmin.x, vmax.x),
                1 => (aabb_min.y, aabb_max.y, vmin.y, vmax.y),
                _ => (aabb_min.z, aabb_max.z, vmin.z, vmax.z),
            };

            let overlap = if axis_vel > 0.0 {
                v_min - a_max
            } else {
                v_max - a_min
            };

            let is_penetrating =
                (axis_vel > 0.0 && overlap < 0.0) || (axis_vel < 0.0 && overlap > 0.0);

            if !is_penetrating {
                continue;
            }

            // keep the largest penetration
            if overlap.abs() > best_overlap.abs() {
                best_overlap = overlap;
            }
        }

        if best_overlap.abs() < 1e-6 {
            *position += axis_delta;
            continue;
        }

        let correction = if axis_vel > 0.0 {
            best_overlap.max(-axis_vel.abs())
        } else {
            best_overlap.min(axis_vel.abs())
        };

        match axis {
            0 => {
                position.x += axis_delta.x + correction;
                delta.x = 0.0;
                flags.hit_wall = true;
            }
            1 => {
                position.y += axis_delta.y + correction;
                delta.y = 0.0;
                if axis_vel < 0.0 {
                    flags.grounded = true;
                } else {
                    flags.hit_ceil = true;
                }
            }
            _ => {
                position.z += axis_delta.z + correction;
                delta.z = 0.0;
                flags.hit_wall = true;
            }
        }
    }

    flags
}

#[update(priority = 5)]
pub fn resolve_collisions_system(world: &mut World) -> Result<()> {
    let delta = world.get_resource::<DeltaTime>()?.0;
    let chunks: Vec<(Vector3<i32>, Chunk)> = world
        .get_objects_with_component::<Chunk>()
        .iter()
        .filter_map(|o| {
            let pos = o.get_component::<VoxelTransform>().ok()?.position;
            let chunk = o.get_component::<Chunk>().ok()?.clone();
            Some((pos, chunk))
        })
        .collect();
    let chunk_refs: Vec<(Vector3<i32>, &Chunk)> =
        chunks.iter().map(|(pos, chunk)| (*pos, chunk)).collect();

    let objects = world.get_objects_with_component_mut::<Collider>();
    for object in objects {
        let Ok(collider) = object.get_component::<Collider>() else {
            continue;
        };
        let collider = collider.clone();
        let Ok(transform) = object.get_component::<Transform>() else {
            continue;
        };
        let Ok(velocity) = object.get_component::<Velocity>() else {
            continue;
        };

        let mut position = transform.global_position;
        let mut frame_delta = Vector3::new(
            velocity.linear_velocity.x * delta,
            velocity.linear_velocity.y * delta,
            velocity.linear_velocity.z * delta,
        );

        let flags = resolve_collisions(
            &mut position,
            &mut frame_delta,
            collider.half_extents,
            &chunk_refs,
        );

        object.get_component_mut::<Transform>()?.global_position = position;
        object.get_component_mut::<Transform>()?.local_position = position;

        let velocity = object.get_component_mut::<Velocity>()?;
        if flags.hit_ceil {
            velocity.linear_velocity.y = 0.0;
        }

        velocity.is_grounded = flags.grounded;
    }

    Ok(())
}

#[derive(Default)]
pub struct CollisionFlags {
    pub grounded: bool,
    pub hit_ceil: bool,
    pub hit_wall: bool,
}
