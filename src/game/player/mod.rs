use std::sync::{Arc, Mutex};

use crate::app::engine::renderer::camera::Camera;

pub struct Player {
    pub camera: Arc<Mutex<Camera>>,
}

impl Default for Player {
    fn default() -> Self {
        let camera = Arc::new(Mutex::new(Camera::default()));

        Player { camera }
    }
}
