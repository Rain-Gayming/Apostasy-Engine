use std::{
    path::Path,
    sync::{Arc, RwLock},
};

use crate::{
    assets::{asset_manager::AssetManager, loaders::voxel_loader::VoxelLoader},
    log,
    objects::world::World,
    voxels::{
        chunk_loader::ChunkLoader,
        texture_atlas::{AtlasBuilder, PendingAtlas},
        voxel::VoxelRegistry,
    },
};

pub(crate) fn add_voxel_package(world: &mut World) {
    log!("Implimanting voxel package");

    let voxel_registry = Arc::new(RwLock::new(VoxelRegistry::default()));
    let atlas_builder = Arc::new(RwLock::new(AtlasBuilder::new(16)));

    {
        let mut asset_manager = AssetManager::new();
        asset_manager.register_loader(VoxelLoader {
            registry: Arc::clone(&voxel_registry),
            atlas_builder: Arc::clone(&atlas_builder),
        });
        asset_manager
            .load_directory(Path::new(&format!(
                "{}/{}",
                env!("CARGO_MANIFEST_DIR"),
                "res/"
            )))
            .unwrap();

        asset_manager.load_directory(Path::new("res/")).unwrap();
    }

    let registry = Arc::try_unwrap(voxel_registry)
        .expect("VoxelRegistry still has multiple owners")
        .into_inner()
        .expect("VoxelRegistry RwLock poisoned");

    let atlas_builder = Arc::try_unwrap(atlas_builder)
        .unwrap()
        .into_inner()
        .unwrap();

    let (atlas_image, atlas_tiles) = atlas_builder.build();

    world.insert_resource(registry);
    world.insert_resource(PendingAtlas {
        image: atlas_image,
        tiles: atlas_tiles,
    });

    world.insert_resource(ChunkLoader::default());
}
