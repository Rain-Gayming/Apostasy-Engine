use anyhow::Result;
use apostasy_macros::{Component, Resource, fixed_update, update};
use cgmath::Vector3;
use hashbrown::HashMap;
use noise::{NoiseFn, Perlin};

use crate::{
    objects::{Object, scene::ObjectId, world::World},
    utils::flatten::flatten,
    voxels::{
        VoxelTransform,
        biome::{BiomeId, BiomeRegistry, sample_biome_weights},
        meshes::NeedsRemeshing,
        voxel::{Voxel, VoxelDefinition, VoxelId, VoxelRegistry},
        voxel_components::break_ticks::BreakTicks,
        voxel_raycast::RaycastHit,
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
pub fn generate_chunk(
    position: Vector3<i32>,
    registry: &VoxelRegistry,
    biome_registry: &BiomeRegistry,
    seed: u32,
    lod: u8,
) -> Object {
    let noise = Perlin::new(seed);

    let world_x = position.x as f64 * 32.0;
    let world_y = position.y as f64 * 32.0;
    let world_z = position.z as f64 * 32.0;

    let mut heightmap = [0i32; 32 * 32];
    let mut column_biome = [0u16; 32 * 32];

    for z in 0..32usize {
        for x in 0..32usize {
            let wx = world_x + x as f64;
            let wz = world_z + z as f64;

            let weights = sample_biome_weights(wx, wz, biome_registry, seed, 0.05);

            let mut blended_height = 0.0f64;
            let mut dominant_biome = 0u16;
            let mut dominant_weight = 0.0f64;

            for (biome_id, weight) in &weights {
                let biome = &biome_registry.defs[*biome_id as usize];
                let nx = wx * biome.frequency;
                let nz = wz * biome.frequency;
                let val = noise.get([nx, nz]) * biome.amplitude;
                blended_height += (10.0 + val) * weight;

                if *weight > dominant_weight {
                    dominant_weight = *weight;
                    dominant_biome = *biome_id;
                }
            }

            heightmap[z * 32 + x] = blended_height as i32;
            column_biome[z * 32 + x] = dominant_biome;
        }
    }

    let mut voxels = vec![0u16; 32 * 32 * 32].into_boxed_slice();

    for z in 0..32usize {
        for x in 0..32usize {
            let surface_y = heightmap[z * 32 + x];
            let biome_id = column_biome[z * 32 + x];
            let biome = &biome_registry.defs[biome_id as usize];

            let surface_voxel = *registry
                .name_to_id
                .get(biome.surface_voxels.first().unwrap())
                .expect("surface voxel not found");
            let subsurface_voxel = *registry
                .name_to_id
                .get(biome.subsurface_voxels.first().unwrap())
                .expect("subsurface voxel not found");

            for y in 0..32usize {
                let wy = world_y as i32 + y as i32;
                let id = if wy > surface_y {
                    0
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

    let center_biome = column_biome[16 * 32 + 16];

    let chunk = Chunk {
        voxels,
        lod,
        biome: center_biome,
    };
    let transform = VoxelTransform { position };

    let mut object = Object::new();
    object.set_name("Chunk".to_string());
    object.add_component(transform);
    object.add_component(chunk);
    object.add_tag(NeedsRemeshing);
    object
}
#[fixed_update]
pub fn check_voxel_raycast(world: &mut World, delta: f32) -> Result<()> {
    let Ok(raycast_hit) = world.get_resource::<RaycastHit>() else {
        // no active raycast — clear all break progress since player stopped looking
        if let Ok(progress) = world.get_resource_mut::<VoxelBreakProgress>() {
            progress.progress.clear();
        }
        return Ok(());
    };
    let raycast_hit = raycast_hit.clone();

    let Some(set_to) = raycast_hit.set_to else {
        world.remove_resource::<RaycastHit>();
        return Ok(());
    };

    // breaking a voxel (set_to == 0)
    if set_to == 0 {
        let hit_world_pos = (
            raycast_hit.chunk_pos.x * 32 + raycast_hit.local_pos.x,
            raycast_hit.chunk_pos.y * 32 + raycast_hit.local_pos.y,
            raycast_hit.chunk_pos.z * 32 + raycast_hit.local_pos.z,
        );

        // get the voxel's break ticks requirement
        let registry = world.get_resource::<VoxelRegistry>()?.clone();

        // find the voxel id at the hit position
        let voxel_id = world
            .get_objects_with_component::<VoxelTransform>()
            .iter()
            .find_map(|obj| {
                let t = obj.get_component::<VoxelTransform>().ok()?;
                if t.position != raycast_hit.chunk_pos {
                    return None;
                }
                let chunk = obj.get_component::<Chunk>().ok()?;
                Some(
                    chunk.voxels[flatten(
                        raycast_hit.local_pos.x as u32,
                        raycast_hit.local_pos.y as u32,
                        raycast_hit.local_pos.z as u32,
                        32,
                    )],
                )
            });

        let Some(voxel_id) = voxel_id else {
            world.remove_resource::<RaycastHit>();
            return Ok(());
        };

        let def = &registry.defs[voxel_id as usize];

        // get required break ticks — if no BreakTicks component, voxel is unbreakable
        let Ok(break_ticks) = def.get_component::<BreakTicks>() else {
            world.remove_resource::<RaycastHit>();
            return Ok(());
        };
        let required_ticks = break_ticks.0;

        // increment progress for this voxel
        // clear progress on any other voxel (player switched target)
        let current_ticks = {
            let progress = world.get_resource_mut::<VoxelBreakProgress>().unwrap();

            // clear progress on voxels that are no longer being targeted
            progress.progress.retain(|pos, _| *pos == hit_world_pos);

            let ticks = progress.progress.entry(hit_world_pos).or_insert(0);
            *ticks += 1;
            *ticks
        };

        if current_ticks >= required_ticks {
            // voxel is fully broken — remove it
            world
                .get_resource_mut::<VoxelBreakProgress>()
                .unwrap()
                .progress
                .remove(&hit_world_pos);

            // find and update the chunk
            let mut chunks_to_update: Vec<ObjectId> = Vec::new();
            for (id, obj) in world.get_objects_with_component_with_ids::<VoxelTransform>() {
                if let Ok(t) = obj.get_component::<VoxelTransform>() {
                    if t.position == raycast_hit.chunk_pos {
                        chunks_to_update.push(id);
                    }
                }
            }

            world.remove_resource::<RaycastHit>();
            for id in chunks_to_update {
                let obj = world.get_object_mut(id).unwrap();
                obj.get_component_mut::<Chunk>()?.set(
                    raycast_hit.local_pos.x as u32,
                    raycast_hit.local_pos.y as u32,
                    raycast_hit.local_pos.z as u32,
                    Voxel { id: 0 },
                );
                obj.add_tag(NeedsRemeshing);
            }
        }

        return Ok(());
    }

    // placing a voxel — existing placement code unchanged
    let (target_chunk_pos, target_local_pos) = {
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

        (
            Vector3::new(
                world_voxel.x.div_euclid(32),
                world_voxel.y.div_euclid(32),
                world_voxel.z.div_euclid(32),
            ),
            Vector3::new(
                world_voxel.x.rem_euclid(32),
                world_voxel.y.rem_euclid(32),
                world_voxel.z.rem_euclid(32),
            ),
        )
    };

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
        obj.get_component_mut::<Chunk>()?.set_if_empty(
            target_local_pos.x as u32,
            target_local_pos.y as u32,
            target_local_pos.z as u32,
            Voxel { id: set_to },
        );
        obj.add_tag(NeedsRemeshing);
        break;
    }

    world.remove_resource::<RaycastHit>();
    Ok(())
}
