use std::{collections::HashMap, sync::Arc};

use winit::{
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
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
}
