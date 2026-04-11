use std::sync::{Arc, Mutex};

use anyhow::Result;
use winit::{
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

use crate::rendering::{RenderingBackend, RenderingInfo};
use winit::application::ApplicationHandler;

pub mod assets;
pub mod rendering;
pub mod utils;

pub struct Core {
    pub rendering_api: RenderingBackend,
    pub rendering_info: Option<Arc<Mutex<RenderingInfo>>>,
}

impl Core {
    pub fn new(rendering_api: RenderingBackend) -> Self {
        Self {
            rendering_api,
            rendering_info: None,
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
                    if let Some(renderer) = &mut rendering_info.renderer {
                        renderer.render().unwrap();
                    }
                }
                _ => {}
            }
        }
    }
}
impl ApplicationHandler for Core {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.rendering_info = Some(RenderingInfo::new(&event_loop, self.rendering_api));
    }

    fn suspended(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        self.window_event(event_loop, window_id, event);
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        // if let Some(engine) = &mut self.engine {
        //     engine.request_redraw(event_loop);
        // }
    }
}

pub fn init_core(rendering_api: RenderingBackend) -> Result<()> {
    let mut core = Core::new(rendering_api);

    let event_loop = EventLoop::new()?;
    event_loop.run_app(&mut core)?;

    Ok(())
}
