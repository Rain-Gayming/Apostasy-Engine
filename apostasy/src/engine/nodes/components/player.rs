use crate::{self as apostasy};
use apostasy_macros::{Component, InspectValue, Inspectable, SerializableComponent};
use serde::{Deserialize, Serialize};

use crate::engine::editor::inspectable::Inspectable;
#[derive(
    Component, Clone, Inspectable, InspectValue, Serialize, Deserialize, SerializableComponent,
)]
pub struct Player {
    pub is_active: bool,
    pub wish_dir: cgmath::Vector3<f32>,
    pub jump_pressed: bool,
    pub previous_jump_pressed: bool,
}
impl Default for Player {
    fn default() -> Self {
        Self { 
            is_active: true,
            wish_dir: cgmath::Vector3::new(0.0, 0.0, 0.0),
            jump_pressed: false,
            previous_jump_pressed: false,
        }
    }
}
