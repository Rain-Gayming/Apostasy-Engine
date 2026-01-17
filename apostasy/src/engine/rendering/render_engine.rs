use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use winit::{
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};

use crate::engine::{
    ecs::World,
    rendering::{
        queue_families::queue_family_picker::single_queue_family,
        renderer::{Renderer, render},
        rendering_context::{RenderingContext, RenderingContextAttributes},
    },
};

pub struct RenderEngine {
    pub renderers: HashMap<WindowId, Renderer>,
    pub windows: HashMap<WindowId, Arc<Window>>,
    pub primary_window_id: WindowId,
    pub rendering_context: Arc<RenderingContext>,
    pub world: World,
}

impl RenderEngine {
    pub fn new(event_loop: &ActiveEventLoop, world: World) -> Result<Self> {
        let primary_window = Arc::new(
            event_loop.create_window(
                Window::default_attributes()
                    .with_title("Apostasy")
                    .with_visible(true),
            )?,
        );
        let primary_window_id = primary_window.id();
        let windows = HashMap::from([(primary_window_id, primary_window.clone())]);

        let rendering_context = Arc::new(RenderingContext::new(RenderingContextAttributes {
            compatability_window: &primary_window,
            queue_family_picker: single_queue_family,
        })?);

        let renderers = windows
            .iter()
            .map(|(id, window)| {
                let renderer = Renderer::new(rendering_context.clone(), window.clone()).unwrap();
                (*id, renderer)
            })
            .collect::<HashMap<WindowId, Renderer>>();

        Ok(Self {
            renderers,
            windows,
            primary_window_id,
            rendering_context,
            world,
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
                } else {
                    self.windows.remove(&window_id);
                    self.renderers.remove(&window_id);
                }
            }
            WindowEvent::Resized(_size) => {
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
                    let _ = render(renderer, &self.world);
                }
            }

            _ => (),
        }
    }

    pub fn request_redraw(&self) {
        for window in self.windows.values() {
            window.request_redraw();
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

        let renderer = Renderer::new(self.rendering_context.clone(), window)?;
        self.renderers.insert(window_id, renderer);
        Ok(window_id)
    }
}
