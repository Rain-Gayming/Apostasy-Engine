use anyhow::Result;
use apostasy_core::log;
use apostasy_macros::{Resource, fixed_update, start, update};
use cgmath::Vector3;
use hashbrown::{HashMap, HashSet};

use crate::{
    objects::{
        components::transform::Transform, scene::ObjectId, systems::DeltaTime, tags::Player,
        world::World,
    },
    physics::velocity::Velocity,
    voxels::{
        VoxelTransform,
        biome::BiomeRegistry,
        chunk::{self, Chunk, generate_chunk},
        meshes::NeedsRemeshing,
        voxel::VoxelRegistry,
    },
};

#[derive(Resource, Clone)]
pub struct ChunkLoader {
    pub last_chunk_position: Vector3<i32>,
    pub load_radius: i32,

    pub chunk_lod_distances: Vec<u32>,
    pub min_ticks: u32,
    pub current_ticks: u32,
}

impl Default for ChunkLoader {
    fn default() -> Self {
        Self {
            last_chunk_position: Vector3::new(-1, 0, 0),
            load_radius: 4,
            chunk_lod_distances: vec![5, 8, 10, 14],
            min_ticks: 30,
            current_ticks: 0,
        }
    }
}

#[start]
pub fn update_chunks_init(world: &mut World) -> Result<()> {
    update_chunks(world, 0.0)
}

