use crate::engine::ecs::{entity::Entity, world::World};

pub unsafe trait Component: Sized {
    fn id() -> Entity;
    fn init(_: &World);
    fn info() -> ComponentInfo;
}

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
