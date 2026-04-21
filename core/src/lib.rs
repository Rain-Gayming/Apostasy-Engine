extern crate self as apostasy_core;

pub use apostasy_macros::Component;
pub use apostasy_macros::fixed_update;
pub use apostasy_macros::late_update;
pub use apostasy_macros::start;
pub use apostasy_macros::update;

pub use anyhow;
pub use cgmath;
use gltf::json::Asset;
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
use crate::rendering::components::camera::Camera;
use crate::rendering::components::model_renderer::ModelRenderer;
use crate::voxels::voxel::VoxelRegistry;
use crate::{
    objects::world::World,
    rendering::{RenderingBackend, RenderingInfo},
};
use winit::application::ApplicationHandler;

pub mod assets;
pub mod objects;
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
    pub fn new(rendering_api: RenderingBackend) -> Self {
        let mut world = World::default();
        world.insert_resource(InputManager::default());

        let mut asset_manager = AssetManager::new();
        let voxel_registry = Arc::new(RwLock::new(VoxelRegistry::default()));
        asset_manager.register_loader(VoxelLoader {
            registry: Arc::clone(&voxel_registry),
        });

        // Read the files in apostasy-core's res/ folder
        asset_manager
            .load_directory(Path::new(&format!(
                "{}/{}",
                env!("CARGO_MANIFEST_DIR"),
                "res/"
            )))
            .unwrap();

        // Read the files in the project that impliments apostasy-core's res/ folder
        asset_manager.load_directory(Path::new("res/")).unwrap();

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

                    for object in world.get_objects_with_component_mut::<ModelRenderer>() {
                        let mesh_renderer = object.get_component_mut::<ModelRenderer>().unwrap();

                        let model = match &mesh_renderer.model {
                            Some(m) => m,
                            None => {
                                let Some(command_pool) = renderer.get_command_pool().ok() else {
                                    continue;
                                };
                                mesh_renderer.model = Some(Box::new(
                                    load_model(
                                        Path::new(&mesh_renderer.model_path),
                                        context.clone(),
                                        command_pool,
                                    )
                                    .unwrap(),
                                ));
                                mesh_renderer.model.as_ref().unwrap()
                            }
                        };

                        for mesh in &model.meshes {
                            renderer
                                .render(Box::new(mesh.clone()), push_constants.clone())
                                .unwrap();
                        }
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
pub fn init_core(rendering_api: RenderingBackend) -> Result<()> {
    let mut core = Core::new(rendering_api);

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
