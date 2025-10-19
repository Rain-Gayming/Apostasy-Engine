use std::{collections::HashSet, io};

use anyhow::Result;
use ash::{
    vk::{self, RenderPass},
    Entry,
};
use winit::{
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};

pub struct RenderingContext {
    pub queues: Vec<vk::Queue>,
    pub device: ash::Device,
    pub queue_family_indices: HashSet<u32>,
    pub queue_families: QueueFamilies,
    pub physical_device: PhysicalDevice,
    pub surface_extension: ash::khr::surface::Instance,
    pub instance: ash::Instance,
    pub entry: Entry,
    pub swapchain_extension: ash::khr::swapchain::Device,
}

#[derive(Debug, Clone)]
pub struct QueueFamily {
    pub index: u32,
    pub properties: vk::QueueFamilyProperties,
}

#[derive(Debug)]
pub struct PhysicalDevice {
    pub handle: vk::PhysicalDevice,
    pub properties: vk::PhysicalDeviceProperties,
    pub features: vk::PhysicalDeviceFeatures,
    pub memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub queue_families: Vec<QueueFamily>,
}

pub type QueueFamilyPicker = fn(Vec<PhysicalDevice>) -> Result<(PhysicalDevice, QueueFamilies)>;

pub struct RenderingContextAttributes<'window> {
    pub compatability_window: &'window Window,
    pub queue_family_picker: QueueFamilyPicker,
}

#[derive(Debug, Clone)]
pub struct QueueFamilies {
    pub graphics: u32,
    pub present: u32,
    pub transfer: u32,
    pub compute: u32,
}

pub mod queue_family_picker {
    use anyhow::Context as AnyHowContext;
    use anyhow::Result;
    use ash::vk::QueueFlags;

    use super::{PhysicalDevice, QueueFamilies};

    pub fn single_queue_family(
        physical_devices: Vec<PhysicalDevice>,
    ) -> Result<(PhysicalDevice, QueueFamilies)> {
        let physical_device = physical_devices.into_iter().next().unwrap();
        let queue_family = physical_device
            .queue_families
            .iter()
            .find(|qf| {
                qf.properties.queue_flags.contains(QueueFlags::GRAPHICS)
                    && qf.properties.queue_flags.contains(QueueFlags::COMPUTE)
            })
            .context("Failed to find a queue family that matches conditions: QueueFlag::Graphics and QueueFlags::Compute")?
            .clone();

        Ok((
            physical_device,
            QueueFamilies {
                graphics: queue_family.index,
                present: queue_family.index,
                transfer: queue_family.index,
                compute: queue_family.index,
            },
        ))
    }
}

impl RenderingContext {
    pub fn new(attributes: RenderingContextAttributes) -> Result<Self> {
        unsafe {
            let entry = ash::Entry::load()?;

            let raw_display_handle = attributes.compatability_window.display_handle()?.as_raw();
            let raw_window_handle = attributes.compatability_window.window_handle()?.as_raw();

            let extensions = ash_window::enumerate_required_extensions(raw_display_handle)?;

            let instance = entry.create_instance(
                &vk::InstanceCreateInfo::default()
                    .application_info(
                        &vk::ApplicationInfo::default().api_version(vk::API_VERSION_1_3),
                    )
                    .enabled_extension_names(extensions),
                None,
            )?;

            let surface_extension = ash::khr::surface::Instance::new(&entry, &instance);
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
                    let queue_family_properties =
                        instance.get_physical_device_queue_family_properties(handle);

                    let queue_families = queue_family_properties
                        .into_iter()
                        .enumerate()
                        .map(|(index, properties)| QueueFamily {
                            index: index as u32,
                            properties,
                        })
                        .collect::<Vec<_>>();

                    PhysicalDevice {
                        handle,
                        properties,
                        features,
                        memory_properties,
                        queue_families,
                    }
                })
                .collect::<Vec<_>>();

            physical_devices.retain(|device| {
                surface_extension
                    .get_physical_device_surface_support(device.handle, 0, compatability_surface)
                    .unwrap_or(false)
            });

            surface_extension.destroy_surface(compatability_surface, None);

            let (physical_device, queue_family) =
                (attributes.queue_family_picker)(physical_devices)?;

