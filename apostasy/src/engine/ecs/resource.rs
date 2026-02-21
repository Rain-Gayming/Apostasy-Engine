use crate::engine::ecs::{World, entity::Entity};

pub type ResourceEntry = fn(&mut World);

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};
#[linkme::distributed_slice]
pub static RESOURCE_ENTRIES: [ResourceEntry];

/// A resource map, used to store resources and their data

#[derive(Default)]
pub struct ResourceMap {
    storage: HashMap<TypeId, Box<dyn Any>>,
}

impl ResourceMap {
    pub fn get<T: Resource>(&self) -> Option<&T> {
        self.storage
            .get(&TypeId::of::<T>())
            .and_then(|r| r.downcast_ref::<T>())
    }

    pub fn get_mut<T: Resource>(&mut self) -> Option<&mut T> {
        self.storage
            .get_mut(&TypeId::of::<T>())
            .and_then(|r| r.downcast_mut::<T>())
    }

    pub fn insert<T: Resource>(&mut self, resource: T) {
        self.storage.insert(TypeId::of::<T>(), Box::new(resource));
    }
}
pub unsafe trait Resource: Sized + 'static {
    fn init(world: &mut World);
    fn id() -> Entity;
    fn name() -> &'static str;
}

pub trait ResourcesGetter {
    type Output<'a>;

    fn get(resource_map: &mut ResourceMap) -> Self::Output<'_>;
}

impl<T: Resource> ResourcesGetter for T {
    type Output<'a> = &'a mut T;

    fn get(resources: &mut ResourceMap) -> Self::Output<'_> {
        resources.get_mut::<T>().unwrap()
    }
}

impl<T: Resource, T2: Resource> ResourcesGetter for (T, T2) {
    type Output<'a> = (&'a mut T, &'a mut T2);

    fn get(resources: &mut ResourceMap) -> Self::Output<'_> {
        unsafe {
            let ptr = resources as *mut ResourceMap;
            (
                (*ptr).get_mut::<T>().unwrap(),
                (*ptr).get_mut::<T2>().unwrap(),
            )
        }
    }
}

impl<T: Resource, T2: Resource, T3: Resource> ResourcesGetter for (T, T2, T3) {
    type Output<'a> = (&'a mut T, &'a mut T2, &'a mut T3);

    fn get(resources: &mut ResourceMap) -> Self::Output<'_> {
        unsafe {
            let ptr = resources as *mut ResourceMap;
            (
                (*ptr).get_mut::<T>().unwrap(),
                (*ptr).get_mut::<T2>().unwrap(),
                (*ptr).get_mut::<T3>().unwrap(),
            )
        }
    }
}
impl<T: Resource, T2: Resource, T3: Resource, T4: Resource> ResourcesGetter for (T, T2, T3, T4) {
    type Output<'a> = (&'a mut T, &'a mut T2, &'a mut T3, &'a mut T4);

    fn get(resources: &mut ResourceMap) -> Self::Output<'_> {
        unsafe {
            let ptr = resources as *mut ResourceMap;
            (
                (*ptr).get_mut::<T>().unwrap(),
                (*ptr).get_mut::<T2>().unwrap(),
                (*ptr).get_mut::<T3>().unwrap(),
                (*ptr).get_mut::<T4>().unwrap(),
            )
        }
    }
}
