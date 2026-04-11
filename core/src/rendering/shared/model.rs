use ash::vk::{Buffer, DeviceMemory};

#[derive(Clone, Debug)]
pub struct GpuModel {
    pub meshes: Vec<GpuMesh>,
    pub name: String,
}

#[derive(Debug, Clone, Default)]
pub struct GpuMesh {
    pub vertex_buffer: Buffer,
    pub vertex_buffer_memory: DeviceMemory,
    pub index_buffer: Buffer,
    pub index_buffer_memory: DeviceMemory,
    pub index_count: u32,
    pub material_name: String,
}

#[derive(Clone)]
pub struct ModelRenderer {
    pub loaded_model: String,
    // pub material_path: String,
}
