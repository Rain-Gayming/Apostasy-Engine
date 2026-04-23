use apostasy_macros::Component;
use cgmath::Vector3;
use noise::{NoiseFn, Perlin};

use crate::{
    objects::Object,
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

pub fn generate_chunk(position: Vector3<i32>, registry: &VoxelRegistry, lod: u8) -> Object {
    let grass_id = *registry
        .name_to_id
        .get("Apostasy:Grass")
        .expect("Apostasy:Grass not found in registry");
    let dirt_id = *registry
        .name_to_id
        .get("Apostasy:Dirt")
        .expect("Apostasy:Dirt not found in registry");

    let noise = Perlin::new(12345);

    let world_x = position.x as f64 * 32.0;
    let world_y = position.y as f64 * 32.0;
    let world_z = position.z as f64 * 32.0;

    let mut heightmap = [0i32; 32 * 32];
    for z in 0..32usize {
        for x in 0..32usize {
            let nx = (world_x + x as f64) * 0.05;
            let nz = (world_z + z as f64) * 0.05;
            let val = noise.get([nx, nz]) * 7.0;
            heightmap[z * 32 + x] = (10.0 + val) as i32;
        }
    }

    let mut voxels = vec![0u16; 32 * 32 * 32].into_boxed_slice();

    for z in 0..32usize {
        for x in 0..32usize {
            let surface_y = heightmap[z * 32 + x];
            for y in 0..32usize {
                let wy = world_y as i32 + y as i32;
                let id = if wy > surface_y {
                    0 // air
                } else if wy == surface_y {
                    grass_id
                } else {
                    dirt_id
                };
                voxels[flatten(x as u32, y as u32, z as u32, 32)] = id;
            }
        }
    }

    let voxels: Box<[VoxelId; 32 * 32 * 32]> =
        voxels.try_into().expect("voxel array size mismatch");

    let chunk = Chunk { voxels, lod };

    let transform = VoxelTransform { position };

    let mut object = Object::new();
    object.set_name("Chunk".to_string());
    object.add_component(transform);
    object.add_component(chunk);
    object.add_tag(NeedsRemeshing);
    object
}
