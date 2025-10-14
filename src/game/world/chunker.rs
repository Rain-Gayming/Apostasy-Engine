use cgmath::Vector3;

use crate::{
    app::engine::renderer::Renderer,
    game::{
        game_constants::CHUNK_SIZE,
        world::{chunk::generate_chunk, chunk_renderer::mesh_chunk},
    },
};

#[derive(Clone, Copy)]
pub struct Chunker {
    pub last_chunk_position: Vector3<i32>,
}

impl Default for Chunker {
    fn default() -> Self {
        Chunker {
            last_chunk_position: Vector3::new(-1, -1, -1),
        }
    }
}

pub fn is_in_new_chunk(chunker: &mut Chunker, new_position: Vector3<i32>) -> bool {
    let new_chunk_position = Vector3::new(
        new_position.x / CHUNK_SIZE as i32,
        new_position.y / CHUNK_SIZE as i32,
        new_position.z / CHUNK_SIZE as i32,
    );

    if new_chunk_position != chunker.last_chunk_position {
        chunker.last_chunk_position = new_chunk_position;
        return true;
    }
    false
}

pub fn create_new_chunk(position: Vector3<i32>, renderer: &mut Renderer) {
    println!("generating new chunk at: {position:?}");
    let chunk = generate_chunk(position);
    mesh_chunk(&chunk, position, renderer);
}
