use std::{any::Any, collections::HashMap};

use crate::engine::ecs::{World, entity::Entity};

pub type ResourceEntry = fn(&mut World);

#[linkme::distributed_slice]
pub static RESOURCE_ENTRIES: [ResourceEntry];

/// A resource map, used to store resources and their data
#[derive(Default)]
pub struct ResourceMap {
    storage: HashMap<Entity, Box<dyn Any>>,
}

impl ResourceMap {
    pub fn get<T: Resource>(&self) -> Option<&T> {
        self.storage
            .get(&T::id())
            .and_then(|r| r.downcast_ref::<T>())
    }

    pub fn get_mut<T: Resource>(&mut self) -> Option<&mut T> {
        self.storage
            .get_mut(&T::id())
            .and_then(|r| r.downcast_mut::<T>())
    }

    pub fn insert<T: Resource>(&mut self, resource: T) {
        self.storage.insert(T::id(), Box::new(resource));
    }
}

pub unsafe trait Resource: Sized + 'static {
    fn init(world: &mut World);
    fn id() -> Entity;
    fn name() -> &'static str;
}
