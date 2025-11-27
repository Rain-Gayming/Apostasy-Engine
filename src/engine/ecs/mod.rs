use std::{collections::HashMap, intrinsics::type_id};

use crate::engine::ecs::{
    archetype::{Archetype, ArchetypeId, ArchetypeMap, ArchetypeRecord, Record},
    component::{Component, ComponentId, ComponentType},
};
pub mod archetype;
pub mod component;

/// The id for an entity
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct EntityId(u64);

/// The ECS world
pub struct World {
    entity_index: HashMap<EntityId, Record>,
    archetype_map: HashMap<ArchetypeId, ArchetypeRecord>,
    archetype_index: HashMap<ComponentType, Archetype>,
    component_index: HashMap<ComponentId, ArchetypeMap>,
}

impl Default for World {
    fn default() -> Self {
        let empty_archetype_index = ComponentType(Vec::new());
        let empty_archetype = Archetype {
            id: ArchetypeId(0),
            component_type: empty_archetype_index.clone(),
            components: vec![],
            edges: HashMap::new(),
        };
        let mut archetype_map = HashMap::new();
        archetype_map.insert(ArchetypeId(0), ArchetypeRecord { column: 0 });

        let mut archetype_index = HashMap::new();
        archetype_index.insert(empty_archetype_index, empty_archetype);

        World {
            entity_index: HashMap::new(),
            archetype_map,
            archetype_index,
            component_index: HashMap::new(),
        }
    }
}

impl World {
    /// Adds new entity to the world
    pub fn new_entity(&mut self) -> &mut Self {
        // get next entity id
        let entity_id: EntityId = EntityId((self.entity_index.len() - 1) as u64);
        // get empty archetype
        let empty_archetype = self
            .archetype_index
            .get(&ComponentType(Vec::new()))
            .unwrap();

        // create a new record for the entity
        let record = Record {
            archetype: empty_archetype.id,
            row: entity_id.0 as usize,
        };

        // add the entity to the index
        self.entity_index.insert(entity_id, record);

        self
    }

    pub fn with_component<T: Component>(&mut self, data: Box<T>) -> &mut Self {
        // get the type id of T
        self
    }
}
