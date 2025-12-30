use derive_more::From;

use crate::{
    engine::ecs::{
        component::Component,
        world::{
            World,
            archetype::{ArchetypeId, RowIndex},
            commands::Command,
        },
    },
    utils::slotmap::Key,
};

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub struct EntityLocation {
    pub archetype: ArchetypeId,
    pub row: RowIndex,
}

impl EntityLocation {
    pub fn uninitalized() -> Self {
        Self {
            archetype: ArchetypeId::empty_archetype(),
            row: RowIndex(usize::MAX),
        }
    }
}

#[derive(Debug, Clone, Copy, From, PartialEq, Eq)]
pub struct Entity(pub Key);

impl From<Entity> for Key {
    fn from(value: Entity) -> Self {
        value.0
    }
}

pub struct EntityView<'a> {
    pub entity: Entity,
    pub world: &'a World,
}

impl EntityView<'_> {
    /// Insert a component into the entity
    pub fn insert<C: Component>(self, component: C) -> Self {
        self.world
            .crust
            .mantle(|mantle| mantle.enqueue(Command::insert(component, self.entity)));
        self
    }
}
