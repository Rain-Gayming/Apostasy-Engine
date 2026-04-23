use anyhow::Result;
use apostasy_core::log;
use apostasy_macros::{Resource, update};
use cgmath::Vector3;

use crate::{
    objects::{components::transform::Transform, tags::Player, world::World},
    voxels::{
        VoxelTransform,
        chunk::{Chunk, generate_chunk},
        meshes::NeedsRemeshing,
        voxel::VoxelRegistry,
    },
};

#[derive(Resource, Clone)]
pub struct ChunkLoader {
    pub loaded_chunk_ids: Vec<u64>,
    pub last_chunk_position: Vector3<i32>,
    pub load_radius: i32,
}

impl Default for ChunkLoader {
    fn default() -> Self {
        Self {
            loaded_chunk_ids: Vec::new(),
            last_chunk_position: Vector3::new(-1, 0, 0),
            load_radius: 2,
        }
    }
}

#[update]
pub fn update_chunks(world: &mut World) -> Result<()> {
    let player = world.get_object_with_tag::<Player>()?;
    let player_transform = player.get_component::<Transform>()?;

    let player_chunk_position = Vector3::new(
        (player_transform.global_position.x / 32.0).floor() as i32,
        (player_transform.global_position.y / 32.0).floor() as i32,
        (player_transform.global_position.z / 32.0).floor() as i32,
    );

    let last_chunk_position = world.get_resource::<ChunkLoader>()?.last_chunk_position;

    if last_chunk_position == player_chunk_position {
        return Ok(());
    }

    log!("Entered new chunk at {:?}", player_chunk_position);

    world.get_resource_mut::<ChunkLoader>()?.last_chunk_position = player_chunk_position;

    let load_radius = world.get_resource::<ChunkLoader>()?.load_radius;
    let registry = world.get_resource::<VoxelRegistry>()?.clone();

    // collect existing chunk positions so we dont spawn duplicates
    let existing_positions: Vec<Vector3<i32>> = world
        .get_objects_with_component::<VoxelTransform>()
        .iter()
        .filter_map(|o| o.get_component::<VoxelTransform>().ok())
        .map(|t| t.position)
        .collect();

    // generate new chunks and track their positions
    let mut new_positions = Vec::new();
    let mut new_chunks = Vec::new();

    for x in (player_chunk_position.x - load_radius)..=(player_chunk_position.x + load_radius) {
        for y in (player_chunk_position.y - load_radius)..=(player_chunk_position.y + load_radius) {
            for z in
                (player_chunk_position.z - load_radius)..=(player_chunk_position.z + load_radius)
            {
                let pos = Vector3::new(x, y, z);

                // skip if already loaded
                if existing_positions.contains(&pos) {
                    continue;
                }

                new_positions.push(pos);
                new_chunks.push(generate_chunk(pos, &registry, 1));
            }
        }
    }

    // add new chunks to world
    let chunk_loader = world.get_resource_mut::<ChunkLoader>()?;
    for chunk in &new_chunks {
        chunk_loader.loaded_chunk_ids.push(chunk.id);
    }

    for chunk in new_chunks {
        world.add_object(chunk);
    }

    let neighbour_offsets = [
        Vector3::new(1, 0, 0),
        Vector3::new(-1, 0, 0),
        Vector3::new(0, 1, 0),
        Vector3::new(0, -1, 0),
        Vector3::new(0, 0, 1),
        Vector3::new(0, 0, -1),
    ];

    let mut neighbour_ids_to_remesh: Vec<u64> = Vec::new();
    for new_pos in &new_positions {
        for offset in &neighbour_offsets {
            let neighbour_pos = new_pos + offset;
            for obj in world.get_objects_with_component::<VoxelTransform>() {
                if let Ok(transform) = obj.get_component::<VoxelTransform>() {
                    if transform.position == neighbour_pos
                        && !new_positions.contains(&neighbour_pos)
                    {
                        neighbour_ids_to_remesh.push(obj.id);
                    }
                }
            }
        }
    }

    // mark neighbours for remeshing
    for id in neighbour_ids_to_remesh {
        if let Some(obj) = world.get_object_mut(id) {
            // obj.add_tag(NeedsRemeshing);
        }
    }

    Ok(())
}
