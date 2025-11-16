use winit::event::{DeviceEvent, WindowEvent};

use crate::app::engine::renderer::Renderer;

pub mod game_constants;
pub mod player;

pub struct Game {}
impl Game {
    pub fn update(&mut self, _renderer: &mut Renderer) {
        // update_camera_position(self.player.camera.clone());
    }

    pub fn window_event(&mut self, event: WindowEvent) {
        if let WindowEvent::KeyboardInput { .. } = event {}
    }

    pub fn device_event(
        &mut self,
        event: winit::event::DeviceEvent,
        // input_manager: &mut InputManager,
    ) {
        if let DeviceEvent::MouseMotion { .. } = event {
            // handle_camera_input(input_manager, &mut self.player.camera)
        }
    }
}

pub fn initialize_game() -> Game {
    Game {}
}
