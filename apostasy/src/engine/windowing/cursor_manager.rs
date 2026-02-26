use crate as apostasy;
use crate::engine::windowing::WindowManager;
use crate::log;
use apostasy_macros::Component;
use winit::window::CursorGrabMode;

#[derive(Component, Clone)]
pub struct CursorManager {
    pub is_hidden: bool,
    pub is_grabbed: bool,
}

impl Default for CursorManager {
    fn default() -> Self {
        Self {
            is_hidden: false,
            is_grabbed: false,
        }
    }
}

impl CursorManager {
    pub fn grab_cursor(&mut self, window_manager: &mut WindowManager) {
        self.is_grabbed = true;
        self.is_hidden = true;

        window_manager.windows[&window_manager.primary_window_id].set_cursor_visible(false);
        let _ = window_manager.windows[&window_manager.primary_window_id]
            .set_cursor_grab(CursorGrabMode::Locked)
            .or_else(|_| {
                window_manager.windows[&window_manager.primary_window_id]
                    .set_cursor_grab(CursorGrabMode::Confined)
            });
    }

    pub fn ungrab_cursor(&mut self, window_manager: &mut WindowManager) {
        self.is_grabbed = false;
        self.is_hidden = false;

        window_manager.windows[&window_manager.primary_window_id].set_cursor_visible(true);
        let _ = window_manager.windows[&window_manager.primary_window_id]
            .set_cursor_grab(CursorGrabMode::None);
    }

    pub fn toggle_hide_cursor(&mut self) {
        self.is_hidden = !self.is_hidden;
    }
}
