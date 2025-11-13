// Query needs to take in components to find and excluse
// use type id's of the inputs to determine if theyre in the archetype?

use std::any::TypeId;

use crate::app::engine::ecs::{
    ECSWorld, archetype::Archetype, component::Component, entities::Entity,
};

/// A struct that is used to get components from the world
pub struct Query<'a> {
    required_components: Vec<TypeId>,
    world: &'a ECSWorld,
}

impl<'a> Query<'a> {
    pub fn new(world: &'a ECSWorld) -> Self {
        Query {
            required_components: Vec::new(),
            world,
        }
    }

    pub fn with<T: Component>(mut self) -> Self {
        self.required_components.push(TypeId::of::<T>());
        self
    }

    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.world
            .archetypes
            .iter()
            .filter(|archetype| self.matches(archetype))
            .flat_map(|archetype| archetype.entities.iter().copied())
    }

    fn matches(&self, archetype: &Archetype) -> bool {
        self.required_components
            .iter()
            .all(|required| archetype.component_types.contains(required))
    }
}
