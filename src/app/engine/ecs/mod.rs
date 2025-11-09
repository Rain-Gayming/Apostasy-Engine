use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::app::engine::ecs::{component::Component, entities::Entity, resources::Resource};

pub mod component;
pub mod entities;
pub mod resources;

#[derive(Default)]
pub struct ECSWorld {
    pub resources: HashMap<TypeId, Box<dyn Any>>,
    pub entities: HashMap<u32, Entity>,
}

impl ECSWorld {
    /// Adds a resource to the resource pool
    pub fn add_resource(&mut self, resource_data: impl Any) {
        let type_id = resource_data.type_id();
        self.resources.insert(type_id, Box::new(resource_data));
    }

    /// Query for a resource and get a non-mutable reference to it as an option
    /// allows for nothing to return but will cause a panic
    pub fn get_resource_ref<T: Any>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        if let Some(data) = self.resources.get(&type_id) {
            data.downcast_ref()
        } else {
            None
        }
    }

    /// Query for a resource and get a mutable reference to it as an option
    /// allows for nothing to return but will cause a panic
    pub fn get_resource_mut<T: Any>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        if let Some(data) = self.resources.get_mut(&type_id) {
            data.downcast_mut()
        } else {
            None
        }
    }

    /// Removes a resource from the pool
    pub fn remove_resource<T: Any>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.resources.remove(&type_id);
    }

    /// Adds a blank entity to the entities pool,
    /// to add components to it use
    /// ```
    /// fn foo(){
    ///     let world = World::default();
    ///     world.
    ///     create_entity().with_component(xxx)
    /// }
    /// ```
    pub fn create_entity(&mut self) -> &mut Self {
        let entity_id: u32 = self.entities.len() as u32;

        self.entities.insert(
            entity_id,
            Entity {
                components: vec![],
                id: entity_id,
            },
        );

        self
    }
}
