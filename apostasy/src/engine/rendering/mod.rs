use anyhow::Result;
use winit::{application::ApplicationHandler, event_loop::EventLoop};

use crate::engine::rendering::render_engine::RenderEngine;

pub mod physical_device;
pub mod queue_families;
pub mod render_context;
pub mod render_engine;
pub mod renderer;

#[derive(Default)]
pub struct Application {
    render_engine: Option<RenderEngine>,
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.render_engine = Some(RenderEngine::new(event_loop));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let Some(engine) = self.render_engine.as_mut() {
            engine.window_event(event_loop, window_id, event);
        }
    }

    fn suspended(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.render_engine = None;
    }
}

pub fn start_renderer() -> Result<()> {
    tracing_subscriber::fmt::init();
    let mut app = Application::default();

    let event_loop = EventLoop::new()?;
    event_loop.run_app(&mut app)?;

    Ok(())
}
