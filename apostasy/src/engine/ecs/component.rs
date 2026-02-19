use std::{
    collections::HashMap,
    fmt::Debug,
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
};

use apostasy_macros::Component;
use derive_more::{Deref, DerefMut, From};

use crate::{
    self as apostasy,
    engine::ecs::{
        World,
        archetype::{ArchetypeId, ColumnIndex},
        entity::{Entity, EntityView},
    },
};

pub type ComponentEntry = fn(world: &World);

#[linkme::distributed_slice]
pub static COMPONENT_ENTRIES: [ComponentEntry];

#[derive(Clone, Copy, Hash, From, PartialOrd, Ord, Debug, PartialEq, Eq)]
pub struct ComponentId(pub u64);
impl From<Entity> for ComponentId {
    fn from(entity: Entity) -> Self {
        Self(entity.raw() & u32::MAX as u64)
    }
}

impl ComponentId {
    pub fn as_entity(&self) -> Option<Entity> {
        Some(Entity::from_raw(self.0))
    }
}

#[allow(clippy::type_complexity, clippy::missing_safety_doc)]
pub unsafe trait Component: Sized + Default {
    fn id() -> Entity;
    fn info() -> ComponentInfo;
    fn init(_: &World);

    fn get_erased_clone() -> Option<unsafe fn(&[MaybeUninit<u8>]) -> &'static [MaybeUninit<u8>]> {
        struct Getter<T>(PhantomData<T>);
        impl<T: Clone> Getter<T> {
            #[allow(dead_code)]
            fn get() -> Option<unsafe fn(&[MaybeUninit<u8>]) -> &'static [MaybeUninit<u8>]> {
                Some(|bytes| unsafe {
                    let t = (bytes.as_ptr() as *const T).as_ref().unwrap();
                    let leaked = ManuallyDrop::new(t.clone());
                    std::slice::from_raw_parts((&raw const leaked).cast(), size_of::<Self>())
                })
            }
        }
        trait NoImpl<T> {
            fn get() -> Option<unsafe fn(&[MaybeUninit<u8>]) -> &'static [MaybeUninit<u8>]> {
                None
            }
        }
        impl<T> NoImpl<T> for Getter<T> {}
        Getter::<Self>::get()
    }
    fn get_erased_default() -> fn() -> &'static [MaybeUninit<u8>] {
        || {
            let val = Self::default();
            let mut bytes = vec![MaybeUninit::<u8>::uninit(); size_of::<Self>()];
            unsafe {
                std::ptr::copy_nonoverlapping(
                    &val as *const Self as *const MaybeUninit<u8>,
                    bytes.as_mut_ptr(),
                    size_of::<Self>(),
                );
            }
            std::mem::forget(val);
            Box::leak(bytes.into_boxed_slice())
        }
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe fn erased_drop(bytes: &mut [std::mem::MaybeUninit<u8>]) {
        unsafe { (bytes.as_ptr() as *mut Self).drop_in_place() }
    }

    fn get_on_insert() -> Option<fn(EntityView<'_>)> {
        struct Getter<T>(PhantomData<T>);
        impl<T: OnInsert> Getter<T> {
            #[allow(dead_code)]
            fn get() -> Option<fn(EntityView<'_>)> {
                Some(T::on_insert)
            }
        }
        trait NoImpl<T> {
            fn get() -> Option<fn(EntityView<'_>)> {
                None
            }
        }
        impl<T> NoImpl<T> for Getter<T> {}
        Getter::<Self>::get()
    }

    fn get_on_remove() -> Option<fn(EntityView<'_>)> {
        struct Getter<T>(PhantomData<T>);
        impl<T: OnRemove> Getter<T> {
            #[allow(dead_code)]
            fn get() -> Option<fn(EntityView<'_>)> {
                Some(T::on_remove)
            }
        }
        trait NoImpl<T> {
            fn get() -> Option<fn(EntityView<'_>)> {
                None
            }
        }
        impl<T> NoImpl<T> for Getter<T> {}
        Getter::<Self>::get()
    }
    fn get_erased_fmt()
    -> Option<fn(&[MaybeUninit<u8>], &mut std::fmt::Formatter<'_>) -> std::fmt::Result> {
        struct Getter<T>(PhantomData<T>);

        impl<T: std::fmt::Debug> Getter<T> {
            #[allow(dead_code)]
            fn get()
            -> Option<fn(&[MaybeUninit<u8>], &mut std::fmt::Formatter<'_>) -> std::fmt::Result>
            {
                Some(|bytes, f| {
                    let val = unsafe { (bytes.as_ptr() as *const T).as_ref().unwrap() };
                    write!(f, "{:?}", val)
                })
            }
        }

        trait NoImpl<T> {
            fn get()
            -> Option<fn(&[MaybeUninit<u8>], &mut std::fmt::Formatter<'_>) -> std::fmt::Result>
            {
                None
            }
        }
        impl<T> NoImpl<T> for Getter<T> {}
        Getter::<Self>::get()
    }
}

pub trait OnInsert {
    fn on_insert(entity: EntityView<'_>);
}

pub trait OnRemove {
    fn on_remove(entity: EntityView<'_>);
}

#[derive(Clone, Copy, Debug, Component)]
#[allow(clippy::type_complexity)]
pub struct ComponentInfo {
    pub name: &'static str,
    pub id: Entity,
    pub size: usize,
    pub align: usize,
    pub drop: unsafe fn(&mut [MaybeUninit<u8>]),
    pub clone: Option<unsafe fn(&[MaybeUninit<u8>]) -> &'static [MaybeUninit<u8>]>,
    pub default: Option<fn() -> &'static [MaybeUninit<u8>]>,
    pub on_insert: Option<fn(EntityView<'_>)>,
    pub on_remove: Option<fn(EntityView<'_>)>,
    pub fmt: Option<fn(&[MaybeUninit<u8>], &mut std::fmt::Formatter<'_>) -> std::fmt::Result>,
}

impl Default for ComponentInfo {
    fn default() -> Self {
        Self {
            name: "",
            id: Entity::from_raw(0),
            size: 0,
            align: 0,
            drop: |_| {},
            clone: None,
            default: None,
            on_insert: None,
            on_remove: None,
            fmt: None,
        }
    }
}

#[derive(Deref, DerefMut, Default, Debug)]
pub struct ComponentLocations(pub HashMap<ArchetypeId, ColumnIndex>);

// Component info should hold:
//  - bit size of the component
//  - the components id
//  - components name
//  - the components align size
//

// Component trait should store:
//  - a reference to the component info
//  - a direct to its id
//  - how its initialized into the world
// Components are stored as entities in the world
