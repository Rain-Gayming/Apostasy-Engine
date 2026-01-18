use anyhow::Result;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
};

use crate::engine::{
    ecs::{
        World,
        resources::input_manager::{InputManager, handle_input_event},
    },
    rendering::render_engine::RenderEngine,
};

pub mod physical_device;
pub mod queue_families;
pub mod render_engine;
pub mod renderer;
pub mod rendering_context;
pub mod surface;
pub mod swapchain;

/// Render application
pub struct Application {
    render_engine: Option<RenderEngine>,
    world: Option<World>,
}

impl Application {
    fn update(&mut self) {
        if let Some(world) = self.world.as_mut() {
            world.update();
        }
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(world) = self.world.take() {
            self.render_engine = Some(RenderEngine::new(event_loop, world).unwrap());
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

    app.update();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app)?;

    Ok(())
}
