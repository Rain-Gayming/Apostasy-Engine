pub mod cursor_manager;
pub mod input_manager;
pub mod renderer;
pub mod rendering_context;

use std::sync::{Arc, Mutex};

use crate::app::engine::{
    cursor_manager::{toggle_cursor_hidden, CursorManager},
    input_manager::{
        is_keybind_name_triggered, process_keyboard_input, update_mouse_delta, InputManager,
    },
    renderer::{
        camera::{handle_camera_input, update_camera, Camera},
        Renderer,
    },
    rendering_context::*,
};
use anyhow::Result;
use cgmath::Vector3;
use winit::{
    event::{DeviceEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};

pub struct Engine {
    pub renderer: Renderer,
    window: Arc<Window>,
    window_id: WindowId,
    rendering_context: Arc<RenderingContext>,
    input_manager: InputManager,
    engine_camera: Arc<Mutex<Camera>>,

    cursor_manager: CursorManager,
}

impl Engine {
    pub fn new(event_loop: &ActiveEventLoop) -> Result<Self> {
        let window = Arc::new(event_loop.create_window(Default::default())?);
        let window_id = window.id();

        let rendering_context = Arc::new(RenderingContext::new(RenderingContextAttributes {
            compatability_window: &window,
            queue_family_picker: queue_family_picker::single_queue_family,
        })?);

        let position: Vector3<f32> = Vector3::new(0.0, 1.0, 0.0);
        let engine_camera = Arc::new(Mutex::new(Camera::new(position)));

        let renderer = Renderer::new(
            rendering_context.clone(),
            window.clone(),
            engine_camera.clone(),
        )
        .unwrap();

        let input_manager = InputManager::default();

        let mut cursor_manager = CursorManager { is_hidden: false };
        toggle_cursor_hidden(&mut cursor_manager, &window, true);

        Ok(Self {
            renderer,
            window,
            window_id,
            rendering_context,
            input_manager,
            engine_camera,
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

                update_camera(self.engine_camera.clone());
            }
            WindowEvent::KeyboardInput { event, .. } => {
                process_keyboard_input(&mut self.input_manager, &event);
                handle_camera_input(&mut self.input_manager, &mut self.engine_camera);

                if is_keybind_name_triggered(&mut self.input_manager, "game_pause".to_string()) {
                    let is_hidden = self.cursor_manager.is_hidden;
                    toggle_cursor_hidden(&mut self.cursor_manager, &self.window, !is_hidden);
                }
            }
            _ => {}
        }
    }

    pub fn create_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        attributes: WindowAttributes,
    ) -> Result<WindowId> {
        let window = Arc::new(event_loop.create_window(attributes)?);
        let window_id = window.id();
        self.window = window.clone();
        self.window_id = window_id;

        let renderer = Renderer::new(
            self.rendering_context.clone(),
            window,
            self.engine_camera.clone(),
        )?;
        self.renderer = renderer;

        Ok(window_id)
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn device_event(&mut self, event: winit::event::DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta, .. } = event {
            update_mouse_delta(&mut self.input_manager, delta.into());
            handle_camera_input(&mut self.input_manager, &mut self.engine_camera)
        }
    }
}
