use std::sync::Arc;

use winit::{
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use crate::engine::rendering::render_context::{RenderContext, RenderContextAttributes};

pub struct Renderer {
    pub context: RenderContext,
}

impl Renderer {
    pub fn new(window: Arc<Window>) -> Self {
        let context = RenderContext::new(RenderContextAttributes { window }).unwrap();
        Self { context }
    }

    pub fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
    }
}
