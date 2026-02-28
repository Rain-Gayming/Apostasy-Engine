use crate::{self as apostasy};
use apostasy_macros::{Component, InspectValue, Inspectable, SerializableComponent};
use serde::{Deserialize, Serialize};

use crate::engine::editor::inspectable::Inspectable;
#[derive(
    Component, Clone, Inspectable, InspectValue, Serialize, Deserialize, SerializableComponent,
)]
pub struct Player {
    pub is_active: bool,
}
impl Default for Player {
    fn default() -> Self {
        Self { is_active: true }
    }
}
