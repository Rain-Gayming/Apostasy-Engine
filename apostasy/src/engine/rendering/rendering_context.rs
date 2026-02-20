use crate::{self as apostasy, engine::rendering::models::model::Texture};
use anyhow::{Result, anyhow};
use apostasy_macros::Resource;
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
    models::vertex::{Vertex, VertexDefinition, VoxelVertex},
    physical_device::PhysicalDevice,
    queue_families::{QueueFamilies, QueueFamily, QueueFamilyPicker},
    surface::Surface,
};

/// A set of data required for a renderer
#[derive(Resource)]
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
    /// Creates a new RenderingContext
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
                    .push_next(
                        &mut PhysicalDeviceDynamicRenderingFeatures::default()
                            .dynamic_rendering(true),
                    )
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

    /// Creates a vertex buffer from a slice of vertices
    pub fn create_vertex_buffer<T: VertexDefinition>(
        &self,
        vertices: &[T],
    ) -> Result<(vk::Buffer, vk::DeviceMemory)> {
        let buffer_size = (size_of::<T>() * vertices.len()) as vk::DeviceSize;

        let buffer_info = vk::BufferCreateInfo::default()
            .size(buffer_size)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe { self.device.create_buffer(&buffer_info, None)? };

        let mem_requirements = unsafe { self.device.get_buffer_memory_requirements(buffer) };

        let memory_type_index = self.find_memory_type(
            mem_requirements.memory_type_bits,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(memory_type_index);

        let buffer_memory = unsafe { self.device.allocate_memory(&alloc_info, None)? };

        unsafe {
            self.device.bind_buffer_memory(buffer, buffer_memory, 0)?;

            let data_ptr = self.device.map_memory(
                buffer_memory,
                0,
                buffer_size,
                vk::MemoryMapFlags::empty(),
            )? as *mut T;

            data_ptr.copy_from_nonoverlapping(vertices.as_ptr(), vertices.len());

            self.device.unmap_memory(buffer_memory);
        }

        Ok((buffer, buffer_memory))
    }

    /// Creates an index buffer from a slice of indices
    pub fn create_index_buffer(&self, indices: &[u32]) -> Result<(vk::Buffer, vk::DeviceMemory)> {
        let buffer_size = (std::mem::size_of::<u32>() * indices.len()) as vk::DeviceSize;

        let buffer_info = vk::BufferCreateInfo::default()
            .size(buffer_size)
            .usage(vk::BufferUsageFlags::INDEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe { self.device.create_buffer(&buffer_info, None)? };

        let mem_requirements = unsafe { self.device.get_buffer_memory_requirements(buffer) };

        let memory_type_index = self.find_memory_type(
            mem_requirements.memory_type_bits,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(memory_type_index);

        let buffer_memory = unsafe { self.device.allocate_memory(&alloc_info, None)? };

        unsafe {
            self.device.bind_buffer_memory(buffer, buffer_memory, 0)?;

            let data_ptr = self.device.map_memory(
                buffer_memory,
                0,
                buffer_size,
                vk::MemoryMapFlags::empty(),
            )? as *mut u32;

            data_ptr.copy_from_nonoverlapping(indices.as_ptr(), indices.len());

            self.device.unmap_memory(buffer_memory);
        }

        Ok((buffer, buffer_memory))
    }

    /// Creates a shader module
    pub fn create_shader_module(&self, code: &[u8]) -> Result<vk::ShaderModule> {
        unsafe {
            let mut code = std::io::Cursor::new(code);
            let code = ash::util::read_spv(&mut code)?;
            let create_info = vk::ShaderModuleCreateInfo::default().code(&code);
            let shader_module = self.device.create_shader_module(&create_info, None)?;
            Ok(shader_module)
        }
    }

    /// Creates a graphics pipeline for non voxel rendering
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::let_and_return)]
    pub fn create_model_graphics_pipeline(
        &self,
        vertex_shader: vk::ShaderModule,
        fragment_shader: vk::ShaderModule,
        format: vk::Format,
        depth_format: vk::Format,
        pipeline_layout: vk::PipelineLayout,
        pipeline_cache: vk::PipelineCache,
    ) -> Result<vk::Pipeline> {
        let entry_point = std::ffi::CString::new("main").unwrap();
        let bindings = vec![Vertex::get_binding_description()];
        let attributes = Vertex::get_attribute_descriptions();

        unsafe {
            let pipelines = self
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
                                .vertex_binding_descriptions(&bindings)
                                .vertex_attribute_descriptions(&attributes),
                        )
                        .input_assembly_state(
                            &vk::PipelineInputAssemblyStateCreateInfo::default()
                                .topology(vk::PrimitiveTopology::TRIANGLE_LIST),
                        )
                        .viewport_state(
                            &vk::PipelineViewportStateCreateInfo::default()
                                .scissor_count(1)
                                .viewport_count(1),
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
                .map_err(|e| anyhow!("Pipeline creation failed: {:?}", e))?;

            Ok(pipelines[0])
        }
    }

    /// Creates a graphics pipeline for voxel rendering
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::let_and_return)]
    pub fn create_voxel_graphics_pipeline(
        &self,
        vertex_shader: vk::ShaderModule,
        fragment_shader: vk::ShaderModule,
        format: vk::Format,
        depth_format: vk::Format,
        pipeline_layout: vk::PipelineLayout,
        pipeline_cache: vk::PipelineCache,
    ) -> Result<vk::Pipeline> {
        let entry_point = std::ffi::CString::new("main").unwrap();
        let bindings = vec![VoxelVertex::get_binding_description()];
        let attributes = VoxelVertex::get_attribute_descriptions();

        unsafe {
            let pipelines = self
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
                                .vertex_binding_descriptions(&bindings)
                                .vertex_attribute_descriptions(&attributes),
                        )
                        .input_assembly_state(
                            &vk::PipelineInputAssemblyStateCreateInfo::default()
                                .topology(vk::PrimitiveTopology::TRIANGLE_LIST),
                        )
                        .viewport_state(
                            &vk::PipelineViewportStateCreateInfo::default()
                                .scissor_count(1)
                                .viewport_count(1),
                        )
                        .rasterization_state(
                            &vk::PipelineRasterizationStateCreateInfo::default()
                                .polygon_mode(vk::PolygonMode::FILL)
                                .cull_mode(vk::CullModeFlags::BACK)
                                .front_face(vk::FrontFace::CLOCKWISE)
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
                .map_err(|e| anyhow!("Pipeline creation failed: {:?}", e))?;

            Ok(pipelines[0])
        }
    }

    /// Transitions from one layout to another
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

    /// Creates an image for rendering
    pub fn create_image(
        &self,
        extent: vk::Extent2D,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<(vk::Image, vk::DeviceMemory)> {
        unsafe {
            let image_info = vk::ImageCreateInfo::default()
                .image_type(vk::ImageType::TYPE_2D)
                .extent(vk::Extent3D {
                    width: extent.width,
                    height: extent.height,
                    depth: 1,
                })
                .mip_levels(1)
                .array_layers(1)
                .format(format)
                .tiling(tiling)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .usage(usage)
                .samples(vk::SampleCountFlags::TYPE_1)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let image = unsafe { self.device.create_image(&image_info, None).unwrap() };
            let mem_reqs = unsafe { self.device.get_image_memory_requirements(image) };

            let alloc_info = vk::MemoryAllocateInfo::default()
                .allocation_size(mem_reqs.size)
                .memory_type_index(self.find_memory_type(mem_reqs.memory_type_bits, properties)?);

            let memory = unsafe { self.device.allocate_memory(&alloc_info, None).unwrap() };
            unsafe { self.device.bind_image_memory(image, memory, 0).unwrap() };
            Ok((image, memory))
        }
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

    fn copy_buffer_to_image(
        &self,
        buffer: vk::Buffer,
        image: vk::Image,
        width: u32,
        height: u32,
        command_pool: vk::CommandPool,
        queue: vk::Queue,
    ) {
        let cmd_buf = self.begin_single_time_commands(command_pool);

        let region = vk::BufferImageCopy::default()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            })
            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            });

        unsafe {
            self.device.cmd_copy_buffer_to_image(
                cmd_buf,
                buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            );
        }

        self.end_single_time_commands(cmd_buf, queue, command_pool);
    }

    fn begin_single_time_commands(&self, command_pool: vk::CommandPool) -> vk::CommandBuffer {
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(command_pool)
            .command_buffer_count(1);

        let cmd_buf = unsafe { self.device.allocate_command_buffers(&alloc_info).unwrap()[0] };
        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.device
                .begin_command_buffer(cmd_buf, &begin_info)
                .unwrap()
        };
        cmd_buf
    }

    fn end_single_time_commands(
        &self,
        cmd_buf: vk::CommandBuffer,
        queue: vk::Queue,
        command_pool: vk::CommandPool,
    ) {
        unsafe {
            self.device.end_command_buffer(cmd_buf).unwrap();

            let buffer = &[cmd_buf];
            let submit_info = vk::SubmitInfo::default().command_buffers(buffer);
            self.device
                .queue_submit(queue, &[submit_info], vk::Fence::null())
                .unwrap();
            self.device.queue_wait_idle(queue).unwrap();
            self.device.free_command_buffers(command_pool, &[cmd_buf]);
        }
    }

    /// Allocates memory for an image
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

    pub fn create_sampler(&self) -> vk::Sampler {
        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(false)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR);

        unsafe { self.device.create_sampler(&sampler_info, None).unwrap() }
    }

    /// Creates a texture descriptor set
    pub fn create_texture_descriptor_set(
        &self,
        descriptor_pool: vk::DescriptorPool,
        descriptor_set_layout: vk::DescriptorSetLayout,
        image_view: vk::ImageView,
        sampler: vk::Sampler,
    ) -> vk::DescriptorSet {
        unsafe {
            let set = self
                .device
                .allocate_descriptor_sets(
                    &vk::DescriptorSetAllocateInfo::default()
                        .descriptor_pool(descriptor_pool)
                        .set_layouts(&[descriptor_set_layout]),
                )
                .unwrap()[0];

            let image_info = vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(image_view)
                .sampler(sampler);

            self.device.update_descriptor_sets(
                &[vk::WriteDescriptorSet::default()
                    .dst_set(set)
                    .dst_binding(1)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&[image_info])],
                &[],
            );

            set
        }
    }

    /// Loads a texture from a file and passes it to a buffer
    pub fn load_texture(
        &self,
        path: &str,
        command_buffer: vk::CommandBuffer,
        command_pool: vk::CommandPool,
        descriptor_pool: vk::DescriptorPool,
        descriptor_set_layout: vk::DescriptorSetLayout,
    ) -> Result<Texture> {
        let count = LOAD_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        println!("load_texture call #{} for path: {}", count, path);
        let path = format!("res/assets/textures/{}", path);

        // Load as u8 RGBA
        let image = image::open(&path).expect("Failed to load image").to_rgba8();
        let (width, height) = image.dimensions();
        let pixels: Vec<u8> = image.into_raw();
        let image_size = pixels.len() as vk::DeviceSize; // w * h * 4

        // Create staging buffer
        let (staging_buffer, staging_buffer_memory) = self.create_buffer(
            image_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        // Upload pixel data to staging buffer
        unsafe {
            let data_ptr = self.device.map_memory(
                staging_buffer_memory,
                0,
                image_size,
                vk::MemoryMapFlags::empty(),
            )? as *mut u8;
            std::ptr::copy_nonoverlapping(pixels.as_ptr(), data_ptr, pixels.len());
            self.device.unmap_memory(staging_buffer_memory);
        }

        let extent = vk::Extent2D { width, height };
        let (vk_image, image_memory) = self.create_image(
            extent,
            vk::Format::R8G8B8A8_SRGB, // use SRGB for color textures
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        let queue = self.queues[self.queue_families.graphics as usize];

        let undefined_image_state = ImageLayoutState {
            layout: vk::ImageLayout::UNDEFINED,
            access: vk::AccessFlags::empty(),
            stage: vk::PipelineStageFlags::TOP_OF_PIPE,
            queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        };

        let transfer_dst_optimal_image_state = ImageLayoutState {
            layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            access: vk::AccessFlags::TRANSFER_WRITE,
            stage: vk::PipelineStageFlags::TRANSFER,
            queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        };

        let shader_read_only_optimal_image_state = ImageLayoutState {
            layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            access: vk::AccessFlags::SHADER_READ,
            stage: vk::PipelineStageFlags::FRAGMENT_SHADER,
            queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        };

        // Prepare image to receive data
        self.transition_image_layout(
            command_buffer,
            vk_image,
            undefined_image_state,
            transfer_dst_optimal_image_state,
            vk::ImageAspectFlags::COLOR,
        );

        //  Copy staging buffer into image
        self.copy_buffer_to_image(staging_buffer, vk_image, width, height, command_pool, queue);

        //  Prepare image for shader reads
        self.transition_image_layout(
            command_buffer,
            vk_image,
            undefined_image_state,
            shader_read_only_optimal_image_state,
            vk::ImageAspectFlags::COLOR,
        );

        unsafe {
            self.device.destroy_buffer(staging_buffer, None);
            self.device.free_memory(staging_buffer_memory, None);
        }

        let image_view = self.create_image_view(
            vk_image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageAspectFlags::COLOR,
        )?;

        let sampler = self.create_sampler();

        let descriptor_set = self.create_texture_descriptor_set(
            descriptor_pool,
            descriptor_set_layout,
            image_view,
            sampler,
        );

        Ok(Texture {
            image: vk_image,
            image_view,
            memory: image_memory,
            sampler,
            descriptor_set,
        })
    }

    /// Begins the rendering process
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

    pub fn create_buffer(
        &self,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<(vk::Buffer, vk::DeviceMemory)> {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe { self.device.create_buffer(&buffer_info, None)? };
        let mem_requirements = unsafe { self.device.get_buffer_memory_requirements(buffer) };

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(
                self.find_memory_type(mem_requirements.memory_type_bits, properties)?,
            );

        let memory = unsafe { self.device.allocate_memory(&alloc_info, None)? };

        unsafe { self.device.bind_buffer_memory(buffer, memory, 0)? };

        Ok((buffer, memory))
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

/// The layout of an image
#[derive(Clone, Copy)]
pub struct ImageLayoutState {
    pub layout: vk::ImageLayout,
    pub access: vk::AccessFlags,
    pub stage: vk::PipelineStageFlags,
    pub queue_family_index: u32,
}
static LOAD_COUNT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
