use cgmath::Vector3;
use winit::event::{DeviceEvent, WindowEvent};

use crate::{
    app::engine::{
        input_manager::InputManager,
        renderer::{
            camera::{handle_camera_input, update_camera_position},
            Renderer,
        },
    },
    game::{
        player::Player,
        world::{new_world, World},
    },
};

pub mod game_constants;
pub mod player;
pub mod world;

pub struct Game {
    pub world: World,
    pub player: Player,
}
impl Game {
    pub fn update(&mut self, renderer: &mut Renderer) {
        update_camera_position(self.player.camera.clone());
    }

    pub fn window_event(&mut self, event: WindowEvent, input_manager: &mut InputManager) {
        if let WindowEvent::KeyboardInput { .. } = event {
            handle_camera_input(input_manager, &mut self.player.camera);
        }
    }

    pub fn device_event(
        &mut self,
        event: winit::event::DeviceEvent,
        input_manager: &mut InputManager,
    ) {
        if let DeviceEvent::MouseMotion { .. } = event {
            handle_camera_input(input_manager, &mut self.player.camera)
        }
    }
}

pub fn initialize_game() -> Game {
    let world = new_world();
    let player = Player::default();

    Game { world, player }
}
