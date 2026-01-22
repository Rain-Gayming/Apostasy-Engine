use std::{collections::HashMap, sync::Arc};

use crate as apostasy;
use apostasy_macros::Resource;
use winit::window::{Window, WindowId};

pub mod cursor_manager;
#[derive(Resource)]
pub struct WindowManager {
    pub windows: HashMap<WindowId, Arc<Window>>,
    pub primary_window_id: WindowId,
}

impl Default for WindowManager {
    fn default() -> Self {
        Self {
            windows: HashMap::new(),
            primary_window_id: WindowId::dummy(),
        }
    }
}
