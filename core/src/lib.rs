extern crate self as apostasy_core;
pub use apostasy_macros::Component;
pub use apostasy_macros::fixed_update;
pub use apostasy_macros::late_update;
pub use apostasy_macros::start;
pub use apostasy_macros::update;

pub use anyhow;
pub use cgmath;
use cgmath::Vector3;
pub use winit;
use winit::event::DeviceEvent;
use winit::event::DeviceId;

use std::sync::{Arc, Mutex};

use anyhow::Result;
use winit::{
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

use crate::assets::asset_manager::AssetManager;
use crate::objects::resources::input_manager::InputManager;
use crate::packages::Packages;
use crate::packages::add_package;
use crate::rendering::components::camera::Camera;
use crate::voxels::VoxelTransform;
use crate::voxels::meshes::VoxelChunkMesh;
use crate::voxels::meshes::remesh_chunks;
use crate::voxels::texture_atlas::PendingAtlas;
use crate::voxels::texture_atlas::VoxelTextureAtlas;
use crate::voxels::texture_atlas::upload_atlas;
use crate::{
    objects::world::World,
    rendering::{RenderingBackend, RenderingInfo},
};
use winit::application::ApplicationHandler;

pub mod assets;
pub mod objects;
pub mod packages;
pub mod physics;
pub mod rendering;
pub mod utils;
pub mod voxels;

pub struct Core {
    pub rendering_api: RenderingBackend,
    pub rendering_info: Option<Arc<Mutex<RenderingInfo>>>,
    pub world: Arc<Mutex<World>>,
    pub asset_loader: AssetManager,
}

impl Core {
    pub fn new(rendering_api: RenderingBackend, packages: Vec<Packages>) -> Self {
        let mut world = World::default();
        world.insert_resource(InputManager::default());

        for package in packages {
            add_package(&mut world, package);
        }

        Self {
            rendering_api,
            rendering_info: None,
            world: Arc::new(Mutex::new(world)),
            asset_loader: AssetManager::new(),
        }
    }
    pub fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(rendering_info) = &mut self.rendering_info {
            let mut rendering_info = rendering_info.lock().unwrap();

            match event {
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                WindowEvent::Resized(_) => {
                    if let Some(renderer) = &mut rendering_info.renderer {
                        renderer.resize().unwrap();
                    }
                }
                WindowEvent::ScaleFactorChanged { .. } => {
                    if let Some(renderer) = &mut rendering_info.renderer {
                        renderer.resize().unwrap();
                    }
                }
                WindowEvent::RedrawRequested => {
                    let mut world = self.world.lock().unwrap();
                    world.update();

                    let context = Arc::new(rendering_info.context.clone());
                    let mut push_constants = rendering_info.push_constants.clone();

                    if let Some(atlas) = world.get_resource::<VoxelTextureAtlas>().ok() {
                        push_constants.set_atlas_tiles(atlas.atlas_size);
                    }

                    let Some(renderer) = &mut rendering_info.renderer else {
                        log_error!("No renderer found!");
                        return;
                    };
                    let &camera = world
                        .get_objects_with_component::<Camera>()
                        .first()
                        .unwrap();

                    let aspect = renderer.get_aspect();

                    let mut push_constants = push_constants;
                    push_constants.set_camera_constants(camera.to_owned(), aspect);

                    if let Ok(command_pool) = renderer.get_command_pool() {
                        remesh_chunks(&mut world, &context, command_pool)
                            .expect("Failed to remesh chunks");
                    }
                    renderer.begin_frame(push_constants.clone()).unwrap();
                    if let Some(texture_atlas) = world.get_resource::<VoxelTextureAtlas>().ok() {
                        for object in world.get_objects_with_component::<VoxelChunkMesh>() {
                            let transform = object.get_component::<VoxelTransform>().unwrap();
                            let voxel_mesh = object.get_component::<VoxelChunkMesh>().unwrap();

                            let mut chunk_push = push_constants.clone();

                            chunk_push.set_position(Vector3::new(
                                transform.position.x * 32,
                                transform.position.y * 32,
                                transform.position.z * 32,
                            ));
                            renderer
                                .voxel_render(
                                    Box::new(voxel_mesh.clone()),
                                    texture_atlas.clone(),
                                    chunk_push,
                                )
                                .unwrap();
                        }
                    }

                    renderer.end_frame().unwrap();
                    world.late_update();
                }

                _ => {}
            }
            let mut world = self.world.lock().unwrap();
            let input_manager = world.get_resource_mut::<InputManager>().unwrap();
            input_manager.handle_input_event(event.clone());
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let mut world = self.world.lock().unwrap();
        let input_manager = world.get_resource_mut::<InputManager>().unwrap();
        input_manager.handle_device_event(event.clone());
    }
}
impl ApplicationHandler for Core {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.rendering_info = Some(RenderingInfo::new(&event_loop, self.rendering_api));

        unsafe {
            let mut world = self.world.lock().unwrap();
            let context = self
                .rendering_info
                .clone()
                .unwrap()
                .lock()
                .unwrap()
                .context
                .clone();

            let pending = world.get_resource::<PendingAtlas>().unwrap().clone();

            let (command_pool, descriptor_pool, descriptor_set_layout) = {
                let ri = self.rendering_info.as_ref().unwrap().lock().unwrap();
                let renderer = ri.renderer.as_ref().unwrap();
                (
                    renderer.get_command_pool().unwrap(),
                    renderer.get_descriptor_pool(),
                    renderer.get_voxel_descriptor_set_layout(),
                )
            };

            let atlas = upload_atlas(
                &context,
                command_pool,
                descriptor_pool,
                descriptor_set_layout,
                &pending.image,
                pending.tiles,
            )
            .expect("Failed to upload voxel atlas");

            world.insert_resource(context);
            world.insert_resource(atlas);
        }
    }
    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        self.window_event(event_loop, window_id, event);
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        self.device_event(event_loop, device_id, event);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(render_info) = &self.rendering_info {
            render_info.lock().unwrap().window.request_redraw();
        }
    }
}

/// Initializes the core of the application
/// Note: nothing can run in main after this
/// Note: automatically runs all start systems
pub fn init_core(rendering_api: RenderingBackend, packages: Vec<Packages>) -> Result<()> {
    let mut core = Core::new(rendering_api, packages);

    // run all start systems
    {
        let mut world = core.world.lock().unwrap();

        world.start();
    }

    // begin event loop
    let event_loop = EventLoop::new()?;
    event_loop.run_app(&mut core)?;

    Ok(())
}
