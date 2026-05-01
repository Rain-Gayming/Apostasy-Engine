use apostasy_macros::{Component, Resource};
use hashbrown::HashMap;

use crate::{
    utils::flatten::flatten,
    voxels::{
        biome::BiomeId,
        voxel::{Voxel, VoxelDefinition, VoxelId, VoxelRegistry},
    },
};

#[derive(Resource, Clone, Default)]
pub struct VoxelBreakProgress {
    pub progress: HashMap<(i32, i32, i32), u32>,
}

#[derive(Clone, Component, Debug)]
pub struct Chunk {
    pub voxels: Box<[VoxelId; 32 * 32 * 32]>,
    pub lod: u8,
    pub biome: BiomeId,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            voxels: Box::new([VoxelId::default(); 32 * 32 * 32]),
            lod: 1,
            biome: 0,
        }
    }
}

impl Chunk {
    pub fn deserialize(&mut self, _value: &serde_yaml::Value) -> anyhow::Result<()> {
        Ok(())
    }
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

    pub fn set_if_empty(&mut self, x: u32, y: u32, z: u32, voxel: Voxel) {
        if self.voxels[flatten(x, y, z, 32)] == 0 {
            self.voxels[flatten(x, y, z, 32)] = voxel.id;
        }
    }

    pub fn set_lod(&mut self, lod: u8) {
        self.lod = lod;
    }
}
