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
        world::{
            chunk::generate_chunk,
            chunk_generator::{get_adjacent_chunks, is_in_new_chunk, load_chunks_in_range},
            chunk_renderer::render_chunk,
            new_world, World,
        },
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
        if update_camera_position(self.player.camera.clone()) {
            let camera = self.player.camera.lock().unwrap();

            if is_in_new_chunk(
                &mut self.player.chunk_generator,
                Vector3::new(
                    camera.position.x as i32,
                    camera.position.y as i32,
                    camera.position.z as i32,
                ),
            ) {
                let mut chunks_loaded = 0;
                load_chunks_in_range(
                    &mut self.player.chunk_generator,
                    &mut self.world.voxel_world,
                );

                for (position, chunk) in self.world.voxel_world.chunks_to_load.iter_mut() {
                    generate_chunk(*position, chunk);

                    self.world
                        .voxel_world
                        .chunks_loaded
                        .insert(*position, chunk.clone());

                    chunks_loaded += 1;
                }
                for (position, mut chunk) in self.world.voxel_world.chunks_to_load.clone() {
                    let adjacent_chunks =
                        get_adjacent_chunks(position, &self.world.voxel_world.chunks_loaded);

                    render_chunk(&mut chunk, renderer, adjacent_chunks.clone());

                    self.world
                        .voxel_world
                        .chunks_rendering
                        .insert(position, chunk);
                }

                self.world.voxel_world.chunks_to_load.clear();
                println!("loaded chunks: {chunks_loaded}");
            }
        }
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
