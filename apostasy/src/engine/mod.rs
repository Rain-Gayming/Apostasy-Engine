use anyhow::Result;
use std::{collections::HashMap, sync::Arc};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId},
    event_loop::{ControlFlow, EventLoop},
};

use winit::{
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};

use crate::engine::{
    ecs::resources::input_manager::{InputManager, handle_device_event, handle_input_event},
    rendering::{
        models::model::{ModelLoader, load_models},
        queue_families::queue_family_picker::single_queue_family,
        renderer::Renderer,
        rendering_context::{RenderingContext, RenderingContextAttributes},
    },
    timer::EngineTimer,
    windowing::WindowManager,
};

use crate::engine::ecs::World;

pub mod ecs;
pub mod editor;
pub mod rendering;
pub mod timer;
pub mod voxels;
pub mod windowing;

/// Render application
pub struct Application {
    render_engine: Option<Engine>,
    _world: Option<World>,
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.render_engine = Some(Engine::new(event_loop).unwrap());
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let Some(engine) = self.render_engine.as_mut() {
            engine.window_event(event_loop, window_id, event.clone());
        }
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let Some(engine) = self.render_engine.as_mut() {
            engine.device_event(event_loop, device_id, event.clone());
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(engine) = &mut self.render_engine {
            engine.request_redraw();
        }
    }

    fn suspended(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        event_loop.exit();
    }
}

pub fn start_app() -> Result<()> {
    tracing_subscriber::fmt::init();
    let mut app = Application {
        render_engine: None,
        _world: None,
    };

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app)?;

    Ok(())
}
/// The render engine, contains all the data for rendering, windowing and their logic
pub struct Engine {
    pub renderers: HashMap<WindowId, Renderer>,
    pub world: World,
    pub rendering_context: Arc<RenderingContext>,
    pub timer: EngineTimer,
}

impl Engine {
    pub fn new(event_loop: &ActiveEventLoop) -> Result<Self> {
        let primary_window = Arc::new(
            event_loop.create_window(
                Window::default_attributes()
                    .with_title("Apostasy")
                    .with_visible(true),
            )?,
        );
        let primary_window_id = primary_window.id();
        let windows = HashMap::from([(primary_window_id, primary_window.clone())]);

        let rendering_context = Arc::new(RenderingContext::new(RenderingContextAttributes {
            compatability_window: &primary_window,
            queue_family_picker: single_queue_family,
        })?);

        let mut world = World::new(rendering_context.clone());
        world.start();

        let renderers = windows
            .iter()
            .map(|(id, window)| {
                let renderer = Renderer::new(rendering_context.clone(), window.clone()).unwrap();
                (*id, renderer)
            })
            .collect::<HashMap<WindowId, Renderer>>();

        let timer = EngineTimer::new();

        world.insert_resource::<WindowManager>(WindowManager::default());

        world.with_resource_mut(|window_manager: &mut WindowManager| {
            window_manager.primary_window_id = primary_window_id;
            window_manager
                .windows
                .insert(primary_window_id, primary_window.clone());
        });

        world.with_resource_mut(|model_loader: &mut ModelLoader| {
            load_models(model_loader, &rendering_context);
        });

        Ok(Self {
            renderers,
            world,
            rendering_context,
            timer,
        })
    }

    pub fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        self.world
            .with_resource_mut(|input_manager: &mut InputManager| {
                handle_input_event(input_manager, event.clone());
            });

        self.world
            .with_resource_mut(|window_manager: &mut WindowManager| {
                if let Some(renderer) = self.renderers.get_mut(&window_id) {
                    renderer.window_event(
                        window_manager.windows.get(&window_id).unwrap(),
                        event.clone(),
                    );
                }
            });

        match event.clone() {
            WindowEvent::Resized(_size) => {
                if let Some(renderer) = self.renderers.get_mut(&window_id) {
                    renderer.resize().unwrap();
                }
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                if let Some(renderer) = self.renderers.get_mut(&window_id) {
                    renderer.resize().unwrap();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = self.renderers.get_mut(&window_id) {
                    let windows: Vec<Arc<Window>> =
                        self.world.with_resource(|window_manager: &WindowManager| {
                            window_manager.windows.values().cloned().collect()
                        });

                    for window in &windows {
                        renderer.prepare_egui(window, &mut self.world);
                    }

                    let _ = renderer.render(&self.world);
                }
            }

            _ => (),
        }
    }

    pub fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        self.world
            .with_resource_mut(|input_manager: &mut InputManager| {
                handle_device_event(input_manager, event.clone());
            });
    }

    pub fn request_redraw(&mut self) {
        self.world.update();
        self.world.fixed_update(self.timer.tick().fixed_dt);

        self.world
            .with_resource_mut(|window_manager: &mut WindowManager| {
                for window in window_manager.windows.values() {
                    window.request_redraw();
                }
            });

        self.world.late_update();
    }

    pub fn create_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        attributes: WindowAttributes,
    ) -> Result<WindowId> {
        let window = Arc::new(event_loop.create_window(attributes)?);
        let window_id = window.id();

        self.world
            .with_resource_mut::<WindowManager, _, _>(|window_manager| {
                window_manager.windows.insert(window_id, window.clone());
            });

        let renderer = Renderer::new(self.rendering_context.clone(), window)?;
        self.renderers.insert(window_id, renderer);
        Ok(window_id)
    }
}