            let queue_family_indices = HashSet::from([
                queue_family.graphics,
                queue_family.present,
                queue_family.transfer,
                queue_family.compute,
            ]);

            let queue_create_infos = queue_family_indices
                .iter()
                .copied()
                .map(|index| {
                    vk::DeviceQueueCreateInfo::default()
                        .queue_family_index(index)
                        .queue_priorities(&[1.0])
                })
                .collect::<Vec<_>>();

            let device = instance.create_device(
                physical_device.handle,
                &vk::DeviceCreateInfo::default()
                    .queue_create_infos(&queue_create_infos)
                    .enabled_extension_names(&[ash::khr::swapchain::NAME.as_ptr()])
                    .enabled_features(
                        &vk::PhysicalDeviceFeatures::default()
                            .depth_bias_clamp(true)
                            .depth_clamp(true)
                            .fill_mode_non_solid(true),
                    )
                    .push_next(
                        &mut vk::PhysicalDeviceDynamicRenderingFeatures::default()
                            .dynamic_rendering(true),
                    )
                    .push_next(
                        &mut vk::PhysicalDeviceBufferDeviceAddressFeatures::default()
                            .buffer_device_address(true),
                    ),
                None,
            )?;

            let swapchain_extension = ash::khr::swapchain::Device::new(&instance, &device);

            let queues = queue_family_indices
                .iter()
                .map(|&index| device.get_device_queue(index, 0))
                .collect::<Vec<_>>();

