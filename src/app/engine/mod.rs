pub mod input_manager;
mod renderer;
mod rendering_context;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::app::engine::{
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
use nalgebra::Vector3;
use winit::{
    event::{DeviceEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};

pub struct Engine {
    renderers: HashMap<WindowId, Renderer>,
    windows: HashMap<WindowId, Arc<Window>>,
    primary_window_id: WindowId,
    rendering_context: Arc<RenderingContext>,
    input_manager: InputManager,
    engine_camera: Arc<Mutex<Camera>>,
}

impl Engine {
    pub fn new(event_loop: &ActiveEventLoop) -> Result<Self> {
        let primary_window = Arc::new(event_loop.create_window(Default::default())?);
        let primary_window_id = primary_window.id();

        let windows = HashMap::from([(primary_window_id, primary_window.clone())]);
        let rendering_context = Arc::new(RenderingContext::new(RenderingContextAttributes {
            compatability_window: &primary_window,
            queue_family_picker: queue_family_picker::single_queue_family,
        })?);

        let position: Vector3<f32> = Vector3::new(0.0, 1.0, 0.0);
        let engine_camera = Arc::new(Mutex::new(Camera::new(position)));
        let renderers = windows
            .iter()
            .map(|(id, window)| {
                let renderer = Renderer::new(
                    rendering_context.clone(),
                    window.clone(),
                    engine_camera.clone(),
                )
                .unwrap();
                (*id, renderer)
            })
            .collect::<HashMap<_, _>>();

        let input_manager = InputManager::new();

        Ok(Self {
            renderers,
            windows,
            primary_window_id,
            rendering_context,
            input_manager,
            engine_camera,
        })
    }

    pub fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                if window_id == self.primary_window_id {
                    event_loop.exit();
                    return;
                }

                self.renderers.remove(&window_id);
                self.windows.remove(&window_id);
            }
            WindowEvent::Resized(_) => {
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
                if let Some(renderer) = self.renderers.get_mut(&window_id) {
                    renderer.render().unwrap();
                }

                update_camera(self.engine_camera.clone());
            }
            WindowEvent::KeyboardInput { event, .. } => {
                process_keyboard_input(&mut self.input_manager, &event);
                handle_camera_input(&self.input_manager, &mut self.engine_camera)
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
        self.windows.insert(window_id, window.clone());

        let renderer = Renderer::new(
            self.rendering_context.clone(),
            window,
            self.engine_camera.clone(),
        )?;
        self.renderers.insert(window_id, renderer);

        Ok(window_id)
    }

    pub fn request_redraw(&self, event_loop: &ActiveEventLoop) {
        for window in self.windows.values() {
            window.request_redraw();
        }
    }

    pub fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta, .. } = event {
            update_mouse_delta(&mut self.input_manager, delta.into());
            handle_camera_input(&self.input_manager, &mut self.engine_camera)
            // store mouse movement in somewhere lol
        }
    }
}
