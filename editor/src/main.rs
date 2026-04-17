use apostasy_core::{init_core, rendering::RenderingBackend};

pub mod editor_camera;
pub mod input;

fn main() {
    init_core(RenderingBackend::Vulkan).unwrap();
}
