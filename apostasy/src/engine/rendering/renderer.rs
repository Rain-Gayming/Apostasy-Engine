use std::sync::Arc;

use anyhow::Result;
use winit::{
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use crate::engine::rendering::{rendering_context::RenderingContext, swapchain::Swapchain};

pub struct Renderer {
    pub swapchain: Swapchain,
    pub context: Arc<RenderingContext>,
}

impl Renderer {
    pub fn new(context: Arc<RenderingContext>, window: Arc<Window>) -> Result<Self> {
        let mut swapchain = Swapchain::new(context.clone(), window.clone())?;

        swapchain.resize().unwrap();

        Ok(Self { swapchain, context })
    }

    pub fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
    }
}
