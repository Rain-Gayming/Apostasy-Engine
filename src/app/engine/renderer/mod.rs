use std::os::raw::c_void;
use std::sync::{Arc, Mutex};
use std::thread::current;

pub mod camera;
mod swapchain;
pub mod voxel_vertex;

use anyhow::{Ok, Result};
use ash::vk::{
    self, Buffer, BufferCreateInfo, BufferUsageFlags, ClearColorValue, DescriptorSetLayoutBinding,
    DescriptorSetLayoutCreateInfo, MemoryAllocateInfo, MemoryPropertyFlags,
    PhysicalDeviceMemoryProperties, SharingMode,
};
use cgmath::Vector3;
use winit::window::Window;

use crate::app::engine::renderer::camera::{get_perspective_projection, get_view_matrix, Camera};
use crate::app::engine::renderer::swapchain::Swapchain;
use crate::app::engine::renderer::voxel_vertex::VoxelVertex;
use crate::app::engine::rendering_context;
use crate::app::engine::{
    renderer::rendering_context::RenderingContext, rendering_context::ImageLayoutState,
};

struct Frame {
    command_buffer: ash::vk::CommandBuffer,
    image_available_semaphore: ash::vk::Semaphore,
    render_finished_semaphore: ash::vk::Semaphore,
    in_flight_fence: ash::vk::Fence,
}

pub struct Renderer {
    pub in_flight_frames_count: usize,
    current_frame: usize,
    frames: Vec<Frame>,
    _command_pool: ash::vk::CommandPool,
    pipeline: ash::vk::Pipeline,
    pipeline_layout: ash::vk::PipelineLayout,
    swapchain: Swapchain,
    pub context: Arc<RenderingContext>,
    camera: Arc<Mutex<Camera>>,
    depth_format: vk::Format,
    depth_image: vk::Image,
    depth_image_memory: vk::DeviceMemory,
    depth_image_view: vk::ImageView,
    descriptor_sets: Vec<vk::DescriptorSet>,
    vertex_buffers: Vec<Buffer>,
    index_buffers: Vec<Buffer>,
    index_counts: Vec<u32>,
    uniform_buffers: Vec<Buffer>,
    index_offset: Vec<[i32; 3]>,
}

use std::fs::{self};

const SHADER_DIR: &str = "res/shaders/";

impl Renderer {
    pub fn new(
        context: Arc<RenderingContext>,
        window: Arc<Window>,
        camera: Arc<Mutex<Camera>>,
    ) -> Result<Self> {
        let mut swapchain = Swapchain::new(Arc::clone(&context), window)?;
        swapchain.resize()?;

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

        let vertex_shader = load_shader_module(&context, "vert.spv")?;
        let fragment_shader = load_shader_module(&context, "frag.spv")?;

        unsafe {
            let depth_image = context
                .device
                .create_image(&depth_image_create_info, None)?;
            let mem_req = context.device.get_image_memory_requirements(depth_image);

            let memory_type = find_memory_type(
                mem_req.memory_type_bits,
                &context.physical_device.memory_properties,
            );

            let depth_alloc_info = vk::MemoryAllocateInfo::default()
                .allocation_size(mem_req.size)
                .memory_type_index(memory_type);
            let depth_image_memory = context.device.allocate_memory(&depth_alloc_info, None)?;
            context
                .device
                .bind_image_memory(depth_image, depth_image_memory, 0)?;

            let depth_image_view = context.create_image_view(
                depth_image,
                depth_format,
                vk::ImageAspectFlags::DEPTH,
            )?;

            let push_constant_range = vk::PushConstantRange {
                stage_flags: vk::ShaderStageFlags::VERTEX,
                offset: 0,
                size: (std::mem::size_of::<[[f32; 4]; 4]>() * 2) as u32
                    + size_of::<[i32; 3]>() as u32,
            };

            let ubo_layout_binding = DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX);

            let ubo_layout_output = &[ubo_layout_binding];
            let ubo_layout_create_info =
                DescriptorSetLayoutCreateInfo::default().bindings(ubo_layout_output);
            let ubo_layout = context
                .device
                .create_descriptor_set_layout(&ubo_layout_create_info, None)?;

            let pipeline_layout = context.device.create_pipeline_layout(
                &ash::vk::PipelineLayoutCreateInfo::default()
                    .push_constant_ranges(&[push_constant_range])
                    .set_layouts(&[ubo_layout]),
                None,
            )?;

            let pipeline = context.create_graphics_pipeline(
                vertex_shader,
                fragment_shader,
                swapchain.extent,
                swapchain.format,
                pipeline_layout,
                Default::default(),
                Some(depth_format),
            )?;

            context.device.destroy_shader_module(vertex_shader, None);
            context.device.destroy_shader_module(fragment_shader, None);

            let command_pool = context.device.create_command_pool(
                &ash::vk::CommandPoolCreateInfo::default()
                    .queue_family_index(context.queue_families.graphics)
                    .flags(ash::vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
                None,
            )?;

            let in_flight_frames_count = 2;

            let command_buffers = context.device.allocate_command_buffers(
                &ash::vk::CommandBufferAllocateInfo::default()
                    .command_pool(command_pool)
                    .level(ash::vk::CommandBufferLevel::PRIMARY)
                    .command_buffer_count(in_flight_frames_count as u32),
            )?;

            let mut frames = Vec::with_capacity(command_buffers.len());
            for command_buffer in command_buffers.into_iter() {
                let image_available_semaphore = context
                    .device
                    .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)?;
                let render_finished_semaphore = context
                    .device
                    .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)?;
                let in_flight_fence = context.device.create_fence(
                    &vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED),
                    None,
                )?;

