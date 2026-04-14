use apostasy_core::{
    Component, anyhow::Result, init_core, objects::world::World, rendering::RenderingBackend,
    start, update,
};
use apostasy_macros::Resource;

fn main() {
    init_core(RenderingBackend::Vulkan).unwrap();
}

#[derive(Component, Clone)]
pub struct T {}

#[derive(Resource, Clone)]
pub struct R {}

#[derive(Resource, Clone)]
pub struct Rb {
    b: i32,
}

#[start]
pub fn s(world: &mut World) -> Result<()> {
    world.insert_resource(Rb { b: 32 });

    Ok(())
}

#[update]
pub fn u(world: &mut World) -> Result<()> {
    let r = world.get_resource::<Rb>()?;

    Ok(())
}
