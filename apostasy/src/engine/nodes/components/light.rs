use crate::{self as apostasy};
use apostasy::engine::editor::inspectable::Inspectable;
use apostasy_macros::{Component, InspectValue, Inspectable, SerializableComponent};
use serde::{Deserialize, Serialize};

#[derive(
    Component, Clone, Serialize, Deserialize, SerializableComponent, Inspectable, InspectValue,
)]
pub struct Light {
    pub strength: f32,
}

impl Default for Light {
    fn default() -> Self {
        Self { strength: 1.0 }
    }
}
