use apostasy_core::{
    cgmath::Vector3,
    noise::{NoiseFn, Perlin},
    objects::Object,
    utils::flatten::flatten,
    voxels::{
        VoxelTransform,
        biome::{BiomeRegistry, sample_biome_weights},
        chunk::Chunk,
        meshes::NeedsRemeshing,
        voxel::{VoxelId, VoxelRegistry},
    },
};

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
