use anyhow::Result;
use apostasy_macros::update;
use cgmath::Vector3;
use slotmap::DefaultKey;

use crate::{
    objects::{
        components::transform::Transform, scene::ObjectId, systems::DeltaTime, world::World,
    },
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

pub fn resolve_chunk_collisions(
    position: &mut Vector3<f32>,
    delta: &mut Vector3<f32>,
    half_extents: Vector3<f32>,
    chunks: &[(Vector3<i32>, &Chunk)],
) -> CollisionFlags {
    let mut flags = CollisionFlags::default();

    let ground_probe = 0.05f32;
    let probe_min = (*position - half_extents) - Vector3::new(0.0, ground_probe, 0.0);
    let probe_max = *position + half_extents;
    if !get_overlapping_voxels(probe_min, probe_max, chunks).is_empty() {
        flags.grounded = true;
    }

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

#[derive(Default)]
pub struct CollisionFlags {
    pub grounded: bool,
    pub hit_ceil: bool,
    pub hit_wall: bool,
}

pub fn resolve_object_collisions(
    position: &mut Vector3<f32>,
    delta: &mut Vector3<f32>,
    half_extents: Vector3<f32>,
    self_id: ObjectId,                                          // to skip self
    other_colliders: &[(ObjectId, Vector3<f32>, Vector3<f32>)], // (id, world_pos, half_extents)
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
        let a_min = candidate - half_extents;
        let a_max = candidate + half_extents;

        let mut best_overlap = 0.0f32;

        for (id, other_pos, other_half) in other_colliders {
            if *id == self_id {
                continue;
            }

            let b_min = other_pos - other_half;
            let b_max = other_pos + other_half;

            // check overlap on all 3 axes to confirm actual AABB intersection
            let overlap_x = a_min.x < b_max.x && a_max.x > b_min.x;
            let overlap_y = a_min.y < b_max.y && a_max.y > b_min.y;
            let overlap_z = a_min.z < b_max.z && a_max.z > b_min.z;

            if !overlap_x || !overlap_y || !overlap_z {
                continue;
            }

            let (ca_min, ca_max, cb_min, cb_max) = match axis {
                0 => (a_min.x, a_max.x, b_min.x, b_max.x),
                1 => (a_min.y, a_max.y, b_min.y, b_max.y),
                _ => (a_min.z, a_max.z, b_min.z, b_max.z),
            };

            let overlap = if axis_vel > 0.0 {
                cb_min - ca_max
            } else {
                cb_max - ca_min
            };

            let is_penetrating =
                (axis_vel > 0.0 && overlap < 0.0) || (axis_vel < 0.0 && overlap > 0.0);

            if !is_penetrating {
                continue;
            }

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
#[update]
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

    // snapshot all collider world positions and scaled half extents before mutation
    let collider_snapshot: Vec<(ObjectId, Vector3<f32>, Vector3<f32>)> = world
        .get_objects_with_component_with_ids::<Collider>()
        .iter()
        .filter_map(|(id, obj)| {
            let transform = obj.get_component::<Transform>().ok()?;
            let collider = obj.get_component::<Collider>().ok()?;

            // scale half extents by global scale
            let scaled = Vector3::new(
                collider.half_extents.x * transform.global_scale.x,
                collider.half_extents.y * transform.global_scale.y,
                collider.half_extents.z * transform.global_scale.z,
            );

            Some((id.clone(), transform.global_position, scaled))
        })
        .collect();

    let mut objects = world.get_objects_with_component_mut::<Collider>();
    for (i, object) in objects.iter_mut().enumerate() {
        let Ok(collider) = object.get_component::<Collider>() else {
            continue;
        };
        let Ok(transform) = object.get_component::<Transform>() else {
            continue;
        };
        let Ok(velocity) = object.get_component::<Velocity>() else {
            continue;
        };

        let scaled_half = Vector3::new(
            collider.half_extents.x * transform.global_scale.x,
            collider.half_extents.y * transform.global_scale.y,
            collider.half_extents.z * transform.global_scale.z,
        );

        let self_id = collider_snapshot[i].0;
        let mut position = transform.global_position;
        let mut frame_delta = velocity.linear_velocity * delta;

        // chunk collision first
        let mut flags =
            resolve_chunk_collisions(&mut position, &mut frame_delta, scaled_half, &chunk_refs);

        // then object vs object
        let obj_flags = resolve_object_collisions(
            &mut position,
            &mut frame_delta,
            scaled_half,
            self_id,
            &collider_snapshot,
        );

        // merge flags
        flags.grounded |= obj_flags.grounded;
        flags.hit_ceil |= obj_flags.hit_ceil;
        flags.hit_wall |= obj_flags.hit_wall;

        object.get_component_mut::<Transform>()?.global_position = position;
        object.get_component_mut::<Transform>()?.local_position = position;

        let vel = object.get_component_mut::<Velocity>()?;
        if flags.hit_ceil {
            vel.linear_velocity.y = 0.0;
        }
        vel.is_grounded = flags.grounded;
    }

    Ok(())
}
