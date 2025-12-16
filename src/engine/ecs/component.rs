use std::any::type_name;

use crate::engine::ecs::{entity::Entity, world::World};

pub type ComponentEntry = fn(world: &World);

#[linkme::distributed_slice]
pub static COMPONENT_ENTRIES: [ComponentEntry];

pub trait Component: Sized {
    fn id() -> Entity;
    fn init(_: &World);
    fn info() -> ComponentInfo;
}

#[derive(Debug)]
pub struct ComponentInfo {
    pub name: &'static str,
    pub align: usize,
    pub size: usize,
    pub id: Entity,
    // pub clone: Option<unsafe fn(&[MaybeUninit<u8>]) -> &'static [MaybeUninit<u8>]>,
    // pub default: Option<fn() -> &'static [MaybeUninit<u8>]>,
    // pub drop: unsafe fn(&mut [MaybeUninit<u8>]),
    // pub on_insert: Option<fn(View<'_>)>,
    // pub on_remove: Option<fn(View<'_>)>,
}

impl Component for ComponentInfo {
    fn id() -> Entity {
        static ENTRY: ComponentEntry = ComponentInfo::init;
        let begin = COMPONENT_ENTRIES[..].as_ptr() as u32;
        let end = &raw const ENTRY as u32;
        unsafe { Entity::from_offset((end - begin) / size_of::<ComponentEntry>() as u32) }
    }

    fn init(world: &World) {
        world
            .entity(ComponentInfo::id())
            .insert(ComponentInfo::info());
    }

    fn info() -> ComponentInfo {
        unsafe {
            ComponentInfo {
                name: type_name::<ComponentInfo>(),
                align: align_of::<ComponentInfo>(),
                size: size_of::<ComponentInfo>(),
                id: ComponentInfo::id(),
            }
        }
    }
}
