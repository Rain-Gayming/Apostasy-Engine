use cgmath::Vector3;

use crate::{
    app::engine::renderer::Renderer,
    game::{
        game_constants::CHUNK_SIZE,
        world::{chunk::generate_chunk, chunk_renderer::render_chunk},
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

pub fn create_new_chunk(position: Vector3<i32>, renderer: &mut Renderer) {
    println!("generating new chunk at: {position:?}");
    let chunk = generate_chunk(position);
    render_chunk(&chunk, position, renderer);
}
