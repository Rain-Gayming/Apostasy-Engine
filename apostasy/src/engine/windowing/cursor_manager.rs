use crate as apostasy;
use crate::engine::windowing::WindowManager;
use apostasy_macros::{Component, InspectValue, Inspectable};
use winit::window::CursorGrabMode;

use crate::engine::editor::inspectable::Inspectable;

#[derive(Clone, Copy, InspectValue, PartialEq, Eq, Default)]
pub enum CursorLockMode {
    #[default]
    UngrabbedVisible,
    UngrabbedHidden,
    GrabbedHidden,
    GrabbedVisible,
}

impl Inspectable for CursorLockMode {
    fn inspect(
        &mut self,
        ui: &mut egui::Ui,
        editor_storage: &mut crate::engine::editor::EditorStorage,
    ) -> bool {
        false
    }
}

#[derive(Component, Clone, Default, Inspectable, InspectValue)]
pub struct CursorManager {
    pub cursor_lock_mode: CursorLockMode,
}

#[allow(unused_must_use)]
impl CursorManager {
    pub fn update_cursor(&mut self, window_manager: &mut WindowManager) {
        match self.cursor_lock_mode {
            CursorLockMode::UngrabbedVisible => {
                window_manager.windows[&window_manager.primary_window_id].set_cursor_visible(true);
                window_manager.windows[&window_manager.primary_window_id]
                    .set_cursor_grab(CursorGrabMode::None);
            }
            CursorLockMode::UngrabbedHidden => {
                window_manager.windows[&window_manager.primary_window_id].set_cursor_visible(false);
                window_manager.windows[&window_manager.primary_window_id]
                    .set_cursor_grab(CursorGrabMode::None);
            }

            CursorLockMode::GrabbedHidden => {
                window_manager.windows[&window_manager.primary_window_id].set_cursor_visible(false);
                window_manager.windows[&window_manager.primary_window_id]
                    .set_cursor_grab(CursorGrabMode::Confined);
            }

            CursorLockMode::GrabbedVisible => {
                window_manager.windows[&window_manager.primary_window_id].set_cursor_visible(true);
                window_manager.windows[&window_manager.primary_window_id]
                    .set_cursor_grab(CursorGrabMode::Locked);
            }
        }
    }

    pub fn grab_cursor(&mut self, window_manager: &mut WindowManager) {
        window_manager.windows[&window_manager.primary_window_id].set_cursor_visible(false);
        let _ = window_manager.windows[&window_manager.primary_window_id]
            .set_cursor_grab(CursorGrabMode::Locked)
            .or_else(|_| {
                window_manager.windows[&window_manager.primary_window_id]
                    .set_cursor_grab(CursorGrabMode::Locked)
            });
    }
}
