use apostasy_core::{
    Component, init_core, objects::world::World, rendering::RenderingBackend, start,
};
fn main() {
    init_core(RenderingBackend::Vulkan).unwrap();
}

#[start]
pub fn istart(world: &mut World) {
    world.add_new_node();

    world.debug_nodes();
}

#[derive(Component, Clone)]
pub struct T {}
