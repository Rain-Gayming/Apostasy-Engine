use ash::vk;

use crate::app::engine::{
    renderer::{find_memory_type, swapchain::Swapchain},
    rendering_context::RenderingContext,
};

pub struct DepthImage {
    pub depth_format: vk::Format,
    pub depth_image: vk::Image,
    pub depth_image_memory: vk::DeviceMemory,
    pub depth_image_view: vk::ImageView,
}

pub fn new_depth_image(context: &RenderingContext, swapchain: &Swapchain) -> DepthImage {
    unsafe {
        let depth_format = vk::Format::D32_SFLOAT;

        let depth_image_create_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(depth_format)
            .extent(vk::Extent3D {
                width: swapchain.extent.width,
                height: swapchain.extent.height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .initial_layout(vk::ImageLayout::UNDEFINED);

        let depth_image = context
            .device
            .create_image(&depth_image_create_info, None)
            .unwrap();
        let mem_req = context.device.get_image_memory_requirements(depth_image);

        let memory_type = find_memory_type(
            mem_req.memory_type_bits,
            &context.physical_device.memory_properties,
        );

        let depth_alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_req.size)
            .memory_type_index(memory_type);

        let depth_image_memory = context
            .device
            .allocate_memory(&depth_alloc_info, None)
            .unwrap();
        context
            .device
            .bind_image_memory(depth_image, depth_image_memory, 0)
            .unwrap();

        let depth_image_view = context
            .create_image_view(depth_image, depth_format, vk::ImageAspectFlags::DEPTH)
            .unwrap();

        DepthImage {
            depth_format,
            depth_image,
            depth_image_memory,
            depth_image_view,
        }
    }
}
