use apostasy_macros::Component;

use crate::engine::ecs::{
    entity::Entity,
    world::{UintID, World},
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ComponentVec(Vec<UintID>);

pub type ComponentEntry = fn(world: &World);

#[linkme::distributed_slice]
pub static COMPONENT_ENTRIES: [ComponentEntry];

pub unsafe trait Component: Sized {
    fn id() -> Entity;
    fn init(_: &World);
    fn info() -> ComponentInfo;
}

#[derive(Clone, Copy, Component, Debug)]
pub struct ComponentInfo {
    pub name: &'static str,
    pub align: usize,
    pub size: usize,
    pub id: Entity,
}
