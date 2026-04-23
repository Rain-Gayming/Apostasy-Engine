use crate::{objects::world::World, packages::voxel_package::add_voxel_package};

pub mod voxel_package;

#[derive(Clone, Copy)]
pub enum Packages {
    Voxel,
}

pub fn add_package(world: &mut World, package: Packages) {
    match package {
        Packages::Voxel => {
            add_voxel_package(world);
        }
    }
}
