use crate::{self as apostasy};
use apostasy_macros::{Component, Inspectable, SerializableComponent};
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Inspectable, Serialize, Deserialize, SerializableComponent)]
pub struct Player {
    pub is_active: bool,
}
impl Default for Player {
    fn default() -> Self {
        Self { is_active: true }
    }
}
