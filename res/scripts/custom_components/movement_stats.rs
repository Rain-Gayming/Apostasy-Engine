use apostasy::engine::nodes::component::Component;
use apostasy_macros::{Component, InspectValue, Inspectable, SerializableComponent};
use serde::{Deserialize, Serialize};

#[derive(
    Default,
    Component,
    Clone,
    Deserialize,
    Serialize,
    SerializableComponent,
    InspectValue,
    Inspectable,
)]
pub struct MovementStats {
    pub current_speed: f32,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub sprint_speed: f32,
    pub jump_speed: f32,
}
