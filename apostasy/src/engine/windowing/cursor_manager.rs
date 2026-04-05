use crate as apostasy;
use crate::engine::windowing::WindowManager;
use apostasy_macros::{Component, InspectValue, Inspectable};
use winit::window::CursorGrabMode;

use crate::engine::editor::inspectable::Inspectable;

#[derive(Clone, Copy, InspectValue, PartialEq, Eq, Default)]
pub enum CursorLockMode {
    #[default]
    NoneVisible,
    NoneHidden,
    ConfinedHidden,
    ConfinedVisible,
    LockedHidden,
    LockedVisible,
}

impl Inspectable for CursorLockMode {
    fn inspect(
        &mut self,
        _ui: &mut egui::Ui,
        _editor_storage: &mut crate::engine::editor::EditorStorage,
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
            CursorLockMode::NoneVisible => {
                window_manager.windows[&window_manager.primary_window_id].set_cursor_visible(true);
                window_manager.windows[&window_manager.primary_window_id]
                    .set_cursor_grab(CursorGrabMode::None);
            }
            CursorLockMode::NoneHidden => {
                window_manager.windows[&window_manager.primary_window_id].set_cursor_visible(false);
                window_manager.windows[&window_manager.primary_window_id]
                    .set_cursor_grab(CursorGrabMode::None);
            }

            CursorLockMode::ConfinedHidden => {
                window_manager.windows[&window_manager.primary_window_id].set_cursor_visible(false);
                window_manager.windows[&window_manager.primary_window_id]
                    .set_cursor_grab(CursorGrabMode::Confined);
            }

            CursorLockMode::ConfinedVisible => {
                window_manager.windows[&window_manager.primary_window_id].set_cursor_visible(true);
                window_manager.windows[&window_manager.primary_window_id]
                    .set_cursor_grab(CursorGrabMode::Confined);
            }

            CursorLockMode::LockedHidden => {
                window_manager.windows[&window_manager.primary_window_id].set_cursor_visible(false);
                window_manager.windows[&window_manager.primary_window_id]
                    .set_cursor_grab(CursorGrabMode::Locked);
            }

            CursorLockMode::LockedVisible => {
                window_manager.windows[&window_manager.primary_window_id].set_cursor_visible(true);
                window_manager.windows[&window_manager.primary_window_id]
                    .set_cursor_grab(CursorGrabMode::Locked);
            }
        }
    }

    /// If the current mode is unlocked, then lock it, otherwise unlock it
    pub fn switch_mode(&mut self) {
        match self.cursor_lock_mode {
            CursorLockMode::NoneVisible => self.cursor_lock_mode = CursorLockMode::LockedHidden,
            CursorLockMode::NoneHidden => self.cursor_lock_mode = CursorLockMode::LockedHidden,

            CursorLockMode::ConfinedHidden => self.cursor_lock_mode = CursorLockMode::NoneVisible,
            CursorLockMode::ConfinedVisible => self.cursor_lock_mode = CursorLockMode::NoneVisible,

            CursorLockMode::LockedHidden => self.cursor_lock_mode = CursorLockMode::NoneVisible,
            CursorLockMode::LockedVisible => self.cursor_lock_mode = CursorLockMode::NoneVisible,
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
