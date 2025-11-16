use winit::window::Window;

use crate::app::engine::ecs::resource::{ResMut, Resource};

pub struct CursorManager {
    pub is_hidden: bool,
}
impl Resource for CursorManager {}

pub fn toggle_cursor_hidden(
    cursor_manager: &mut ResMut<CursorManager>,
    window: &Window,
    should_hide: bool,
) {
    if should_hide {
        cursor_manager.is_hidden = true;
        let _ = window.set_cursor_grab(winit::window::CursorGrabMode::Locked);
        window.set_cursor_visible(false);
    } else {
        cursor_manager.is_hidden = false;
        let _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
        window.set_cursor_visible(true);
    }
}
