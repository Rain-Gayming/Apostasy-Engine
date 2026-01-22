use crate::{self as apostasy, engine::windowing::WindowManager};
use apostasy_macros::Resource;
use winit::window::CursorGrabMode;

#[derive(Resource, Default)]
pub struct CursorManager {
    pub is_hidden: bool,
    pub is_grabbed: bool,
}

pub fn grab_cursor(cursor_manager: &mut CursorManager, window_manager: &mut WindowManager) {
    println!("grabbing cursor");
    cursor_manager.is_grabbed = true;
    cursor_manager.is_hidden = true;

    window_manager.windows[&window_manager.primary_window_id].set_cursor_visible(false);
    let _ = window_manager.windows[&window_manager.primary_window_id]
        .set_cursor_grab(CursorGrabMode::Locked)
        .or_else(|_| {
            window_manager.windows[&window_manager.primary_window_id]
                .set_cursor_grab(CursorGrabMode::Confined)
        });
}

pub fn ungrab_cursor(cursor_manager: &mut CursorManager, window_manager: &mut WindowManager) {
    cursor_manager.is_grabbed = false;
    cursor_manager.is_hidden = false;

    window_manager.windows[&window_manager.primary_window_id].set_cursor_visible(false);
    let _ = window_manager.windows[&window_manager.primary_window_id]
        .set_cursor_grab(CursorGrabMode::None);
}

pub fn toggle_hide_cursor(cursor_manager: &mut CursorManager) {
    cursor_manager.is_hidden = !cursor_manager.is_hidden;
}
