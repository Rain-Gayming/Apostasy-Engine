use std::{any, collections::HashMap};

use crate::engine::ecs::{
    ComponentType,
    component::{Component, ComponentId},
};

/// The id for an archetype
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct ArchetypeId(pub u64);

/// The actual collection of component data for one type of component held in an archetype
#[derive(Clone)]
pub struct Column(pub Vec<Box<dyn Component>>);

/// The edges of an archetype that contain atleast one same component
#[derive(Clone)]
pub struct ArchetypeEdge {
    pub add: Archetype,
    pub remove: Archetype,
}

/// An archetype, contains a set of entities with specific components and only those components
#[derive(Clone)]
pub struct Archetype {
    pub id: ArchetypeId,
    pub component_type: ComponentType,
    pub components: Vec<Column>,
    pub edges: HashMap<ComponentId, ArchetypeEdge>,
}

/// A record in component index with the component column for an archetype
#[derive(Clone)]
pub struct ArchetypeRecord {
    pub column: usize,
}

/// A record in the entity index with the archetype and index of an entity
#[derive(Clone)]
pub struct Record {
    pub archetype: ArchetypeId,
    pub row: usize,
}

#[derive(Clone)]
pub struct ArchetypeMap(HashMap<ArchetypeId, ArchetypeRecord>);
