use anyhow::Result;
use ash::{
    khr::{surface, swapchain},
    vk::{
        self, ApplicationInfo, DeviceQueueCreateInfo, Image, ImageView, InstanceCreateInfo,
        PhysicalDeviceBufferDeviceAddressFeatures, PhysicalDeviceDynamicRenderingFeatures, Queue,
    },
};
use std::{collections::HashSet, sync::Arc};
use winit::{
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};

use crate::engine::rendering::{
    physical_device::PhysicalDevice,
    queue_families::{QueueFamilies, QueueFamily, QueueFamilyPicker},
    surface::Surface,
};

pub struct RenderingContext {
    pub queues: Vec<vk::Queue>,
    pub device: ash::Device,
    pub swapchain_extensions: ash::khr::swapchain::Device,
    pub queue_family_indices: HashSet<u32>,
    pub queue_families: QueueFamilies,
    pub physical_device: PhysicalDevice,
    pub surface_extensions: ash::khr::surface::Instance,
    pub instance: ash::Instance,
    pub entry: ash::Entry,
}
pub struct RenderingContextAttributes<'window> {
    pub compatability_window: &'window Window,
    pub queue_family_picker: QueueFamilyPicker,
}

impl RenderingContext {
    pub fn new(context_attributes: RenderingContextAttributes) -> Result<Self> {
        unsafe {
            let entry = ash::Entry::load()?;

            let raw_display_handle = context_attributes
                .compatability_window
                .display_handle()?
                .as_raw();
            let raw_window_handle = context_attributes
                .compatability_window
                .window_handle()?
                .as_raw();

            let instance = entry.create_instance(
                &InstanceCreateInfo::default()
                    .application_info(&ApplicationInfo::default().api_version(vk::API_VERSION_1_3))
                    .enabled_extension_names(ash_window::enumerate_required_extensions(
                        raw_display_handle,
                    )?),
                None,
            )?;

            let surface_extensions = surface::Instance::new(&entry, &instance);
            let compatability_surface = ash_window::create_surface(
                &entry,
                &instance,
                raw_display_handle,
                raw_window_handle,
                None,
            )?;

            let mut physical_devices = instance
                .enumerate_physical_devices()?
                .into_iter()
                .map(|handle| {
                    let properties = instance.get_physical_device_properties(handle);
                    let features = instance.get_physical_device_features(handle);
                    let memory_properties = instance.get_physical_device_memory_properties(handle);
                    let queue_families =
                        instance.get_physical_device_queue_family_properties(handle);

                    let queue_families = queue_families
                        .iter()
                        .cloned()
                        .enumerate()
                        .map(|(index, properties)| QueueFamily {
                            index: index as u32,
                            properties,
                        })
                        .collect::<Vec<QueueFamily>>();

                    PhysicalDevice {
                        handle,
                        properties,
                        features,
                        memory_properties,
                        queue_families,
                    }
                })
                .collect::<Vec<PhysicalDevice>>();

            physical_devices.retain(|device| {
                device.queue_families.iter().any(|qf| {
                    surface_extensions
                        .get_physical_device_surface_support(
                            device.handle,
                            qf.index,
                            compatability_surface,
                        )
                        .unwrap_or(false)
                })
            });

            surface_extensions.destroy_surface(compatability_surface, None);

            let (physical_device, queue_families) =
                (context_attributes.queue_family_picker)(physical_devices)?;

            let queue_family_indices: HashSet<u32> = HashSet::from_iter([
                queue_families.graphics,
                queue_families.present,
                queue_families.transfer,
                queue_families.compute,
            ]);

            let queue_create_infos = queue_family_indices
                .iter()
                .copied()
                .map(|index| {
                    DeviceQueueCreateInfo::default()
                        .queue_family_index(index)
                        .queue_priorities(&[1.0])
                })
                .collect::<Vec<_>>();

            let device = instance.create_device(
                physical_device.handle,
                &vk::DeviceCreateInfo::default()
                    .queue_create_infos(&queue_create_infos)
                    .enabled_extension_names(&[swapchain::NAME.as_ptr()])
                    .push_next(&mut PhysicalDeviceDynamicRenderingFeatures::default())
                    .push_next(
                        &mut PhysicalDeviceBufferDeviceAddressFeatures::default()
                            .buffer_device_address(true),
                    ),
                None,
            )?;

            let swapchain_extensions = ash::khr::swapchain::Device::new(&instance, &device);

            let queues = queue_family_indices
                .iter()
                .map(|index| device.get_device_queue(*index, 0))
                .collect::<Vec<Queue>>();

            Ok(Self {
                queues,
                device,
                swapchain_extensions,
                queue_family_indices,
                queue_families,
                physical_device,
                surface_extensions,
                instance,
                entry,
            })
        }
    }
    /// Safety: the window should outlive the surface
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn create_surface(&self, window: &Arc<Window>) -> Result<Surface> {
        unsafe {
            let raw_display_handle = window.display_handle()?.as_raw();
            let raw_window_handle = window.window_handle()?.as_raw();

            let handle = ash_window::create_surface(
                &self.entry,
                &self.instance,
                raw_display_handle,
                raw_window_handle,
                None,
            )?;

            let capabilities = self
                .surface_extensions
                .get_physical_device_surface_capabilities(self.physical_device.handle, handle)?;

            Ok(Surface {
                handle,
                capabilities,
            })
        }
    }

