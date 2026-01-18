use anyhow::Result;
use std::{collections::HashMap, sync::Arc};
use winit::{
    application::ApplicationHandler,
    event_loop::{ControlFlow, EventLoop},
};

use winit::{
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};

use crate::engine::{
    ecs::resources::input_manager::{InputManager, handle_input_event},
    rendering::{
        queue_families::queue_family_picker::single_queue_family,
        renderer::{Renderer, render},
        rendering_context::{RenderingContext, RenderingContextAttributes},
    },
};

use crate::engine::ecs::World;

pub mod ecs;
pub mod rendering;

/// Render application
pub struct Application {
    render_engine: Option<Engine>,
    world: Option<World>,
}

impl Application {
    fn start(&mut self) {
        if let Some(world) = self.world.as_mut() {
            world.start();
        }
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(world) = self.world.take() {
            self.render_engine = Some(Engine::new(event_loop, world).unwrap());
        }
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

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(engine) = &mut self.render_engine {
            engine.request_redraw();
        }
    }

    fn suspended(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.render_engine = None;
    }
}

pub fn start_app(world: World) -> Result<()> {
    tracing_subscriber::fmt::init();
    let mut app = Application {
        render_engine: None,
        world: Some(world),
    };

    app.start();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app)?;

    Ok(())
}
/// The render engine, contains all the data for rendering, windowing and their logic
pub struct Engine {
    pub renderers: HashMap<WindowId, Renderer>,
    pub windows: HashMap<WindowId, Arc<Window>>,
    pub primary_window_id: WindowId,
    pub rendering_context: Arc<RenderingContext>,
    pub world: World,
}

impl Engine {
    pub fn new(event_loop: &ActiveEventLoop, world: World) -> Result<Self> {
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

        let renderers = windows
            .iter()
            .map(|(id, window)| {
                let renderer = Renderer::new(rendering_context.clone(), window.clone()).unwrap();
                (*id, renderer)
            })
            .collect::<HashMap<WindowId, Renderer>>();

        Ok(Self {
            renderers,
            windows,
            primary_window_id,
            rendering_context,
            world,
        })
    }

    pub fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        self.world
            .with_resource_mut::<InputManager, _>(|input_manager| {
                handle_input_event(input_manager, event.clone());
            });
        match event {
            WindowEvent::CloseRequested => {
                if window_id == self.primary_window_id {
                    event_loop.exit();
                } else {
                    self.windows.remove(&window_id);
                    self.renderers.remove(&window_id);
                }
            }
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
                self.world.update();
                if let Some(renderer) = self.renderers.get_mut(&window_id) {
                    let _ = render(renderer, &self.world);
                }
            }

            _ => (),
        }
    }

    pub fn request_redraw(&self) {
        for window in self.windows.values() {
            window.request_redraw();
        }
    }

    pub fn create_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        attributes: WindowAttributes,
    ) -> Result<WindowId> {
        let window = Arc::new(event_loop.create_window(attributes)?);
        let window_id = window.id();
        self.windows.insert(window_id, window.clone());

        let renderer = Renderer::new(self.rendering_context.clone(), window)?;
        self.renderers.insert(window_id, renderer);
        Ok(window_id)
    }
}
