use apostasy_core::{
    cgmath::Vector3,
    noise::{NoiseFn, Perlin},
    utils::flatten::flatten,
    voxels::{
        biome::{BiomeRegistry, NOISE, sample_biome_weights},
        chunk::GeneratedChunkData,
        voxel::{VoxelId, VoxelRegistry},
    },
};

fn fractal_brownian_motion(
    noise: &Perlin,
    x: f64,
    z: f64,
    octaves: u32,
    lacunarity: f64,
    gain: f64,
) -> f64 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        value += noise.get([x * frequency, z * frequency]) * amplitude;
        max_value += amplitude;
        amplitude *= gain;
        frequency *= lacunarity;
    }

    value / max_value // normalized to [-1, 1]
}

fn smooth_weight(w: f64) -> f64 {
    w * w * (3.0 - 2.0 * w)
}

fn apply_height_curve(val: f64) -> f64 {
    if val > 0.0 { val.powf(1.5) } else { val }
}

fn lod_octaves(biome_octaves: u32, lod: u8) -> u32 {
    match lod {
        1 => biome_octaves,
        2 => (biome_octaves - 1).max(2),
        3 => (biome_octaves - 2).max(2),
        _ => 2,
    }
}

pub fn generate_chunk_data(
    position: Vector3<i32>,
    registry: &VoxelRegistry,
    biome_registry: &BiomeRegistry,
    seed: u32,
    lod: u8,
) -> GeneratedChunkData {
    let noise = NOISE.get_or_init(|| Perlin::new(seed));
    let world_x = position.x as f64 * 32.0;
    let world_y = position.y as f64 * 32.0;
    let world_z = position.z as f64 * 32.0;

    let base_height = 64.0_f64;

    let mut heightmap = [0i32; 32 * 32];
    let mut column_biome = [0u16; 32 * 32];

    for z in 0..32usize {
        for x in 0..32usize {
            let wx = world_x + x as f64;
            let wz = world_z + z as f64;

            let weights = sample_biome_weights(wx, wz, biome_registry, seed, 0.05);

            // Smooth and renormalize weights
            let smoothed: Vec<(u16, f64)> = weights
                .iter()
                .map(|(id, w)| (*id, smooth_weight(*w)))
                .collect();
            let weight_sum: f64 = smoothed.iter().map(|(_, w)| w).sum();

            let mut blended_height = 0.0f64;
            let mut dominant_biome = 0u16;
            let mut dominant_weight = 0.0f64;

            for (biome_id, raw_weight) in &smoothed {
                let weight = raw_weight / weight_sum; // renormalize
                let biome = &biome_registry.defs[*biome_id as usize];

                let nx = wx * biome.frequency;
                let nz = wz * biome.frequency;

                let octaves = lod_octaves(biome.octaves, lod);
                let val = fractal_brownian_motion(&noise, nx, nz, octaves, 2.0, 0.5);
                let shaped = apply_height_curve(val);

                blended_height += (base_height + shaped * biome.amplitude) * weight;

                if weight > dominant_weight {
                    dominant_weight = weight;
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
            let underground_voxel = *registry
                .name_to_id
                .get(biome.underground_voxels.first().unwrap())
                .expect("subsurface voxel not found");

            for y in 0..32usize {
                let wy = world_y as i32 + y as i32;
                let depth = surface_y - wy;

                let id = if wy > surface_y {
                    0 // air
                } else if depth == 0 {
                    surface_voxel
                } else if depth < 4 {
                    subsurface_voxel
                } else {
                    underground_voxel
                };

                voxels[flatten(x as u32, y as u32, z as u32, 32)] = id;
            }
        }
    }

    let voxels: Box<[VoxelId; 32 * 32 * 32]> =
        voxels.try_into().expect("voxel array size mismatch");

    let center_biome = column_biome[16 * 32 + 16];

    GeneratedChunkData {
        position,
        voxels,
        lod,
        biome: center_biome,
    }
}
