use apostasy_core::{init_core, rendering::RenderingBackend};

fn main() {
    init_core(RenderingBackend::Vulkan).unwrap();
}
