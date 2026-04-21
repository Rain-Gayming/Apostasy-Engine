extern crate self as apostasy_core;

use anyhow::Context;
pub use apostasy_macros::Component;
pub use apostasy_macros::fixed_update;
pub use apostasy_macros::late_update;
pub use apostasy_macros::start;
pub use apostasy_macros::update;

pub use anyhow;
pub use cgmath;
pub use winit;
use winit::event::DeviceEvent;
use winit::event::DeviceId;

use std::path::Path;
use std::sync::RwLock;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use winit::{
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

use crate::assets::asset_manager::AssetManager;
use crate::assets::gltf::load_model;
use crate::assets::loaders::voxel_loader::VoxelLoader;
use crate::objects::resources::input_manager::InputManager;
use crate::packages::Packages;
use crate::rendering::components::camera::Camera;
use crate::rendering::components::model_renderer::ModelRenderer;
use crate::voxels::meshes::VoxelChunkMesh;
use crate::voxels::meshes::remesh_chunks;
use crate::voxels::voxel::VoxelRegistry;
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
    pub fn new(rendering_api: RenderingBackend, _packages: Vec<Packages>) -> Self {
        let asset_manager = AssetManager::new();
        let mut world = World::default();
        world.insert_resource(InputManager::default());

        let voxel_registry = Arc::new(RwLock::new(VoxelRegistry::default()));

        {
            let mut asset_manager = AssetManager::new();
            asset_manager.register_loader(VoxelLoader {
                registry: Arc::clone(&voxel_registry),
            });

            asset_manager
                .load_directory(Path::new(&format!(
                    "{}/{}",
                    env!("CARGO_MANIFEST_DIR"),
                    "res/"
                )))
                .unwrap();

            // Read the files in the project that impliments apostasy-core's res/ folder
            asset_manager.load_directory(Path::new("res/")).unwrap();
        }

        let registry = Arc::try_unwrap(voxel_registry)
            .expect("VoxelRegistry still has multiple owners")
            .into_inner()
            .expect("VoxelRegistry RwLock poisoned");

        world.insert_resource(registry);

        Self {
            rendering_api,
            rendering_info: None,
            world: Arc::new(Mutex::new(world)),
            asset_loader: asset_manager,
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
                    let push_constants = rendering_info.push_constants.clone();

                    let Some(renderer) = &mut rendering_info.renderer else {
                        log_error!("No renderer found!");
                        return;
                    };
                    let Some(&camera) = world.get_objects_with_component::<Camera>().first() else {
                        log_error!("No camera found!");
                        return;
                    };

                    let aspect = renderer.get_aspect();

                    let mut push_constants = push_constants;
                    push_constants.set_camera_constants(camera.to_owned(), aspect);

                    if let Ok(command_pool) = renderer.get_command_pool() {
                        remesh_chunks(&mut world, &context, command_pool)
                            .expect("Failed to remesh chunks");
                    }

                    for object in world.get_objects_with_component::<VoxelChunkMesh>() {
                        let voxel_mesh = object.get_component::<VoxelChunkMesh>().unwrap();

                        renderer
                            .voxel_render(Box::new(voxel_mesh.clone()), push_constants.clone())
                            .unwrap();
                    }

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

        {
            let mut world = self.world.lock().unwrap();
            let context = self
                .rendering_info
                .clone()
                .unwrap()
                .lock()
                .unwrap()
                .context
                .clone();

            world.insert_resource(context);
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