                frames.push(Frame {
                    command_buffer,
                    image_available_semaphore,
                    render_finished_semaphore,
                    in_flight_fence,
                });
            }

            // Bind the vertex buffer memory

            Ok(Self {
                in_flight_frames_count,
                current_frame: 0,
                frames,
                _command_pool: command_pool,
                pipeline,
                pipeline_layout,
                context,
                swapchain,
                camera,
                depth_format,
                depth_image,
                depth_image_memory,
                depth_image_view,
                descriptor_sets: Vec::new(),
                vertex_buffers: Vec::new(),
                index_counts: Vec::new(),
                uniform_buffers: Vec::new(),
                index_buffers: Vec::new(),
                index_offset: Vec::new(),
            })
        }
    }

    pub fn update_depth_buffer(&mut self) -> Result<()> {
        let depth_format = vk::Format::D32_SFLOAT;

        let depth_image_create_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(depth_format)
            .extent(vk::Extent3D {
                width: self.swapchain.extent.width,
                height: self.swapchain.extent.height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .initial_layout(vk::ImageLayout::UNDEFINED);
        unsafe {
            let depth_image = self
                .context
                .device
                .create_image(&depth_image_create_info, None)?;
            let mem_req = self
                .context
                .device
                .get_image_memory_requirements(depth_image);
            fn find_memory_type(
                type_bits: u32,
                props: vk::MemoryPropertyFlags,
                mem_props: &vk::PhysicalDeviceMemoryProperties,
            ) -> Option<u32> {
                for (i, mt) in mem_props.memory_types.iter().enumerate() {
                    if (type_bits & (1 << i)) != 0 && mt.property_flags.contains(props) {
                        return Some(i as u32);
                    }
                }
                None
            }
            let memory_type = find_memory_type(
                mem_req.memory_type_bits,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                &self.context.physical_device.memory_properties,
            )
            .ok_or_else(|| anyhow::anyhow!("No suitable memory type for depth image"))?;

            let depth_alloc_info = vk::MemoryAllocateInfo::default()
                .allocation_size(mem_req.size)
                .memory_type_index(memory_type);
            let depth_image_memory = self
                .context
                .device
                .allocate_memory(&depth_alloc_info, None)?;
            self.context
                .device
                .bind_image_memory(depth_image, depth_image_memory, 0)?;

            let depth_image_view = self.context.create_image_view(
                depth_image,
                self.depth_format,
                vk::ImageAspectFlags::DEPTH,
            )?;
            self.depth_format = depth_format;
            self.depth_image = depth_image;
            self.depth_image_memory = depth_image_memory;
            self.depth_image_view = depth_image_view;

            Ok(())
        }
    }
    pub fn resize(&mut self) -> Result<()> {
        let result = self.swapchain.resize();
        let _ = self.update_depth_buffer();
        result
    }

    pub fn render(&mut self) -> Result<()> {
        let frame = &self.frames[self.current_frame];
        unsafe {
            self.context
                .device
                .wait_for_fences(&[frame.in_flight_fence], true, u64::MAX)?;

            let image_index = self
                .swapchain
                .aquire_next_image(frame.image_available_semaphore)?;

            self.context.device.reset_fences(&[frame.in_flight_fence])?;

            self.context.device.reset_command_buffer(
                frame.command_buffer,
                ash::vk::CommandBufferResetFlags::empty(),
            )?;

            self.context.device.begin_command_buffer(
                frame.command_buffer,
                &ash::vk::CommandBufferBeginInfo::default(),
            )?;

            let undefined_image_state = ImageLayoutState {
                layout: vk::ImageLayout::UNDEFINED,
                access_mask: vk::AccessFlags::empty(),
                stage_mask: vk::PipelineStageFlags::TOP_OF_PIPE,
                queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            };
            let renderable_state = ImageLayoutState {
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            };

            let present_image_state = ImageLayoutState {
                layout: vk::ImageLayout::PRESENT_SRC_KHR,
                access_mask: vk::AccessFlags::empty(),
                stage_mask: vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            };

            let undefined_depth_state = ImageLayoutState {
                layout: vk::ImageLayout::UNDEFINED,
                access_mask: vk::AccessFlags::empty(),
                stage_mask: vk::PipelineStageFlags::TOP_OF_PIPE,
                queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            };
            let depth_attach_state = ImageLayoutState {
                layout: vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
                access_mask: vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                stage_mask: vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS
                    | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            };

            self.context.transition_image_layout(
                frame.command_buffer,
                self.depth_image,
                undefined_depth_state,
                depth_attach_state,
                vk::ImageAspectFlags::DEPTH,
            );

            self.context.transition_image_layout(
                frame.command_buffer,
                self.swapchain.images[image_index as usize],
                undefined_image_state,
                renderable_state,
                vk::ImageAspectFlags::COLOR,
            );

            self.context.begin_rendering(
                frame.command_buffer,
                self.swapchain.views[image_index as usize],
                ClearColorValue {
                    float32: [0.01, 0.01, 0.01, 1.0],
                },
                vk::Rect2D::default().extent(self.swapchain.extent),
                self.depth_image_view,
                vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            );

            self.context.device.cmd_set_viewport(
                frame.command_buffer,
                0,
                &[vk::Viewport::default()
                    .width(self.swapchain.extent.width as f32)
                    .height(self.swapchain.extent.height as f32)
                    .min_depth(0.0)
                    .max_depth(1.0)],
            );

            self.context.device.cmd_set_scissor(
                frame.command_buffer,
                0,
                &[vk::Rect2D::default().extent(self.swapchain.extent)],
            );

            let view: [[f32; 4]; 4] = get_view_matrix(self.camera.clone()).into();
            let view_bytes = std::slice::from_raw_parts(
                &view as *const [[f32; 4]; 4] as *const u8,
                std::mem::size_of::<[[f32; 4]; 4]>(),
            );

            let aspect = self.swapchain.extent.width as f32 / self.swapchain.extent.height as f32;
            let projection: [[f32; 4]; 4] =
                get_perspective_projection(self.camera.clone(), aspect).into();
            let projection_bytes = std::slice::from_raw_parts(
                &projection as *const [[f32; 4]; 4] as *const u8,
                std::mem::size_of::<[[f32; 4]; 4]>(),
            );

            let mut push_data = Vec::with_capacity(std::mem::size_of::<[[f32; 4]; 4]>() * 2);
            push_data.extend_from_slice(view_bytes);
            push_data.extend_from_slice(projection_bytes);

            self.context.device.cmd_bind_pipeline(
                frame.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );

            for index in 0..self.index_offset.len() {
                let index_offset = self.index_offset[index];
                let offset_bytes = std::slice::from_raw_parts(
                    &index_offset as *const [i32; 3] as *const u8,
                    std::mem::size_of::<[i32; 3]>(),
                );

                push_data.extend_from_slice(offset_bytes);

                self.context.device.cmd_push_constants(
                    frame.command_buffer,
                    self.pipeline_layout,
                    vk::ShaderStageFlags::VERTEX,
                    0,
                    &push_data,
                );

                self.context.device.cmd_bind_vertex_buffers(
                    frame.command_buffer,
                    0,
                    &[self.vertex_buffers[index]],
                    &[0],
                );

                self.context.device.cmd_bind_index_buffer(
                    frame.command_buffer,
                    self.index_buffers[index],
                    0,
                    vk::IndexType::UINT16,
                );
                self.context.device.cmd_draw_indexed(
                    frame.command_buffer,
                    self.index_counts[index],
                    1,
                    0,
                    0,
                    0,
                );
            }
            self.context.device.cmd_end_rendering(frame.command_buffer);

            self.context.transition_image_layout(
                frame.command_buffer,
                self.swapchain.images[image_index as usize],
                renderable_state,
                present_image_state,
                vk::ImageAspectFlags::COLOR,
            );

            self.context
                .device
                .end_command_buffer(frame.command_buffer)?;

            let image_available_semaphore_slice = &[frame.image_available_semaphore];
            let render_semaphore_slice = &[frame.render_finished_semaphore];
            let command_buffer = &[frame.command_buffer];

            let submit_info = vk::SubmitInfo::default()
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                .command_buffers(command_buffer)
                .wait_semaphores(image_available_semaphore_slice)
                .signal_semaphores(render_semaphore_slice);

            self.context.device.queue_submit(
                self.context.queues[self.context.queue_families.graphics as usize],
                &[submit_info],
                frame.in_flight_fence,
            )?;

            self.swapchain
                .present(image_index, &frame.render_finished_semaphore)?;

            self.current_frame = (self.current_frame + 1) % self.frames.len();
            Ok(())
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.context
                .device
                .destroy_image_view(self.depth_image_view, None);
            self.context
                .device
                .free_memory(self.depth_image_memory, None);
            self.context.device.destroy_image(self.depth_image, None);

            for buffer in self.vertex_buffers.iter() {
                self.context.device.destroy_buffer(*buffer, None);
            }

            self.context
                .device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.context.device.destroy_pipeline(self.pipeline, None);
        }
    }
}

fn load_shader_module(
    context: &Arc<RenderingContext>,
    path: &str,
) -> Result<ash::vk::ShaderModule> {
    let code = fs::read(format!("{SHADER_DIR}{path}"))?;
    Ok(context.create_shader_module(&code)?)
}

pub fn find_memory_type(type_filter: u32, properties: &PhysicalDeviceMemoryProperties) -> u32 {
    for index in 0..properties.memory_type_count {
        if (type_filter & (1 << index)) != 0
            && properties.memory_types[index as usize]
                .property_flags
                .contains(MemoryPropertyFlags::HOST_VISIBLE)
        {
            return index;
        }
    }
    panic!("Failed to find suitable memory type!");
}
pub fn create_vertex_buffer_from_data(
    renderer: &mut Renderer,
    vertex_data: Vec<VoxelVertex>,
    index_data: Vec<u16>,
    chunk_position: Vector3<i32>,
) {
    let context = &renderer.context;

    unsafe {
        // === VERTEX BUFFER === //

        let buffer_size = (size_of::<VoxelVertex>() * vertex_data.len()) as u64;

        let vertex_buffer_info = BufferCreateInfo {
            size: buffer_size,
            usage: BufferUsageFlags::VERTEX_BUFFER,
            sharing_mode: SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        let vertex_buffer = context
            .device
            .create_buffer(&vertex_buffer_info, None)
            .expect("Create vertex buffer failed!");

        let memory_requirements = context.device.get_buffer_memory_requirements(vertex_buffer);

        let alloc_info = MemoryAllocateInfo {
            allocation_size: memory_requirements.size,
            memory_type_index: find_memory_type(
                memory_requirements.memory_type_bits,
                &context.physical_device.memory_properties,
            ),
            ..Default::default()
        };

        let vertex_buffer_memory = context
            .device
            .allocate_memory(&alloc_info, None)
            .expect("Allocate vertex buffer memory failed!");

        context
            .device
            .bind_buffer_memory(vertex_buffer, vertex_buffer_memory, 0)
            .expect("Bind vertex buffer memory failed!");

        let data_ptr = context
            .device
            .map_memory(
                vertex_buffer_memory,
                0,
                buffer_size,
                vk::MemoryMapFlags::empty(),
            )
            .expect("Map memory failed!");

        std::ptr::copy_nonoverlapping(
            vertex_data.as_ptr() as *const c_void,
            data_ptr,
            buffer_size as usize,
        );

        context.device.unmap_memory(vertex_buffer_memory);
        // === INDEX BUFFER === //

        let index_buffer_size = (size_of::<u16>() * index_data.len()) as u64;

        let index_buffer_info = BufferCreateInfo {
            size: index_buffer_size,
            usage: BufferUsageFlags::INDEX_BUFFER,
            sharing_mode: SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        let index_buffer = context
            .device
            .create_buffer(&index_buffer_info, None)
            .expect("Create vertex buffer failed!");

        let index_memory_requirements = context.device.get_buffer_memory_requirements(index_buffer);

        let index_alloc_info = MemoryAllocateInfo {
            allocation_size: index_memory_requirements.size,
            memory_type_index: find_memory_type(
                index_memory_requirements.memory_type_bits,
                &context.physical_device.memory_properties,
            ),
            ..Default::default()
        };

        let index_buffer_memory = context
            .device
            .allocate_memory(&index_alloc_info, None)
            .expect("Allocate vertex buffer memory failed!");

        context
            .device
            .bind_buffer_memory(index_buffer, index_buffer_memory, 0)
            .expect("Bind index buffer memory failed!");

        let data_ptr = context
            .device
            .map_memory(
                index_buffer_memory,
                0,
                index_buffer_size,
                vk::MemoryMapFlags::empty(),
            )
            .expect("Map memory failed!");

        std::ptr::copy_nonoverlapping(
            index_data.as_ptr() as *const c_void,
            data_ptr,
            index_buffer_size as usize,
        );

        context.device.unmap_memory(index_buffer_memory);

        renderer.vertex_buffers.push(vertex_buffer);
        renderer.index_buffers.push(index_buffer);
        renderer.index_counts.push(index_data.len() as u32);
        renderer
            .index_offset
            .push([chunk_position.x + 1, chunk_position.y, chunk_position.z]);
        println!(
            "index offset: {:#?}",
            renderer.index_offset[renderer.index_offset.len() - 1]
        );
    }
}