#[fixed_update]
pub fn update_chunks(world: &mut World, _delta: f32) -> Result<()> {
    // let chunk_loader = world.get_resource_mut::<ChunkLoader>()?;
    //
    // chunk_loader.current_ticks += 1;
    //
    //
    // if chunk_loader.current_ticks != chunk_loader.min_ticks{
    //     return Ok(())
    // }
    //
    // chunk_loader.current_ticks = 0;

    let load_radius = world.get_resource::<ChunkLoader>()?.load_radius;
    let registry = world.get_resource::<VoxelRegistry>()?.clone();
    let biome_registry = world.get_resource::<BiomeRegistry>()?.clone();

    let player = world.get_object_with_tag::<Player>()?;
    let player_transform = player.get_component::<Transform>()?;
    let player_velocity = player.get_component::<Velocity>()?;

    let player_chunk_position = Vector3::new(
        (player_transform.global_position.x / 32.0).floor() as i32,
        (player_transform.global_position.y / 32.0).floor() as i32,
        (player_transform.global_position.z / 32.0).floor() as i32,
    );

    let last_chunk_position = world.get_resource::<ChunkLoader>()?.last_chunk_position;

    if last_chunk_position == player_chunk_position {
        return Ok(());
    }

    // only accept the new chunk position if the player is moving toward it,
    // not being pushed back across the boundary by collision resolution
    let delta = player_chunk_position - last_chunk_position;
    let vel = player_velocity.linear_velocity;
    let moving_toward = (delta.x != 0 && delta.x.signum() == vel.x.signum() as i32)
        || (delta.y != 0 && delta.y.signum() == vel.y.signum() as i32)
        || (delta.z != 0 && delta.z.signum() == vel.z.signum() as i32);

    if !moving_toward {
        return Ok(());
    }

    log!("Entered new chunk at {:?}", player_chunk_position);

    world.get_resource_mut::<ChunkLoader>()?.last_chunk_position = player_chunk_position;

    // collect ids of chunks too far away to unload
    let unload_ids: Vec<ObjectId> = world
        .get_objects_with_component_with_ids::<Chunk>()
        .into_iter()
        .filter_map(|(id, o)| {
            let pos = o.get_component::<VoxelTransform>().ok()?.position;
            let dx = (pos.x - player_chunk_position.x).abs();
            let dy = (pos.y - player_chunk_position.y).abs();
            let dz = (pos.z - player_chunk_position.z).abs();
            if dx > load_radius || dy > load_radius || dz > load_radius {
                Some(id)
            } else {
                None
            }
        })
        .collect();

    // unload distant chunks
    for id in unload_ids {
        world.remove_object(id);
    }

    // collect existing chunk positions so we dont spawn duplicates
    let existing_positions: Vec<Vector3<i32>> = world
        .get_objects_with_component::<VoxelTransform>()
        .iter()
        .filter_map(|o| o.get_component::<VoxelTransform>().ok())
        .map(|t| t.position)
        .collect();
    let existing_position_ids: Vec<ObjectId> = world
        .get_objects_with_component_with_ids::<VoxelTransform>()
        .iter()
        .map(|o| o.0)
        .collect();

    // generate new chunks and track their positions
    let mut new_positions = Vec::new();

    for x in (player_chunk_position.x - load_radius)..=(player_chunk_position.x + load_radius) {
        for y in (player_chunk_position.y - load_radius)..=(player_chunk_position.y + load_radius) {
            for z in
                (player_chunk_position.z - load_radius)..=(player_chunk_position.z + load_radius)
            {
                let pos = Vector3::new(x, y, z);

                let chunk_loader = world.get_resource::<ChunkLoader>()?;

                // check if lod has changed then skip
                if existing_positions.contains(&pos) {
                    for lod in chunk_loader.clone().chunk_lod_distances {
                        let dx = (pos.x - player_chunk_position.x) as f32;
                        let dy = (pos.y - player_chunk_position.y) as f32;
                        let dz = (pos.z - player_chunk_position.z) as f32;

                        let distance = (dx * dx + dy * dy + dz * dz).sqrt().floor();
                        if distance < lod as f32 {
                            new_positions.push(pos);

                            let lod = chunk_loader
                                .chunk_lod_distances
                                .iter()
                                .position(|&r| r == lod)
                                .unwrap();

                            let obj_index =
                                existing_positions.iter().position(|&r| r == pos).unwrap();

                            let object = world
                                .get_object_mut(
                                    existing_position_ids.get(obj_index).unwrap().clone(),
                                )
                                .unwrap();

                            let chunk = object.get_component_mut::<Chunk>().unwrap();
                            if lod as u8 != chunk.lod {
                                object.get_component_mut::<Chunk>().unwrap().lod = lod as u8 + 1;
                                object.add_tag(NeedsRemeshing);
                            }

                            break;
                        }
                    }
                    continue;
                }

                for lod in chunk_loader.clone().chunk_lod_distances {
                    let dx = (pos.x - player_chunk_position.x) as f32;
                    let dy = (pos.y - player_chunk_position.y) as f32;
                    let dz = (pos.z - player_chunk_position.z) as f32;

                    let distance = (dx * dx + dy * dy + dz * dz).sqrt().floor();
                    if distance < lod as f32 {
                        new_positions.push(pos);

                        let lod = chunk_loader
                            .chunk_lod_distances
                            .iter()
                            .position(|&r| r == lod)
                            .unwrap();

                        world.add_object(generate_chunk(
                            pos,
                            &registry,
                            &biome_registry,
                            12311231 as u32,
                            lod as u8 + 1,
                        ));
                        break;
                    }
                }
            }
        }
    }

    let neighbour_offsets = [
        Vector3::new(1, 0, 0),
        Vector3::new(-1, 0, 0),
        Vector3::new(0, 1, 0),
        Vector3::new(0, -1, 0),
        Vector3::new(0, 0, 1),
        Vector3::new(0, 0, -1),
    ];

    let position_to_id: HashMap<Vector3<i32>, ObjectId> = world
        .get_objects_with_component_with_ids::<VoxelTransform>()
        .into_iter()
        .filter_map(|(id, obj)| {
            let pos = obj.get_component::<VoxelTransform>().ok()?.position;
            Some((pos, id))
        })
        .collect();

    let new_positions_set: HashSet<Vector3<i32>> = new_positions.iter().cloned().collect();
    let mut neighbour_ids_to_remesh: Vec<ObjectId> = Vec::new();
    for new_pos in &new_positions_set {
        for offset in &neighbour_offsets {
            let neighbour_pos = new_pos + offset;
            if !new_positions.contains(&neighbour_pos) {
                if let Some(&id) = position_to_id.get(&neighbour_pos) {
                    neighbour_ids_to_remesh.push(id);
                }
            }
        }
    }

    // mark neighbours for remeshing
    for id in neighbour_ids_to_remesh {
        let chunk = world.get_object_mut(id).unwrap();
        chunk.add_tag(NeedsRemeshing);
    }
    Ok(())
}
