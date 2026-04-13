use apostasy_core::{
    Component, init_core, objects::world::World, rendering::RenderingBackend, start, update,
};
fn main() {
    init_core(RenderingBackend::Vulkan).unwrap();
}

#[derive(Component, Clone)]
pub struct T {}
