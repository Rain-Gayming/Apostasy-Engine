use crate::{log, objects::world::World, voxels::voxel::VoxelRegistry};

pub(crate) fn add_voxel_package(world: &mut World) {
    log!("Implimanting voxel package");
    world.insert_resource(VoxelRegistry::default());
}
