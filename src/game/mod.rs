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
            chunk_generator::{
                create_new_chunk, get_adjacent_chunks, get_chunks_in_range, is_in_new_chunk,
            },
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
                let mut chunks = Vec::new();
                for chunk in get_chunks_in_range(
                    &mut self.player.chunk_generator,
                    &mut self.world.voxel_world,
                ) {
                    chunks.push(create_new_chunk(chunk, &mut self.world.voxel_world));
                }

                for position in self.world.voxel_world.chunks_to_unmesh.clone() {
                    let offset = [position.x, position.y, position.z];

                    if renderer.index_offset.contains(&offset) {
                        let offset_index = renderer
                            .index_offset
                            .iter()
                            .position(|&pos| pos == offset)
                            .unwrap();

                        renderer.vertex_buffers.remove(offset_index);
                        renderer.index_buffers.remove(offset_index);
                        renderer.index_counts.remove(offset_index);
                        renderer.index_offset.remove(offset_index);
                        self.world.voxel_world.chunks_to_unmesh.remove(offset_index);
                    }
                }

                for (position, mut chunk) in self.world.voxel_world.chunks.clone() {
                    let adjacent_chunks = get_adjacent_chunks(position, &self.world.voxel_world);
                    render_chunk(&mut chunk, renderer, adjacent_chunks);
                }
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
