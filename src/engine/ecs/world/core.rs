use std::collections::HashMap;

use derive_more::{Deref, DerefMut};
use parking_lot::{Mutex, RwLock};

use crate::{
    engine::ecs::{
        component::{Component, ComponentInfo},
        entity::Entity,
        world::archetype::{
            Archetype, ArchetypeId, Column, ColumnIndex, FieldId, RowIndex, Signature,
        },
    },
    utils::slotmap::SlotMap,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EntityLocation {
    pub archetype: ArchetypeId,
    pub row: RowIndex,
}

impl EntityLocation {
    pub fn uninitialized() -> Self {
        Self {
            archetype: ArchetypeId::empty_archetype(),
            row: RowIndex(usize::MAX),
        }
    }
}

#[derive(Deref, DerefMut, Default, Debug)]
pub struct FieldLocations(HashMap<ArchetypeId, ColumnIndex>);

pub struct Core {
    pub entity_index: Mutex<SlotMap<Entity, EntityLocation>>,
    // field_index: HashMap<FieldId, FieldLocations>,
    pub archetypes: SlotMap<ArchetypeId, Archetype>,
    // signature index
    // archetypes
}

impl Core {
    pub fn new() -> Self {
        // Add empty archetype & component info archetype
        let mut archetypes = SlotMap::<ArchetypeId, Archetype>::default();
        let mut entity_index = SlotMap::<Entity, EntityLocation>::default();
        let empty_archetype_id = archetypes.insert(Archetype::default());
        let component_info_archetype_id = archetypes.insert(Archetype::default());
        assert_eq!(empty_archetype_id, ArchetypeId::empty_archetype());

        if let Some(empty_archetype) = archetypes.get_mut(empty_archetype_id) {
            // Make sure all component entities are sawned before init
            // Needed if components add relationships (traits)
            // for n in 0..COMPONENT_ENTRIES.len() {
            //     let id = entity_index.insert(EntityLocation {
            //         archetype: empty_archetype_id,
            //         row: RowIndex(n),
            //     });
            //     empty_archetype.entities.push(id);
            // }
            // // Add ComponentInfo edge
            // let component_info_edge = &mut empty_archetype //
            //     .edges
            //     .entry(ComponentInfo::id().into())
            //     .or_default();
            // component_info_edge.add = Some(component_info_archetype_id);
        }

        // Ccreate ComponentInfo archetype
        let component_info_signature = Signature::new(&[ComponentInfo::id().into()]);
        archetypes[component_info_archetype_id] = Archetype {
            // signature: component_info_signature.clone(),
            entities: Default::default(),
            columns: vec![RwLock::new(Column::new(ComponentInfo::info()))],
            // edges: HashMap::from([(
            //     ComponentInfo::id().into(),
            //     ArchetypeEdge {
            //         remove: Some(empty_archetype_id),
            //         add: None,
            //     },
            // )]),
        };

        Self {
            archetypes,
            entity_index: Mutex::new(entity_index),
            // field_index: HashMap::from([(
            //     ComponentInfo::id().into(),
            //     FieldLocations(HashMap::from([(
            //         component_info_archetype_id,
            //         ColumnIndex(0),
            //     )])),
            // )]),
            // signature_index: HashMap::from([
            //     (Signature::default(), empty_archetype_id),
            //     (component_info_signature, component_info_archetype_id),
            // ]),
        }
    }

    pub fn create_uninitialized_entity(&self) -> Entity {
        let mut entity_index = self.entity_index.lock();
        entity_index.insert(EntityLocation::uninitialized())
    }
    pub fn initialize_entity_location(&mut self, entity: Entity) -> EntityLocation {
        let entity_index = self.entity_index.get_mut();
        let mut location = entity_index[entity];
        if location == EntityLocation::uninitialized() {
            let empty_archetype = &mut self.archetypes[ArchetypeId::empty_archetype()];
            location.row = RowIndex(empty_archetype.entities.len());
            empty_archetype.entities.push(entity);
            entity_index[entity] = location;
        }
        location
    }
}
