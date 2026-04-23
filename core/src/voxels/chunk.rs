use anyhow::Result;
use apostasy_macros::Component;
use apostasy_macros::start;
use cgmath::Vector3;
use noise::{NoiseFn, Perlin};
use rand::RngExt;

use crate::{
    objects::{Object, components::transform::Transform, world::World},
    utils::flatten::flatten,
    voxels::{
        VoxelTransform,
        meshes::NeedsRemeshing,
        voxel::{Voxel, VoxelDefinition, VoxelId, VoxelRegistry},
    },
};

#[derive(Clone, Component, Debug)]
pub struct Chunk {
    pub voxels: Box<[VoxelId; 32 * 32 * 32]>,
    pub lod: u8,
}
impl Default for Chunk {
    fn default() -> Self {
        Self {
            voxels: Box::new([VoxelId::default(); 32 * 32 * 32]),
            lod: 1,
        }
    }
}

impl Chunk {
    fn _get_def<'a>(
        &self,
        x: u32,
        y: u32,
        z: u32,
        registry: &'a VoxelRegistry,
    ) -> &'a VoxelDefinition {
        let id = self.voxels[flatten(x, y, z, 32)];
        &registry.defs[id as usize]
    }

    pub fn set(&mut self, x: u32, y: u32, z: u32, voxel: Voxel) {
        self.voxels[flatten(x, y, z, 32)] = voxel.id;
    }
    pub fn set_lod(&mut self, lod: u8) {
        self.lod = lod;
    }
}

#[start]
pub fn create_test_chunk(world: &mut World) -> Result<()> {
    let registry = world
        .get_resource::<VoxelRegistry>()
        .expect("VoxelRegistry not loaded");

    for reg in registry.defs.iter() {
        println!("{}:{}", reg.namespace, reg.name);
    }
    let grass_id = registry
        .name_to_id
        .get("Apostasy:Grass")
        .copied()
        .expect("Apostasy:Dirt not found in registry");

    let dirt_id = registry
        .name_to_id
        .get("Apostasy:Dirt")
        .copied()
        .expect("Apostasy:Dirt not found in registry");

    for step in 1..4 {
        let mut chunk = Chunk::default();

        let noise = Perlin::new(12345);
        let lod = step;

        for z in 0..32u32 {
            for y in 0..32u32 {
                for x in 0..32u32 {
                    let val = noise.get([
                        (x as f64 * step as f64) * 0.05,
                        y as f64 * 0.05,
                        z as f64 * 0.05,
                    ]) * 7.0;

                    if y as f64 > 10.0 + val {
                        chunk.set(x, y, z, Voxel { id: 0 });
                    } else if y as f64 > 10.0 - 3.0 {
                        chunk.set(x, y, z, Voxel { id: grass_id });
                    } else {
                        chunk.set(x, y, z, Voxel { id: dirt_id });
                    }
                }
            }
        }

        chunk.set_lod(lod);

        let transform = VoxelTransform {
            position: Vector3::new(step as i32, 0, 0),
        };

        let mut object = Object::new();
        object.set_name("Chunk".to_string());
        object.add_component(transform);
        object.add_component(chunk);
        object.add_tag(NeedsRemeshing);
        world.add_object(object);
    }

    Ok(())
}
