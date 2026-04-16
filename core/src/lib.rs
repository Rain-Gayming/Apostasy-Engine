extern crate self as apostasy_core;

pub use apostasy_macros::Component;
pub use apostasy_macros::fixed_update;
pub use apostasy_macros::late_update;
pub use apostasy_macros::start;
pub use apostasy_macros::update;

pub use anyhow;

use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use winit::{
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

use crate::assets::gltf::load_model;
use crate::rendering::components::camera::Camera;
use crate::rendering::components::model_renderer::ModelRenderer;
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

pub struct Core {
    pub rendering_api: RenderingBackend,
    pub rendering_info: Option<Arc<Mutex<RenderingInfo>>>,
    pub world: Arc<Mutex<World>>,
}

impl Core {
    pub fn new(rendering_api: RenderingBackend) -> Self {
        Self {
            rendering_api,
            rendering_info: None,
            world: Arc::new(Mutex::new(World::default())),
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
                                mesh_renderer.model = Some(
                                    load_model(
                                        Path::new(&mesh_renderer.model_path),
                                        context.clone(),
                                        command_pool,
                                    )
                                    .unwrap(),
                                );
                                mesh_renderer.model.as_ref().unwrap()
                            }
                        };

                        for mesh in &model.meshes {
                            renderer
                                .render(mesh.clone(), push_constants.clone())
                                .unwrap();
                        }
                    }

                    world.late_update();
                }
                _ => {}
            }
        }
    }

    pub fn redraw(world: &mut World, rendering_info: &mut RenderingInfo) {}
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
