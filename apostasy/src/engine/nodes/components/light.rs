use crate::{self as apostasy};
use apostasy::engine::editor::inspectable::Inspectable;
use apostasy_macros::{Component, InspectValue, Inspectable, SerializableComponent};
use serde::{Deserialize, Serialize};

#[derive(
    Component,
    Clone,
    Serialize,
    Deserialize,
    SerializableComponent,
    Inspectable,
    InspectValue,
    Default,
)]
pub struct Light {
    pub strenght: f32,
}
