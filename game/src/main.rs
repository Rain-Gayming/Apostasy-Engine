use apostasy_core::{init_core, packages::Packages, rendering::RenderingBackend};
pub mod entities;

fn main() {
    init_core(
        RenderingBackend::Vulkan,
        vec![Packages::Voxel, Packages::ItemSystem],
    )
    .unwrap();
}
