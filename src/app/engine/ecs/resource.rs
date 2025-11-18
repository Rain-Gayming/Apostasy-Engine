use std::cell::RefMut;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::panic;
use std::{
    any::TypeId,
    cell::{Ref, RefCell},
    collections::HashMap,
    marker::PhantomData,
};

use downcast_rs::{Downcast, impl_downcast};

use crate::app::engine::ecs::systems::SystemParam;

/// Resource trait
pub trait Resource: Downcast {}
impl_downcast!(Resource);
impl Debug for dyn Resource {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_struct("type")
            .field("type_id", &::std::any::TypeId::of::<Self>())
            .finish()
    }
}

/// Reference to a resource
pub struct Res<'a, T: 'static> {
    pub value: Ref<'a, Box<dyn Resource>>,
    _marker: PhantomData<&'a T>,
}

/// Allows for derefencing and downcasting of Res<>
impl<T: 'static + Resource> Deref for Res<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.value.downcast_ref().unwrap()
    }
}

/// Allows resources to be passed as a parameter for systems
impl<'res, T: 'static> SystemParam for Res<'res, T> {
    type Item<'new> = Res<'new, T>;

    fn retrieve<'r>(resources: &'r HashMap<TypeId, RefCell<Box<dyn Resource>>>) -> Self::Item<'r> {
        Res {
            value: resources.get(&TypeId::of::<T>()).unwrap().borrow(),
            _marker: PhantomData,
        }
    }
}

/// Mutable reference to a resource
pub struct ResMut<'a, T: 'static> {
    pub value: RefMut<'a, Box<dyn Resource>>,
    _marker: PhantomData<&'a mut T>,
}

/// Allows for downcasting and refencing to a ref ofof Res<>
impl<T: 'static + Resource> Deref for ResMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.value.downcast_ref().unwrap()
    }
}

/// Allows for downcasting and mut refencing to a ref ofof Res<>
impl<T: 'static + Resource> DerefMut for ResMut<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.value.downcast_mut().unwrap()
    }
}

/// Man fuck if i know
impl<'res, T: 'static + Resource> SystemParam for ResMut<'res, T> {
    type Item<'new> = ResMut<'new, T>;

    fn retrieve<'r>(resources: &'r HashMap<TypeId, RefCell<Box<dyn Resource>>>) -> Self::Item<'r> {
        if resources.get(&TypeId::of::<T>()).is_none() {
            panic!("RESOURCE NOT FOUND");
        }

        ResMut {
            value: resources.get(&TypeId::of::<T>()).unwrap().borrow_mut(),
            _marker: PhantomData,
        }
    }
}
