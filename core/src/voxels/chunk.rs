use anyhow::Result;
use apostasy_macros::{Component, update};
use cgmath::Vector3;
use noise::{NoiseFn, Perlin};
use rand::{RngExt, rng};
use slotmap::DefaultKey;

use crate::{
    objects::{Object, scene::ObjectId, world::World},
    utils::flatten::flatten,
    voxels::{
        VoxelTransform,
        biome::{BiomeId, BiomeRegistry},
        meshes::NeedsRemeshing,
        voxel::{Voxel, VoxelDefinition, VoxelId, VoxelRegistry},
        voxel_raycast::RaycastHit,
    },
};

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

pub fn generate_chunk(
    position: Vector3<i32>,
    registry: &VoxelRegistry,
    biome_registry: &BiomeRegistry,
    lod: u8,
) -> Object {
    let mut rng = rng();
    let biome_index = rng.random_range(0..=biome_registry.defs.len() - 1);
    let biome = biome_registry.defs.get(biome_index).unwrap();

    let surface_voxel = *registry
        .name_to_id
        .get(biome.surface_voxels.first().unwrap())
        .expect("Apostasy:Grass not found in registry");
    let subsurface_voxel = *registry
        .name_to_id
        .get(biome.subsurface_voxels.first().unwrap())
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
                    surface_voxel
                } else {
                    subsurface_voxel
                };
                voxels[flatten(x as u32, y as u32, z as u32, 32)] = id;
            }
        }
    }

    let voxels: Box<[VoxelId; 32 * 32 * 32]> =
        voxels.try_into().expect("voxel array size mismatch");

    let chunk = Chunk {
        voxels,
        lod,
        biome: biome_index as u16,
    };

    let transform = VoxelTransform { position };

    let mut object = Object::new();
    object.set_name("Chunk".to_string());
    object.add_component(transform);
    object.add_component(chunk);
    object.add_tag(NeedsRemeshing);
    object
}

#[update]
pub fn check_voxel_raycast(world: &mut World) -> Result<()> {
    let raycast_hit = world.get_resource_mut::<RaycastHit>()?.clone();

    let Some(set_to) = raycast_hit.set_to else {
        world.remove_resource::<RaycastHit>();
        return Ok(());
    };

    let (target_chunk_pos, target_local_pos) = if set_to != 0 {
        let offset = match raycast_hit.face {
            0 => Vector3::new(1i32, 0, 0),
            1 => Vector3::new(-1, 0, 0),
            2 => Vector3::new(0, 1, 0),
            3 => Vector3::new(0, -1, 0),
            4 => Vector3::new(0, 0, 1),
            5 => Vector3::new(0, 0, -1),
            _ => Vector3::new(0, 0, 0),
        };

        let world_voxel = Vector3::new(
            raycast_hit.chunk_pos.x * 32 + raycast_hit.local_pos.x + offset.x,
            raycast_hit.chunk_pos.y * 32 + raycast_hit.local_pos.y + offset.y,
            raycast_hit.chunk_pos.z * 32 + raycast_hit.local_pos.z + offset.z,
        );

        let chunk_pos = Vector3::new(
            world_voxel.x.div_euclid(32),
            world_voxel.y.div_euclid(32),
            world_voxel.z.div_euclid(32),
        );
        let local_pos = Vector3::new(
            world_voxel.x.rem_euclid(32),
            world_voxel.y.rem_euclid(32),
            world_voxel.z.rem_euclid(32),
        );

        (chunk_pos, local_pos)
    } else {
        (raycast_hit.chunk_pos, raycast_hit.local_pos)
    };

    // collect ObjectIds of chunks that need updating
    let mut chunks_to_update: Vec<ObjectId> = Vec::new();
    for (id, obj) in world.get_objects_with_component_with_ids::<VoxelTransform>() {
        if let Ok(t) = obj.get_component::<VoxelTransform>() {
            if t.position == target_chunk_pos {
                chunks_to_update.push(id);
            }
        }
    }

    for id in chunks_to_update {
        let obj = world.get_object_mut(id).unwrap();
        let chunk = obj.get_component_mut::<Chunk>()?;

        if set_to != 0 {
            chunk.set_if_empty(
                target_local_pos.x as u32,
                target_local_pos.y as u32,
                target_local_pos.z as u32,
                Voxel { id: set_to },
            );
        } else {
            chunk.set(
                target_local_pos.x as u32,
                target_local_pos.y as u32,
                target_local_pos.z as u32,
                Voxel { id: 0 },
            );
        }

        obj.add_tag(NeedsRemeshing);

        let neighbour_offsets = [
            (Vector3::new(1i32, 0, 0), target_local_pos.x == 31),
            (Vector3::new(-1, 0, 0), target_local_pos.x == 0),
            (Vector3::new(0, 1, 0), target_local_pos.y == 31),
            (Vector3::new(0, -1, 0), target_local_pos.y == 0),
            (Vector3::new(0, 0, 1), target_local_pos.z == 31),
            (Vector3::new(0, 0, -1), target_local_pos.z == 0),
        ];

        let mut neighbour_ids: Vec<ObjectId> = Vec::new();
        for (offset, is_border) in &neighbour_offsets {
            if *is_border {
                let neighbour_pos = target_chunk_pos + offset;
                for (id, obj) in world.get_objects_with_component_with_ids::<VoxelTransform>() {
                    if let Ok(t) = obj.get_component::<VoxelTransform>() {
                        if t.position == neighbour_pos {
                            neighbour_ids.push(id);
                        }
                    }
                }
            }
        }

        for nid in neighbour_ids {
            if let Some(nobj) = world.get_object_mut(nid) {
                nobj.add_tag(NeedsRemeshing);
            }
        }

        break;
    }

    world.remove_resource::<RaycastHit>();
    Ok(())
}
