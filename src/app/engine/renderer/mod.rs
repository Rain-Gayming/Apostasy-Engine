use std::sync::{Arc, Mutex};

pub mod camera;
mod swapchain;
pub mod voxel_vertex;

use anyhow::{Ok, Result};
use ash::vk::{self, Buffer, ClearColorValue, MemoryPropertyFlags, PhysicalDeviceMemoryProperties};
use winit::window::Window;

use crate::app::engine::renderer::camera::{get_perspective_projection, get_view_matrix, Camera};
use crate::app::engine::renderer::swapchain::Swapchain;
use crate::app::engine::{renderer, rendering_context};
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
    command_pool: ash::vk::CommandPool,
    pipeline: ash::vk::Pipeline,
    pipeline_layout: ash::vk::PipelineLayout,
    swapchain: Swapchain,
    pub context: Arc<RenderingContext>,
    camera: Arc<Mutex<Camera>>,
    depth_format: vk::Format,
    depth_image: vk::Image,
    depth_image_memory: vk::DeviceMemory,
    depth_image_view: vk::ImageView,
    vertex_buffers: Vec<Buffer>,
    vertex_data: u32,
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
                size: (std::mem::size_of::<[[f32; 4]; 4]>() * 2) as u32,
            };

            let pipeline_layout = context.device.create_pipeline_layout(
                &ash::vk::PipelineLayoutCreateInfo::default()
                    .push_constant_ranges(&[push_constant_range]),
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

            let in_flight_frames_count = 1;

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
                command_pool,
                pipeline,
                pipeline_layout,
                context,
                swapchain,
                camera,
                depth_format,
                depth_image,
                depth_image_memory,
                depth_image_view,
                vertex_buffers: Vec::new(),
                vertex_data: 0,
            })
        }
    }

    pub fn update_depth_buffer(&mut self) -> Result<()> {
        let depth_format = vk::Format::D32_SFLOAT;

        let vertex_shader = load_shader_module(&self.context, "vert.spv")?;
        let fragment_shader = load_shader_module(&self.context, "frag.spv")?;
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

            let push_constant_range = vk::PushConstantRange {
                stage_flags: vk::ShaderStageFlags::VERTEX,
                offset: 0,
                size: (std::mem::size_of::<[[f32; 4]; 4]>() * 2) as u32,
            };

            let pipeline_layout = self.context.device.create_pipeline_layout(
                &ash::vk::PipelineLayoutCreateInfo::default()
                    .push_constant_ranges(&[push_constant_range]),
                None,
            )?;

            let pipeline = self.context.create_graphics_pipeline(
                vertex_shader,
                fragment_shader,
                self.swapchain.extent,
                self.swapchain.format,
                pipeline_layout,
                Default::default(),
                Some(depth_format),
            );

            self.depth_format = depth_format;
            self.depth_image = depth_image;
            self.depth_image_memory = depth_image_memory;
            self.depth_image_view = depth_image_view;
            self.pipeline = pipeline?;
            self.pipeline_layout = pipeline_layout;

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

            self.context.device.reset_fences(&[frame.in_flight_fence])?;

            self.context.device.reset_command_buffer(
                frame.command_buffer,
                ash::vk::CommandBufferResetFlags::empty(),
            )?;

            let image_index = self
                .swapchain
                .aquire_next_image(frame.image_available_semaphore)?;

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

            // transition image layout to be used for color attachmant
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
                }, // depth clear
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

            self.context.device.cmd_bind_vertex_buffers(
                frame.command_buffer,
                0,
                &self.vertex_buffers,
                &[0],
            );
            self.context.device.cmd_push_constants(
                frame.command_buffer,
                self.pipeline_layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                &push_data,
            );
            self.context.device.cmd_bind_pipeline(
                frame.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );

            self.context
                .device
                .cmd_draw(frame.command_buffer, self.vertex_data, 1, 0, 0);

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

            let render_finished_semaphore = self
                .context
                .device
                .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                .expect("Create semaphore failed!");

            let image_available_semaphore = &[frame.image_available_semaphore];
            let render_finished_semaphore_holder = &[render_finished_semaphore];
            let command_buffer = &[frame.command_buffer];
            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(image_available_semaphore)
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                .command_buffers(command_buffer)
                .signal_semaphores(render_finished_semaphore_holder);

            self.context.device.queue_submit(
                self.context.queues[self.context.queue_families.graphics as usize],
                &[submit_info],
                frame.in_flight_fence,
            )?;

            self.swapchain
                .present(image_index, &render_finished_semaphore)?;

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
    vertex_buffer_info: vk::BufferCreateInfo,
    renderer: &mut Renderer,
    vertex_count: usize,
) {
    let context = renderer.context.clone();
    unsafe {
        let vertex_buffer: ash::vk::Buffer = context
            .device
            .create_buffer(&vertex_buffer_info, None)
            .expect("Create vertex buffer failed!");
        let memory_requirements = context.device.get_buffer_memory_requirements(vertex_buffer);
        let alloc_info = ash::vk::MemoryAllocateInfo {
            allocation_size: memory_requirements.size,
            memory_type_index: find_memory_type(
                memory_requirements.memory_type_bits,
                &context.physical_device.memory_properties,
            ),
            ..Default::default()
        };

        let vertex_buffer_memory: ash::vk::DeviceMemory = context
            .device
            .allocate_memory(&alloc_info, None)
            .expect("Allocate vertex buffer memory failed!");

        context
            .device
            .bind_buffer_memory(vertex_buffer, vertex_buffer_memory, 0)
            .expect("Bind vertex buffer memory failed!");

        renderer.vertex_buffers.push(vertex_buffer);
        renderer.vertex_data += vertex_count as u32;
    }
}