    pub fn create_image(
        &self,
        extent: vk::Extent2D,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
    ) -> Result<vk::Image> {
        unsafe {
            let image = self.device.create_image(
                &vk::ImageCreateInfo::default()
                    .image_type(vk::ImageType::TYPE_2D)
                    .format(format)
                    .extent(vk::Extent3D {
                        width: extent.width,
                        height: extent.height,
                        depth: 1,
                    })
                    .mip_levels(1)
                    .array_layers(1)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .tiling(vk::ImageTiling::OPTIMAL)
                    .usage(usage)
                    .sharing_mode(vk::SharingMode::EXCLUSIVE)
                    .initial_layout(vk::ImageLayout::UNDEFINED),
                None,
            )?;
            Ok(image)
        }
    }

    pub fn allocate_image_memory(
        &self,
        image: vk::Image,
        memory_properties: vk::MemoryPropertyFlags,
    ) -> Result<vk::DeviceMemory> {
        unsafe {
            let requirements = self.device.get_image_memory_requirements(image);
            let memory_type_index =
                self.find_memory_type(requirements.memory_type_bits, memory_properties)?;
            let memory = self.device.allocate_memory(
                &vk::MemoryAllocateInfo::default()
                    .allocation_size(requirements.size)
                    .memory_type_index(memory_type_index),
                None,
            )?;
            self.device.bind_image_memory(image, memory, 0)?;
            Ok(memory)
        }
    }

