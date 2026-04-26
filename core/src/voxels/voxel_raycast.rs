use crate::objects::world::World;
use crate::rendering::components::camera::Camera;
use crate::utils::flatten::flatten;
use crate::voxels::VoxelTransform;
use crate::voxels::chunk::Chunk;
use crate::{objects::components::transform::Transform, voxels::voxel::VoxelId};
use anyhow::{Error, Result};
use apostasy_macros::Resource;
use cgmath::Vector3;

#[derive(Resource, Debug, Clone)]
pub struct RaycastHit {
    pub voxel_pos: Vector3<i32>,
    pub chunk_pos: Vector3<i32>,
    pub local_pos: Vector3<i32>,
    pub face: u8,
    pub distance: f32,
    pub set_to: Option<VoxelId>,
}

pub struct Ray {
    pub origin: Vector3<f32>,
    pub direction: Vector3<f32>,
}

impl Ray {
    pub fn new(origin: Vector3<f32>, direction: Vector3<f32>) -> Self {
        Self {
            origin,
            // normalize direction
            direction: {
                let len = (direction.x * direction.x
                    + direction.y * direction.y
                    + direction.z * direction.z)
                    .sqrt();
                Vector3::new(direction.x / len, direction.y / len, direction.z / len)
            },
        }
    }
}

pub fn raycast(
    ray: &Ray,
    max_distance: f32,
    chunks: &[(Vector3<i32>, &Chunk)], // (chunk_pos, chunk_data)
    set_to: Option<VoxelId>,
) -> Option<RaycastHit> {
    // current voxel position
    let mut voxel = Vector3::new(
        ray.origin.x.floor() as i32,
        ray.origin.y.floor() as i32,
        ray.origin.z.floor() as i32,
    );

    // which direction we step in each axis
    let step = Vector3::new(
        if ray.direction.x >= 0.0 { 1i32 } else { -1 },
        if ray.direction.y >= 0.0 { 1i32 } else { -1 },
        if ray.direction.z >= 0.0 { 1i32 } else { -1 },
    );

    // how far along the ray we must travel to cross a voxel boundary
    let t_delta = Vector3::new(
        if ray.direction.x.abs() < 1e-8 {
            f32::MAX
        } else {
            1.0 / ray.direction.x.abs()
        },
        if ray.direction.y.abs() < 1e-8 {
            f32::MAX
        } else {
            1.0 / ray.direction.y.abs()
        },
        if ray.direction.z.abs() < 1e-8 {
            f32::MAX
        } else {
            1.0 / ray.direction.z.abs()
        },
    );

    // distance to the next boundary in each axis
    let mut t_max = Vector3::new(
        if ray.direction.x >= 0.0 {
            (voxel.x as f32 + 1.0 - ray.origin.x) / ray.direction.x.abs().max(1e-8)
        } else {
            (ray.origin.x - voxel.x as f32) / ray.direction.x.abs().max(1e-8)
        },
        if ray.direction.y >= 0.0 {
            (voxel.y as f32 + 1.0 - ray.origin.y) / ray.direction.y.abs().max(1e-8)
        } else {
            (ray.origin.y - voxel.y as f32) / ray.direction.y.abs().max(1e-8)
        },
        if ray.direction.z >= 0.0 {
            (voxel.z as f32 + 1.0 - ray.origin.z) / ray.direction.z.abs().max(1e-8)
        } else {
            (ray.origin.z - voxel.z as f32) / ray.direction.z.abs().max(1e-8)
        },
    );

    let mut last_face: u8 = 0;
    let mut distance = 0.0f32;

    while distance < max_distance {
        // check if current voxel is solid
        if let Some(_hit_face) = sample_world(voxel, chunks) {
            let chunk_pos = Vector3::new(
                voxel.x.div_euclid(32),
                voxel.y.div_euclid(32),
                voxel.z.div_euclid(32),
            );
            let local_pos = Vector3::new(
                voxel.x.rem_euclid(32),
                voxel.y.rem_euclid(32),
                voxel.z.rem_euclid(32),
            );

            return Some(RaycastHit {
                voxel_pos: voxel,
                chunk_pos,
                local_pos,
                face: last_face,
                distance,
                set_to,
            });
        }

        // step to the next voxel boundary
        if t_max.x < t_max.y && t_max.x < t_max.z {
            voxel.x += step.x;
            distance = t_max.x;
            t_max.x += t_delta.x;

            last_face = if step.x > 0 { 1 } else { 0 }; // -X or +X
        } else if t_max.y < t_max.z {
            voxel.y += step.y;
            distance = t_max.y;
            t_max.y += t_delta.y;
            last_face = if step.y > 0 { 3 } else { 2 }; // -Y or +Y
        } else {
            voxel.z += step.z;
            distance = t_max.z;
            t_max.z += t_delta.z;
            last_face = if step.z > 0 { 5 } else { 4 }; // -Z or +Z
        }
    }

    None
}

fn sample_world(voxel: Vector3<i32>, chunks: &[(Vector3<i32>, &Chunk)]) -> Option<u8> {
    // find which chunk this voxel is in
    let chunk_pos = Vector3::new(
        voxel.x.div_euclid(32),
        voxel.y.div_euclid(32),
        voxel.z.div_euclid(32),
    );

    let chunk = chunks
        .iter()
        .find(|(pos, _)| *pos == chunk_pos)
        .map(|(_, c)| c)?;

    let local = Vector3::new(
        voxel.x.rem_euclid(32) as u32,
        voxel.y.rem_euclid(32) as u32,
        voxel.z.rem_euclid(32) as u32,
    );

    let id = chunk.voxels[flatten(local.x, local.y, local.z, 32)];
    if id != 0 { Some(0) } else { None }
}

pub fn get_camera_ray(transform: &Transform) -> Ray {
    Ray::new(transform.global_position, transform.calculate_forward())
}

/// Creates a raycast hit then submits it to the world for usage
pub fn voxel_raycast_system(world: &mut World, set_to: Option<VoxelId>) -> Result<()> {
    let camera_obj = world
        .get_objects_with_component::<Camera>()
        .first()
        .copied()
        .ok_or_else(|| anyhow::anyhow!("No camera"))?;

    let transform = camera_obj.get_component::<Transform>()?.clone();
    let ray = get_camera_ray(&transform);

    // collect chunks for sampling
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

    if let Some(hit) = raycast(&ray, 10.0, &chunk_refs, set_to) {
        world.insert_resource(hit);
    }

    Ok(())
}

/// Creates a raycast hit then returns it
pub fn voxel_raycast(world: &mut World, set_to: Option<VoxelId>) -> Result<RaycastHit> {
    let camera_obj = world
        .get_objects_with_component::<Camera>()
        .first()
        .copied()
        .ok_or_else(|| anyhow::anyhow!("No camera"))?;

    let transform = camera_obj.get_component::<Transform>()?.clone();
    let ray = get_camera_ray(&transform);

    // collect chunks for sampling
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

    let hit = raycast(&ray, 10.0, &chunk_refs, set_to);

    return hit.ok_or(Error::msg("Hit nothing"));
}
