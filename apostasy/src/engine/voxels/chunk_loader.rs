use crate::log;
use std::collections::HashMap;

use apostasy_macros::{Component, Resource, update};
use cgmath::Vector3;

use crate::{
    self as apostasy,
    engine::{
        ecs::{
            Package, World,
            components::transform::{Transform, VoxelChunkTransform},
            entity::Entity,
        },
        voxels::voxel_chunk::{CHUNK_SIZE, UngeneratedVoxelChunk, UnmeshedVoxelChunk, VoxelChunk},
    },
};

#[derive(Component, Default)]
pub struct ChunkLoaderFlag;

#[derive(Resource, Default)]
pub struct ChunkStorage {
    pub loaded_chunks: HashMap<Vector3<i32>, Entity>,
}

#[update(priority = 1)]
pub fn load_chunks(world: &mut World) {
    if !world.packages.contains(&Package::Voxels) {
        return;
    }
    world
        .query()
        .include::<ChunkLoaderFlag>()
        .include::<Transform>()
        .build()
        .run(|entity| {
            world.with_resource_mut::<ChunkStorage, _, _>(|storage| {
                let transform = entity.get::<Transform>().unwrap();
                let transform_chunk = Vector3::new(
                    transform.position.x.floor() as i32,
                    transform.position.y.floor() as i32,
                    transform.position.z.floor() as i32,
                );
                let chunk_pos = transform_chunk / CHUNK_SIZE as i32;

                for x in chunk_pos.x - 2..=chunk_pos.x + 2 {
                    for y in chunk_pos.y - 2..=chunk_pos.y + 2 {
                        for z in chunk_pos.z - 2..=chunk_pos.z + 2 {
                            if y > -2 {
                                continue;
                            }
                            let chunk_pos = Vector3::new(x, y, z);
                            if !storage.loaded_chunks.contains_key(&chunk_pos) {
                                let entity = world
                                    .spawn()
                                    .insert(VoxelChunk::default())
                                    .insert(VoxelChunkTransform {
                                        position: chunk_pos * CHUNK_SIZE as i32,
                                    })
                                    .insert(UngeneratedVoxelChunk)
                                    .insert(UnmeshedVoxelChunk);
                                storage.loaded_chunks.insert(chunk_pos, entity.entity);
                                log!("Loaded chunk at {:?}", chunk_pos);
                            }
                        }
                    }
                }
            });
        });
}
