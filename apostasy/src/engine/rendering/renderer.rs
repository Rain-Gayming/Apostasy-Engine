use std::sync::Arc;

use anyhow::Result;
use winit::{
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use crate::engine::rendering::rendering_context::RenderingContext;

pub struct Renderer {
    pub context: Arc<RenderingContext>,
}

impl Renderer {
    pub fn new(context: Arc<RenderingContext>, window: Arc<Window>) -> Result<Self> {
        Ok(Self { context })
    }

    pub fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
    }
}
