use std::collections::HashMap;

use cgmath::Vector3;

use crate::game::{
    game_constants::CHUNK_SIZE,
    world::{chunk::Chunk, voxel_world::VoxelWorld},
};

#[derive(Clone, Copy)]
pub struct ChunkGenerator {
    pub last_chunk_position: Vector3<i32>,
}

impl Default for ChunkGenerator {
    fn default() -> Self {
        ChunkGenerator {
            last_chunk_position: Vector3::new(-1, -1, -1),
        }
    }
}

pub fn is_in_new_chunk(chunk_generator: &mut ChunkGenerator, new_position: Vector3<i32>) -> bool {
    let new_chunk_position = Vector3::new(
        new_position.x / CHUNK_SIZE as i32,
        new_position.y / CHUNK_SIZE as i32,
        new_position.z / CHUNK_SIZE as i32,
    );

    if new_chunk_position != chunk_generator.last_chunk_position {
        chunk_generator.last_chunk_position = new_chunk_position;
        return true;
    }
    false
}

pub fn load_chunks_in_range(chunk_generator: &mut ChunkGenerator, voxel_world: &mut VoxelWorld) {
    let chunk_size = 3;
    for x in (chunk_generator.last_chunk_position.x - chunk_size)
        ..(chunk_generator.last_chunk_position.x + chunk_size)
    {
        for y in (chunk_generator.last_chunk_position.y - chunk_size)
            ..(chunk_generator.last_chunk_position.y + chunk_size)
        {
            for z in (chunk_generator.last_chunk_position.z - chunk_size)
                ..(chunk_generator.last_chunk_position.z + chunk_size)
            {
                if !voxel_world
                    .chunks_loaded
                    .contains_key(&Vector3::new(x, y, z))
                {
                    voxel_world
                        .chunks_to_load
                        .insert(Vector3::new(x, y, z), Chunk::default());
                }
            }
        }
    }
}

pub fn get_adjacent_chunks(
    chunk_position: Vector3<i32>,
    chunks_to_check: &HashMap<Vector3<i32>, Chunk>,
) -> [Option<Chunk>; 6] {
    let x_positive_chunk = chunks_to_check
        .get(&Vector3::new(
            chunk_position.x + 1,
            chunk_position.y,
            chunk_position.z,
        ))
        .cloned();

    let x_negative_chunk = chunks_to_check
        .get(&Vector3::new(
            chunk_position.x - 1,
            chunk_position.y,
            chunk_position.z,
        ))
        .cloned();

    let y_positive_chunk = chunks_to_check
        .get(&Vector3::new(
            chunk_position.x,
            chunk_position.y + 1,
            chunk_position.z,
        ))
        .cloned();

    let y_negative_chunk = chunks_to_check
        .get(&Vector3::new(
            chunk_position.x,
            chunk_position.y - 1,
            chunk_position.z,
        ))
        .cloned();

    let z_positive_chunk = chunks_to_check
        .get(&Vector3::new(
            chunk_position.x,
            chunk_position.y,
            chunk_position.z + 1,
        ))
        .cloned();

    let z_negative_chunk = chunks_to_check
        .get(&Vector3::new(
            chunk_position.x,
            chunk_position.y,
            chunk_position.z - 1,
        ))
        .cloned();

    [
        z_negative_chunk,
        z_positive_chunk,
        x_positive_chunk,
        x_negative_chunk,
        y_negative_chunk,
        y_positive_chunk,
    ]
}
