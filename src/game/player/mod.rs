use std::sync::{Arc, Mutex};

use crate::{app::engine::renderer::camera::Camera, game::world::chunk_generator::ChunkGenerator};

pub struct Player {
    pub chunk_generator: ChunkGenerator,
    pub camera: Arc<Mutex<Camera>>,
}

impl Default for Player {
    fn default() -> Self {
        let camera = Arc::new(Mutex::new(Camera::default()));

        let chunk_generator = ChunkGenerator::default();

        Player {
            chunk_generator,
            camera,
        }
    }
}