    pub fn find_memory_type(
        &self,
        filter: u32,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<u32> {
        for i in 0..self.physical_device.memory_properties.memory_type_count {
            if (filter & (1 << i)) != 0
                && (self.physical_device.memory_properties.memory_types[i as usize].property_flags
                    & properties)
                    == properties
            {
                return Ok(i);
            }
        }
        Err(anyhow::anyhow!("Failed to find suitable memory type"))
    }

    pub fn create_image_view(
        &self,
        image: Image,
        format: vk::Format,
        aspect_flags: vk::ImageAspectFlags,
    ) -> Result<ImageView> {
        unsafe {
            let image = self.device.create_image_view(
                &vk::ImageViewCreateInfo::default()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::IDENTITY,
                        g: vk::ComponentSwizzle::IDENTITY,
                        b: vk::ComponentSwizzle::IDENTITY,
                        a: vk::ComponentSwizzle::IDENTITY,
                    })
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: aspect_flags,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    }),
                None,
            )?;
            Ok(image)
        }
    }

    pub fn create_shader_module(&self, code: &[u8]) -> Result<vk::ShaderModule> {
        unsafe {
            let mut code = std::io::Cursor::new(code);
            let code = ash::util::read_spv(&mut code)?;
            let create_info = vk::ShaderModuleCreateInfo::default().code(&code);
            let shader_module = self.device.create_shader_module(&create_info, None)?;
            Ok(shader_module)
        }
    }

    pub fn create_graphics_pipeline(
        &self,
        vertex_shader: vk::ShaderModule,
        fragment_shader: vk::ShaderModule,
        extent: vk::Extent2D,
        format: vk::Format,
        depth_format: vk::Format,
        pipeline_layout: vk::PipelineLayout,
        pipeline_cache: vk::PipelineCache,
    ) -> Result<vk::Pipeline> {
        let entry_point = std::ffi::CString::new("main").unwrap();

        unsafe {
            Ok(self
                .device
                .create_graphics_pipelines(
                    pipeline_cache,
                    &[vk::GraphicsPipelineCreateInfo::default()
                        .stages(&[
                            vk::PipelineShaderStageCreateInfo::default()
                                .stage(vk::ShaderStageFlags::VERTEX)
                                .module(vertex_shader)
                                .name(&entry_point),
                            vk::PipelineShaderStageCreateInfo::default()
                                .stage(vk::ShaderStageFlags::FRAGMENT)
                                .module(fragment_shader)
                                .name(&entry_point),
                        ])
                        .vertex_input_state(
                            &vk::PipelineVertexInputStateCreateInfo::default()
                                .vertex_binding_descriptions(&[])
                                .vertex_attribute_descriptions(&[]),
                        )
                        .input_assembly_state(
                            &vk::PipelineInputAssemblyStateCreateInfo::default()
                                .topology(vk::PrimitiveTopology::TRIANGLE_LIST),
                        )
                        .viewport_state(
                            &vk::PipelineViewportStateCreateInfo::default()
                                .viewports(&[vk::Viewport::default()
                                    .width(extent.width as f32)
                                    .height(extent.height as f32)
                                    .max_depth(2.0)])
                                .scissors(&[vk::Rect2D::default().extent(extent)]),
                        )
                        .rasterization_state(
                            &vk::PipelineRasterizationStateCreateInfo::default()
                                .polygon_mode(vk::PolygonMode::FILL)
                                .cull_mode(vk::CullModeFlags::BACK)
                                .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
                                .line_width(1.0),
                        )
                        .multisample_state(
                            &vk::PipelineMultisampleStateCreateInfo::default()
                                .rasterization_samples(vk::SampleCountFlags::TYPE_1),
                        )
                        .color_blend_state(
                            &vk::PipelineColorBlendStateCreateInfo::default().attachments(&[
                                vk::PipelineColorBlendAttachmentState::default()
                                    .color_write_mask(vk::ColorComponentFlags::RGBA)
                                    .blend_enable(false),
                            ]),
                        )
                        .depth_stencil_state(
                            &vk::PipelineDepthStencilStateCreateInfo::default()
                                .depth_test_enable(true)
                                .depth_write_enable(true)
                                .depth_compare_op(vk::CompareOp::LESS),
                        )
                        .dynamic_state(
                            &vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&[
                                vk::DynamicState::VIEWPORT,
                                vk::DynamicState::SCISSOR,
                            ]),
                        )
                        .layout(pipeline_layout)
                        .push_next(
                            &mut vk::PipelineRenderingCreateInfo::default()
                                .color_attachment_formats(&[format])
                                .depth_attachment_format(depth_format),
                        )],
                    None,
                )
                .unwrap()
                .into_iter()
                .next()
                .unwrap())
        }
    }

    pub fn transition_image_layout(
        &self,
        command_buffer: vk::CommandBuffer,
        image: vk::Image,
        old_layout: ImageLayoutState,
        new_layout: ImageLayoutState,
        aspect_flags: vk::ImageAspectFlags,
    ) {
        unsafe {
            self.device.cmd_pipeline_barrier(
                command_buffer,
                old_layout.stage,
                new_layout.stage,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[vk::ImageMemoryBarrier::default()
                    .old_layout(old_layout.layout)
                    .new_layout(new_layout.layout)
                    .image(image)
                    .src_access_mask(old_layout.access)
                    .dst_access_mask(new_layout.access)
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: aspect_flags,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    })],
            );
        }
    }

    pub fn begin_rendering(
        &self,
        command_buffer: vk::CommandBuffer,
        view: vk::ImageView,
        depth_view: vk::ImageView,
        clear_value: vk::ClearColorValue,
        render_area: vk::Rect2D,
    ) -> Result<()> {
        unsafe {
            self.device.cmd_begin_rendering(
                command_buffer,
                &vk::RenderingInfo::default()
                    .layer_count(1)
                    .color_attachments(&[vk::RenderingAttachmentInfo::default()
                        .image_view(view)
                        .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                        .clear_value(vk::ClearValue { color: clear_value })
                        .load_op(vk::AttachmentLoadOp::CLEAR)
                        .store_op(vk::AttachmentStoreOp::STORE)])
                    .depth_attachment(
                        &vk::RenderingAttachmentInfo::default()
                            .image_view(depth_view)
                            .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
                            .clear_value(vk::ClearValue {
                                depth_stencil: vk::ClearDepthStencilValue {
                                    depth: 1.0,
                                    stencil: 0,
                                },
                            })
                            .load_op(vk::AttachmentLoadOp::CLEAR)
                            .store_op(vk::AttachmentStoreOp::STORE),
                    )
                    .render_area(render_area),
            );
        }
        Ok(())
    }
}

impl Drop for RenderingContext {
    fn drop(&mut self) {
        unsafe {
            // self.device.destory_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

#[derive(Clone, Copy)]
pub struct ImageLayoutState {
    pub layout: vk::ImageLayout,
    pub access: vk::AccessFlags,
    pub stage: vk::PipelineStageFlags,
    pub queue_family_index: u32,
}
