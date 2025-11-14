use std::{any::TypeId, fmt::Debug};

use crate::app::engine::ecs::{
    ECSWorld, archetype::Archetype, component::Component, entities::Entity,
};

/// A struct that is used to get components from the world
pub struct Query<'a> {
    required_components: Vec<TypeId>,
    world: &'a ECSWorld,
}

impl<'a> Debug for Query<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("query")
            .field("components", &self.required_components)
            .finish()
    }
}

impl<'a> Query<'a> {
    /// Create a new query
    pub fn new(world: &'a ECSWorld) -> Self {
        Query {
            required_components: Vec::new(),
            world,
        }
    }

    /// Adds a component to the query
    pub fn with<T: Component>(mut self) -> Self {
        self.required_components.push(TypeId::of::<T>());
        self
    }

    /// Searches all archetypes for the components
    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.world
            .archetypes
            .iter()
            .filter(|archetype| self.matches(archetype))
            .flat_map(|archetype| archetype.entities.iter().copied())
    }
    /// Checks if the archetype has the components asked for
    fn matches(&self, archetype: &Archetype) -> bool {
        self.required_components
            .iter()
            .all(|required| archetype.component_types.contains(required))
    }
}
