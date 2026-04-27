use std::any::TypeId;

use anyhow::{Error, Result};

use crate::{
    log_warn,
    objects::{
        component::{Component, get_component_registration},
        scene::ObjectId,
        tag::Tag,
    },
};

pub mod component;
pub mod components;
pub mod query;
pub mod resource;
pub mod resources;
pub mod scene;
pub mod systems;
pub mod tag;
pub mod tags;
pub mod world;

use crate::objects::component::BoxedComponent;

#[derive(Clone)]
pub struct Object {
    pub id: ObjectId,
    pub name: String,
    pub components: Vec<BoxedComponent>,
    pub tags: Vec<Box<dyn Tag>>,
    pub parent: Option<ObjectId>,
    pub children: Vec<ObjectId>,
}
impl Default for Object {
    fn default() -> Self {
        Object::new()
    }
}

impl Object {
    pub fn new() -> Self {
        Self {
            name: "Object".to_string(),
            id: ObjectId::default(),
            children: Vec::new(),
            tags: Vec::new(),
            parent: None,
            components: Vec::new(),
        }
    }
    /// Adds a child to the node
    pub fn add_child(&mut self, mut child: Object) -> &mut Self {
        child.parent = Some(self.id.clone());
        self.children.push(child.id);
        self
    }

    pub fn set_name(&mut self, name: String) -> Self {
        self.name = name;

        self.clone()
    }

    // ========== ========== Tags ========== ==========

    /// Checks if the node has a tag of type T
    pub fn has_tag<T: Tag + 'static>(&self) -> bool {
        self.tags
            .iter()
            .any(|tag| tag.as_any().downcast_ref::<T>().is_some())
    }

    /// Gets a tag of type T from the node
    pub(crate) fn get_tag<T: Tag + 'static>(&self) -> Result<&T> {
        self.tags
            .iter()
            .find(|c| c.as_any().type_id() == TypeId::of::<T>())
            .and_then(|c| c.as_any().downcast_ref())
            .ok_or(Error::msg("No Component of type"))
    }

    /// Adds a tag of type T to the node
    pub fn add_tag<T: Tag + 'static>(&mut self, tag: T) -> Self {
        if self.get_tag::<T>().is_ok() {
            log_warn!("You can only have one of any tag on an entity");
            return self.clone();
        } else {
            self.tags.push(Box::new(tag));
            return self.clone();
        }
    }

    /// Gets a tag of type T from the node
    pub(crate) fn remove_tag<T: Tag + 'static>(&mut self) {
        let index = self
            .tags
            .iter()
            .position(|c| c.as_any().type_id() == TypeId::of::<T>());

        if let Some(i) = index {
            self.tags.remove(i);
        }
    }

    // ========== ========== Components ========== ==========

    /// Checks if the node has a component of type T
    pub fn has_component<T: Component + 'static>(&self) -> bool {
        self.components
            .iter()
            .any(|component| component.as_any().downcast_ref::<T>().is_some())
    }
    pub fn remove_component<T: Component + 'static>(&mut self) {
        let index = self
            .components
            .iter()
            .position(|c| c.as_any().type_id() == TypeId::of::<T>());

        if let Some(i) = index {
            self.components.remove(i);
        }
    }
    /// Gets a component of type T from the node
    pub fn get_component<T: Component + 'static>(&self) -> Result<&T> {
        let msg = format!("No Component of type: {}", T::name());
        self.components
            .iter()
            .find(|c| c.as_any().type_id() == TypeId::of::<T>())
            .and_then(|c| c.as_any().downcast_ref())
            .ok_or(Error::msg(msg))
    }

    pub fn get_component_mut<T: Component + 'static>(&mut self) -> Result<&mut T> {
        let msg = format!("No Component of type: {}", T::name());
        self.components
            .iter_mut()
            .find(|c| c.as_any().type_id() == TypeId::of::<T>())
            .and_then(|c| c.as_any_mut().downcast_mut())
            .ok_or(Error::msg(msg))
    }

    /// Adds a component of type T to the node
    pub fn add_component<T: Component + 'static>(&mut self, component: T) -> Self {
        if self.get_component::<T>().is_ok() {
            log_warn!("You can only have one of any component on an entity");
            return self.clone();
        } else {
            self.components.push(Box::new(component));
            return self.clone();
        }
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
