use parking_lot::Mutex;

use crate::{
    engine::ecs::{
        entity::{Entity, EntityLocation},
        world::archetype::{Archetype, ArchetypeId, RowIndex},
    },
    utils::slotmap::SlotMap,
};

// Container for the ECS,
/// resources, compontents, archetypes ect
pub struct Core {
    pub archetypes: SlotMap<ArchetypeId, Archetype>,
    pub entity_index: Mutex<SlotMap<Entity, EntityLocation>>,
}

impl Core {
    pub fn new() -> Self {
        Core {
            archetypes: SlotMap::default(),
            entity_index: Mutex::new(SlotMap::default()),
        }
    }

    /// Create a blank entity
    pub fn create_uninitalized_entity_location(&self) -> Entity {
        let mut entity_index = self.entity_index.lock();
        entity_index.insert(EntityLocation::uninitalized())
    }

    /// Initalize an entities location to the empty archetype
    pub fn initalize_entity_location(&mut self, entity: Entity) -> EntityLocation {
        let entity_index = self.entity_index.get_mut();
        let mut location = entity_index[entity];
        if location == EntityLocation::uninitalized() {
            let empty_archetype = &mut self.archetypes[ArchetypeId::empty_archetype()];
            location.row = RowIndex(empty_archetype.entities.len());
            empty_archetype.entities.push(entity);
            entity_index[entity] = location;
        }
        location
    }
}
