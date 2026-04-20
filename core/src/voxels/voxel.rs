use hashbrown::HashMap;

#[derive(Clone, Copy, Debug)]
pub struct Voxel {}

struct VoxelDefinition {
    name: String,
    namespace: String,
    class: String,
    // components: HashMap<String, Component>,
}