            Ok(Self {
                queues,
                device,
                queue_family_indices,
                queue_families: queue_family,
                physical_device,
                surface_extension,
                instance,
                entry,
                swapchain_extension,
            })
        }
    }

    #[allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc)]
    pub unsafe fn create_surface(&self, window: &Window) -> Result<Surface> {
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
            .surface_extension
            .get_physical_device_surface_capabilities(self.physical_device.handle, handle)?;

        let formats = self
            .surface_extension
            .get_physical_device_surface_formats(self.physical_device.handle, handle)?;

        let present_modes = self
            .surface_extension
            .get_physical_device_surface_present_modes(self.physical_device.handle, handle)?;

        Ok(Surface {
            handle,
            capabilities,
            formats,
            present_modes,
        })
    }

    pub fn create_image_view(
        &self,
        image: vk::Image,
        format: vk::Format,
        aspect_flags: vk::ImageAspectFlags,
    ) -> Result<vk::ImageView> {
        let image_view = unsafe {
            self.device.create_image_view(
                &vk::ImageViewCreateInfo::default()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(format)
                    .subresource_range(
                        vk::ImageSubresourceRange::default()
                            .aspect_mask(aspect_flags)
                            .base_mip_level(0)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1),
                    ),
                None,
            )
        }?;
        Ok(image_view)
    }

    pub fn create_shader_module(&self, code: &[u8]) -> Result<vk::ShaderModule, vk::Result> {
        let mut code = io::Cursor::new(code);
        let code = ash::util::read_spv(&mut code).unwrap();
        let create_info = vk::ShaderModuleCreateInfo::default().code(&code);
        let shader_module = unsafe { self.device.create_shader_module(&create_info, None) }?;
        Ok(shader_module)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_graphics_pipeline(
        &self,
        vertex_shader: vk::ShaderModule,
        fragment_shader: vk::ShaderModule,
        image_extent: vk::Extent2D,
        image_format: vk::Format,
        pipeline_layout: vk::PipelineLayout,
        pipeline_chache: vk::PipelineCache,
        depth_format: Option<vk::Format>,
    ) -> Result<vk::Pipeline> {
        let entry_point = std::ffi::CString::new("main").unwrap();

        // keep all objects alive as locals so pointers passed to Vulkan are valid
        let shader_stages = vec![
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vertex_shader)
                .name(&entry_point),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragment_shader)
                .name(&entry_point),
        ];

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: image_extent.width as f32,
            height: image_extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: image_extent,
        };
        let viewports = [viewport];
        let scissors = [scissor];

        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewports(&viewports)
            .scissors(&scissors);

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::default()
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::LINE)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::CLOCKWISE)
            .line_width(1.0);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false);

        let color_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false);

        let binding = [color_attachment];
        let color_blend_state =
            vk::PipelineColorBlendStateCreateInfo::default().attachments(&binding);

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let color_attachment_formats = [image_format];
        let mut pipeline_rendering = vk::PipelineRenderingCreateInfo::default()
            .color_attachment_formats(&color_attachment_formats);

        if let Some(df) = depth_format {
            pipeline_rendering = pipeline_rendering.depth_attachment_format(df);
        }

        let binding = &[ash::vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<u32>() as u32,
            input_rate: ash::vk::VertexInputRate::VERTEX,
        }];
        let vertex_input_attribute_descriptions = [ash::vk::VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: ash::vk::Format::R32_SINT,
            offset: 0,
        }];

        let vertex_input_state = ash::vk::PipelineVertexInputStateCreateInfo {
            vertex_binding_description_count: 1,
            p_vertex_binding_descriptions: binding.as_ptr(),
            vertex_attribute_description_count: vertex_input_attribute_descriptions.len() as u32,
            p_vertex_attribute_descriptions: vertex_input_attribute_descriptions.as_ptr(),
            ..Default::default()
        };

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS);

        unsafe {
            let pipeline_create_info = vk::GraphicsPipelineCreateInfo::default()
                .stages(&shader_stages)
                .vertex_input_state(&vertex_input_state)
                .input_assembly_state(&input_assembly_state)
                .viewport_state(&viewport_state)
                .rasterization_state(&rasterization_state)
                .multisample_state(&multisample_state)
                .color_blend_state(&color_blend_state)
                .render_pass(RenderPass::null())
                .dynamic_state(&dynamic_state)
                .depth_stencil_state(&depth_stencil_state)
                .push_next(&mut pipeline_rendering)
                .layout(pipeline_layout);

            let pipelines = self
                .device
                .create_graphics_pipelines(pipeline_chache, &[pipeline_create_info], None)
                .map_err(|(e, _)| anyhow::anyhow!("failed to create graphics pipeline: {e:?}"))?;

            let pipeline = pipelines
                .into_iter()
                .next()
                .ok_or_else(|| anyhow::anyhow!("failed to get a pipeline"))?;

            Ok(pipeline)
        }
    }

    pub fn transition_image_layout(
        &self,
        command_buffer: vk::CommandBuffer,
        image: vk::Image,
        old_state: ImageLayoutState,
        new_state: ImageLayoutState,
        aspect_mask: vk::ImageAspectFlags,
    ) {
        let image_memory_barrier = vk::ImageMemoryBarrier::default()
            .old_layout(old_state.layout)
            .new_layout(new_state.layout)
            .image(image)
            .src_access_mask(old_state.access_mask)
            .dst_access_mask(new_state.access_mask)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(aspect_mask)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1),
            );

        unsafe {
            self.device.cmd_pipeline_barrier(
                command_buffer,
                old_state.stage_mask,
                new_state.stage_mask,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[image_memory_barrier],
            );
        }
    }

    pub fn begin_rendering(
        &self,
        command_buffer: vk::CommandBuffer,
        view: vk::ImageView,
        clear_color: vk::ClearColorValue,
        render_area: vk::Rect2D,
        depth_view: vk::ImageView,
        depth_clear: vk::ClearDepthStencilValue,
    ) {
        unsafe {
            let color_attachment = vk::RenderingAttachmentInfo::default()
                .image_view(view)
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .clear_value(vk::ClearValue { color: clear_color })
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE);

            let color_attachments = [color_attachment];

            let depth_attachment_storage = vk::RenderingAttachmentInfo::default()
                .image_view(depth_view)
                .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
                .clear_value(vk::ClearValue {
                    depth_stencil: depth_clear,
                })
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE);

            let rendering_info = vk::RenderingInfo::default()
                .layer_count(1)
                .color_attachments(&color_attachments)
                .render_area(render_area)
                .depth_attachment(&depth_attachment_storage);

            self.device
                .cmd_begin_rendering(command_buffer, &rendering_info);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ImageLayoutState {
    pub access_mask: vk::AccessFlags,
    pub layout: vk::ImageLayout,
    pub stage_mask: vk::PipelineStageFlags,
    pub queue_family_index: u32,
}

#[derive(Clone)]
pub struct Surface {
    pub handle: vk::SurfaceKHR,
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl Drop for RenderingContext {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}
