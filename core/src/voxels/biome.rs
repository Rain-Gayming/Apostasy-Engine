use apostasy_macros::Resource;
use hashbrown::HashMap;
use noise::{NoiseFn, Perlin};

pub type BiomeId = u16;
#[derive(Resource, Default, Clone, Debug)]
pub struct BiomeRegistry {
    pub defs: Vec<BiomeDefinition>,
    pub name_to_id: HashMap<String, BiomeId>,
    pub id_to_name: HashMap<BiomeId, String>,
}

#[derive(Clone, Debug)]
pub struct BiomeDefinition {
    pub name: String,
    pub namespace: String,
    pub class: String,

    pub surface_voxels: Vec<String>,
    pub subsurface_voxels: Vec<String>,

    pub amplitude: f64,
    pub frequency: f64,

    pub temperature: f64,
    pub humidity: f64,
}

pub fn select_biome(world_x: f64, world_z: f64, registry: &BiomeRegistry, seed: u32) -> BiomeId {
    let temp_noise = Perlin::new(seed);
    let humid_noise = Perlin::new(seed.wrapping_add(1));

    let temperature = (temp_noise.get([world_x * 0.001, world_z * 0.001]) + 1.0) * 0.5;
    let humidity = (humid_noise.get([world_x * 0.001, world_z * 0.001]) + 1.0) * 0.5;

    registry
        .defs
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            let dist_a = (a.temperature - temperature).powi(2) + (a.humidity - humidity).powi(2);
            let dist_b = (b.temperature - temperature).powi(2) + (b.humidity - humidity).powi(2);
            dist_a.partial_cmp(&dist_b).unwrap()
        })
        .map(|(i, _)| i as BiomeId)
        .unwrap_or(0)
}
pub fn sample_biome_weights(
    world_x: f64,
    world_z: f64,
    registry: &BiomeRegistry,
    seed: u32,
    blend_distance: f64,
) -> Vec<(BiomeId, f64)> {
    let temp_noise = Perlin::new(seed);
    let humid_noise = Perlin::new(seed.wrapping_add(1));

    let temperature = (temp_noise.get([world_x * 0.001, world_z * 0.001]) + 1.0) * 0.5;
    let humidity = (humid_noise.get([world_x * 0.001, world_z * 0.001]) + 1.0) * 0.5;

    // get distance to each biome the climate
    let mut distances: Vec<(BiomeId, f64)> = registry
        .defs
        .iter()
        .enumerate()
        .map(|(i, def)| {
            let dist = ((def.temperature - temperature).powi(2)
                + (def.humidity - humidity).powi(2))
            .sqrt();
            (i as BiomeId, dist)
        })
        .collect();

    // sort by distance to get the closest and second closest
    distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let closest_dist = distances[0].1;
    let second_dist = if distances.len() > 1 {
        distances[1].1
    } else {
        f64::MAX
    };

    let blend_zone = second_dist - closest_dist;

    if blend_zone < 1e-10 || blend_zone >= blend_distance {
        // no blending needed
        return vec![(distances[0].0, 1.0)];
    }

    // smoothstep the blend amount so it eases
    let step = (blend_zone / blend_distance).min(1.0);
    let smooth_step = step * step * (3.0 - 2.0 * step);

    let dominant_weight = 0.5 + smooth_step * 0.5;
    let secondary_weight = 1.0 - dominant_weight;

    if secondary_weight < 0.01 {
        vec![(distances[0].0, 1.0)]
    } else {
        vec![
            (distances[0].0, dominant_weight),
            (distances[1].0, secondary_weight),
        ]
    }
}
