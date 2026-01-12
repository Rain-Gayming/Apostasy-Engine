use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use winit::{
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{self, Window, WindowAttributes, WindowId},
};

use crate::engine::rendering::renderer::Renderer;

pub struct RenderEngine {
    pub windows: HashMap<WindowId, Arc<Window>>,
    pub renderers: HashMap<WindowId, Renderer>,
    pub primary_window_id: WindowId,
}

impl RenderEngine {
    pub fn new(event_loop: &ActiveEventLoop) -> Self {
        let primary_window = Arc::new(event_loop.create_window(Default::default()).unwrap());
        let primary_window_id = primary_window.id();
        let windows = HashMap::from([(primary_window_id, primary_window.clone())]);

        let renderers = windows
            .iter()
            .map(|(id, window)| {
                let renderer = Renderer::new(window.clone());
                (*id, renderer)
            })
            .collect::<HashMap<WindowId, Renderer>>();

        Self {
            windows,
            renderers,
            primary_window_id,
        }
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
            _ => (),
        }
    }

    pub fn create_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        attributes: WindowAttributes,
    ) -> Result<WindowId> {
        let window = Arc::new(event_loop.create_window(attributes)?);
        let window_id = window.id();

        self.windows.insert(window_id, window);

        let renderer = Renderer::new(self.windows.get(&window_id).unwrap().clone());
        self.renderers.insert(window_id, renderer);
        Ok(window_id)
    }
}
