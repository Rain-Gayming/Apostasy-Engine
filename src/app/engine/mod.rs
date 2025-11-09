pub mod cursor_manager;
pub mod ecs;
pub mod input_manager;
pub mod renderer;
pub mod rendering_context;

use std::sync::{Arc, Mutex};

use crate::app::engine::{
    cursor_manager::{CursorManager, toggle_cursor_hidden},
    input_manager::{
        InputManager, is_keybind_name_triggered, process_keyboard_input, update_mouse_delta,
    },
    renderer::{Renderer, camera::Camera},
    rendering_context::*,
};
use anyhow::Result;
use winit::{
    event::{DeviceEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    window::Window,
};

pub struct Engine {
    pub renderer: Renderer,
    window: Arc<Window>,
    pub input_manager: InputManager,
    cursor_manager: CursorManager,
}

impl Engine {
    pub fn new(event_loop: &ActiveEventLoop, camera: Arc<Mutex<Camera>>) -> Result<Self> {
        let window = Arc::new(event_loop.create_window(Default::default())?);

        let rendering_context = Arc::new(RenderingContext::new(RenderingContextAttributes {
            compatability_window: &window,
            queue_family_picker: queue_family_picker::single_queue_family,
        })?);

        let renderer =
            Renderer::new(rendering_context.clone(), window.clone(), camera.clone()).unwrap();

        let input_manager = InputManager::default();

        let mut cursor_manager = CursorManager { is_hidden: false };
        toggle_cursor_hidden(&mut cursor_manager, &window, true);

        Ok(Self {
            renderer,
            window,
            input_manager,
            cursor_manager,
        })
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
                process_keyboard_input(&mut self.input_manager, &event);

                if is_keybind_name_triggered(&mut self.input_manager, "game_pause".to_string()) {
                    let is_hidden = self.cursor_manager.is_hidden;
                    toggle_cursor_hidden(&mut self.cursor_manager, &self.window, !is_hidden);
                }
            }
            _ => {}
        }
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn device_event(&mut self, event: winit::event::DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta, .. } = event {
            update_mouse_delta(&mut self.input_manager, delta.into());
        }
    }
}
