use apostasy_macros::Component;
use ash::vk::Buffer;
use ash::vk::{self, DeviceMemory};

use crate::rendering::shared::model::GpuMesh;
use crate::rendering::shared::vertex::VertexDefinition;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VoxelVertex {
    pub data: u64,
}

impl VertexDefinition for VoxelVertex {
    fn get_binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(std::mem::size_of::<VoxelVertex>() as u32)
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
}
#[derive(Debug, Component, Clone, Default)]
pub struct VoxelChunkMesh {
    pub vertex_buffer: Buffer,
    pub vertex_buffer_memory: DeviceMemory,
    pub index_buffer: Buffer,
    pub index_buffer_memory: DeviceMemory,
    pub index_count: u32,
}

impl GpuMesh for VoxelChunkMesh {
    fn get_vertex_buffer(&self) -> Buffer {
        self.vertex_buffer.clone()
    }
    fn get_index_buffer(&self) -> Buffer {
        self.index_buffer.clone()
    }
    fn get_index_count(&self) -> u32 {
        self.index_count
    }
}
