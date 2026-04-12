use std::any::TypeId;

use anyhow::Result;

use crate::{
    log_warn,
    objects::component::{Component, get_component_registration},
};

pub mod component;
pub mod components;
pub mod systems;
pub mod world;

pub struct Object {
    pub id: u64,
    pub name: String,
    pub components: Vec<Box<dyn Component>>,
    pub parent: Option<u64>,
    pub children: Vec<u64>,
}

impl Default for Object {
    fn default() -> Self {
        Object::new()
    }
}

impl Object {
    pub fn new() -> Self {
        let mut components: Vec<Box<dyn Component>> = Vec::new();

        Self {
            name: "Object".to_string(),
            id: 0,
            children: Vec::new(),
            parent: None,
            components,
        }
    }
    /// Checks if the node has a component of type T
    pub fn has_component<T: Component + 'static>(&self) -> bool {
        self.components
            .iter()
            .any(|component| component.as_any().downcast_ref::<T>().is_some())
    }

    /// Gets a component of type T from the node
    pub fn get_component<T: Component + 'static>(&self) -> Option<&T> {
        self.components
            .iter()
            .find(|c| c.as_any().type_id() == TypeId::of::<T>())
            .and_then(|c| c.as_any().downcast_ref())
    }

    pub fn get_component_mut<T: Component + 'static>(&mut self) -> Option<&mut T> {
        self.components
            .iter_mut()
            .find(|c| c.as_any().type_id() == TypeId::of::<T>())
            .and_then(|c| c.as_any_mut().downcast_mut())
    }

    /// Adds a component of type T to the node
    pub fn add_component<T: Component + 'static>(&mut self, component: T) -> &mut Self {
        if self.get_component::<T>().is_some() {
            log_warn!("You can only have one of any component on an entity");
            return self;
        } else {
            self.components.push(Box::new(component));
            return self;
        }
    }

    /// Adds a child to the node
    pub fn add_child(&mut self, mut child: Object) -> &mut Self {
        child.parent = Some(self.id.clone());
        self.children.push(child.id);
        self
    }

    /// Adds a component of type T to the node
    /// Note: capitalization is ignored
    pub fn add_component_by_name(&mut self, component_name: &str) -> Result<()> {
        let mut component_name = component_name.to_string();
        component_name = component_name.replace(" ", "");
        component_name = component_name.replace("_", "");

        let registration = get_component_registration(component_name.to_lowercase().as_str())
            .ok_or_else(|| {
                log_warn!("Component '{}' is not registered", component_name);
                anyhow::anyhow!(
                    "Component '{}' is not registered",
                    component_name.to_lowercase()
                )
            })?;

        // Check for duplicate using type_name since we don't have T here
        let component = (registration.create)();
        let new_type_name = component.type_name();

        if self
            .components
            .iter()
            .any(|c| c.type_name() == new_type_name)
        {
            log_warn!("You can only have one of any component on an entity");
            return Ok(());
        }

        self.components.push(component);
        Ok(())
    }
}
