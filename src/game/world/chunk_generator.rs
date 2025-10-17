use cgmath::{InnerSpace, Vector3};

use crate::{
    app::engine::renderer::Renderer,
    game::{
        game_constants::CHUNK_SIZE,
        world::{
            chunk::{generate_chunk, Chunk},
            voxel,
            voxel_world::VoxelWorld,
        },
    },
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

pub fn get_chunks_in_range(
    chunk_generator: &mut ChunkGenerator,
    voxel_world: &mut VoxelWorld,
) -> Vec<Vector3<i32>> {
    let mut chunks = Vec::new();
    for x in
        (chunk_generator.last_chunk_position.x - 8)..(chunk_generator.last_chunk_position.x + 8)
    {
        for y in
            (chunk_generator.last_chunk_position.y - 8)..(chunk_generator.last_chunk_position.y + 8)
        {
            for z in (chunk_generator.last_chunk_position.z - 8)
                ..(chunk_generator.last_chunk_position.z + 8)
            {
                chunks.push(Vector3::new(x, y, z));
            }
        }
    }

    for (position, _chunk) in voxel_world.chunks.clone() {
        let position_float = Vector3::new(position.x as f32, position.y as f32, position.z as f32);
        let generator_position_float = Vector3::new(
            chunk_generator.last_chunk_position.x as f32,
            chunk_generator.last_chunk_position.y as f32,
            chunk_generator.last_chunk_position.z as f32,
        );
        if (generator_position_float - position_float).magnitude() > 8.0 {
            voxel_world.chunks_to_unmesh.push(position);
            voxel_world.chunks.remove(&position);
        }
    }
    chunks
}

pub fn create_new_chunk(position: Vector3<i32>, voxel_world: &mut VoxelWorld) -> Chunk {
    let chunk = generate_chunk(position);
    voxel_world.chunks.insert(position, chunk.clone());
    chunk
}

pub fn get_adjacent_chunks(
    chunk_position: Vector3<i32>,
    voxel_world: &VoxelWorld,
) -> [Option<Chunk>; 6] {
    let x_positive_chunk = voxel_world
        .chunks
        .get(&Vector3::new(
            chunk_position.x + 1,
            chunk_position.y,
            chunk_position.z,
        ))
        .cloned();

    let x_negative_chunk = voxel_world
        .chunks
        .get(&Vector3::new(
            chunk_position.x - 1,
            chunk_position.y,
            chunk_position.z,
        ))
        .cloned();

    let y_positive_chunk = voxel_world
        .chunks
        .get(&Vector3::new(
            chunk_position.x,
            chunk_position.y + 1,
            chunk_position.z,
        ))
        .cloned();

    let y_negative_chunk = voxel_world
        .chunks
        .get(&Vector3::new(
            chunk_position.x,
            chunk_position.y - 1,
            chunk_position.z,
        ))
        .cloned();

    let z_positive_chunk = voxel_world
        .chunks
        .get(&Vector3::new(
            chunk_position.x,
            chunk_position.y,
            chunk_position.z + 1,
        ))
        .cloned();

    let z_negative_chunk = voxel_world
        .chunks
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
