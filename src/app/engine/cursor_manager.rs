use winit::window::Window;

pub struct CursorManager {
    pub is_hidden: bool,
}

pub fn toggle_cursor_hidden(
    cursor_manager: &mut CursorManager,
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
