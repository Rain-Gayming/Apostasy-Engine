use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::app::engine::ecs::component::Component;

/// A entity, used to hold components and an ID for the entity itself
pub struct Entity {
    pub components: HashMap<TypeId, Box<dyn Component>>,
    pub id: u32,
}

impl Entity {
    /// Adds a component to an entity
    ///
    /// ```
    /// fn create_entity() {
    ///     let mut world = ECSWorld::default();
    ///
    ///     let new_entity = world
    ///         .create_entity()
    ///         .add_component::<NewComponent>(NewComponent(59.0))
    ///         .add_component::<NewComponentB>(NewComponentB(590.0));
    /// }
    /// ```
    pub fn add_component<T: Component>(&mut self, data: impl Any + Component) -> &mut Self {
        let type_id = TypeId::of::<T>();

        self.components.insert(type_id, Box::new(data));
        self
    }

    /// Takes in a component type and gets a non-mutable reference to it from the entity
    ///
    /// ```
    /// fn get_component_reference() {
    ///     let mut world = ECSWorld::default();
    ///
    ///     let new_entity = world
    ///         .create_entity()
    ///         .add_component::<NewComponent>(NewComponent(59.0));
    ///     let new_component = new_entity.get_component_ref::<NewComponent>().unwrap();
    ///
    ///     assert_eq!(new_component.0, 59.0);
    /// }
    /// ```
    pub fn get_component_ref<T: Component>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        if let Some(data) = self.components.get(&type_id) {
            data.downcast_ref()
        } else {
            None
        }
    }

    /// Takes in a component type and gets a mutable reference to it from the entity
    ///
    /// ```
    /// fn get_component_mutalby() {
    ///     let mut world = ECSWorld::default();
    ///
    ///     let new_entity = world
    ///         .create_entity()
    ///         .add_component::<NewComponent>(NewComponent(59.0));
    ///     let new_component = new_entity.get_component_mut::<NewComponent>().unwrap();
    ///
    ///     new_component.0 += 10.0;
    ///
    ///     assert_eq!(new_component.0, 69.0);
    /// }
    /// ```
    pub fn get_component_mut<T: Component>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        if let Some(data) = self.components.get_mut(&type_id) {
            data.downcast_mut()
        } else {
            None
        }
    }
}
