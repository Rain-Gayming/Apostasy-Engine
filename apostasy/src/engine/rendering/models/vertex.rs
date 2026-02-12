use ash::vk;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VertexType {
    Model,
    Voxel,
}

#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coord: [f32; 2],
}
impl VertexDefinition for Vertex {
    fn get_binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(std::mem::size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
    }

    fn get_attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            // Position
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(0),
            // Normal
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(12),
            // Tex Coord
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(2)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(24),
        ]
    }

    fn get_vertex_type() -> VertexType {
        VertexType::Model
    }
}

#[repr(C)]
pub struct VoxelVertex {
    pub data: u32,
}

impl VertexDefinition for VoxelVertex {
    fn get_binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(1)
            .stride(std::mem::size_of::<VoxelVertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
    }
    fn get_attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription::default()
                .binding(1)
                .location(0)
                .format(vk::Format::R32_UINT)
                .offset(0),
        ]
    }
    fn get_vertex_type() -> VertexType {
        VertexType::Voxel
    }
}

pub trait VertexDefinition {
    fn get_binding_description() -> vk::VertexInputBindingDescription;
    fn get_attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription>;
    fn get_vertex_type() -> VertexType;
}
