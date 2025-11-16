pub mod ecs;
pub mod renderer;
pub mod rendering_context;

use std::sync::Arc;

use crate::app::engine::{renderer::Renderer, rendering_context::*};
use anyhow::Result;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::Window};

pub struct Engine {
    pub renderer: Renderer,
    window: Arc<Window>,
}

impl Engine {
    pub fn new(event_loop: &ActiveEventLoop) -> Result<Self> {
        let window = Arc::new(event_loop.create_window(Default::default())?);

        let rendering_context = Arc::new(RenderingContext::new(RenderingContextAttributes {
            compatability_window: &window,
            queue_family_picker: queue_family_picker::single_queue_family,
        })?);

        let renderer = Renderer::new(rendering_context.clone(), window.clone()).unwrap();

        Ok(Self { renderer, window })
    }

    pub fn window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(_) => {
                self.renderer.resize().unwrap();
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                self.renderer.resize().unwrap();
            }
            WindowEvent::RedrawRequested => {
                self.renderer.render().unwrap();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // send input over to the game
            }
            _ => {}
        }
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn device_event(&mut self, event: winit::event::DeviceEvent) {
        // send this data over to the game
        // if let DeviceEvent::MouseMotion { delta, .. } = event {
        //     update_mouse_delta(&mut self.input_manager, delta.into());
        // }
    }
}
