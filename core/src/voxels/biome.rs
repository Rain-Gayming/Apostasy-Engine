use apostasy_macros::Resource;
use hashbrown::HashMap;

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
}
