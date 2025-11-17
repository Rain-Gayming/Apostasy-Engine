use std::fs;
use std::sync::Arc;

pub mod depth_image;
pub mod image_states;
pub mod mesh;
pub mod push_constants;
mod swapchain;
pub mod thread_manager;
pub mod vertex;

use anyhow::{Ok, Result};
use ash::vk::{
    self, ClearColorValue, CommandPool, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo,
    MemoryPropertyFlags, PhysicalDeviceMemoryProperties, Pipeline, PipelineLayout,
};

use crate::app::engine::ecs::resource::{ResMut, Resource};
use crate::app::engine::renderer::depth_image::{DepthImage, new_depth_image};
use crate::app::engine::renderer::image_states::ImageStates;
use crate::app::engine::renderer::rendering_context::RenderingContext;
use crate::app::engine::renderer::swapchain::Swapchain;
use crate::app::engine::rendering_context;
use winit::window::Window;

#[derive(Clone)]
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
    _command_pool: CommandPool,
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,
    swapchain: Swapchain,
    pub context: Arc<RenderingContext>,
    pub image_states: ImageStates,
    pub depth_image: DepthImage,
}
impl Resource for Renderer {}

const SHADER_DIR: &str = "res/shaders/";

impl Renderer {
    pub fn new(context: Arc<RenderingContext>, window: Arc<Window>) -> Result<Self> {
        let mut swapchain = Swapchain::new(Arc::clone(&context), window)?;
        swapchain.resize()?;

        let vertex_shader = load_shader_module(&context, "vert.spv")?;
        let fragment_shader = load_shader_module(&context, "frag.spv")?;

        unsafe {
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

            let depth_format = vk::Format::D32_SFLOAT;
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

            let depth_image = new_depth_image(&context, &swapchain);
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
                image_states: ImageStates::default(),
                depth_image,
            })
        }
    }
    //
}

pub fn resize(mut renderer: ResMut<Renderer>) {
    let _ = renderer.swapchain.resize();
}

/// Recreates the depth buffer upon screen resizing
pub fn update_depth_buffer(mut renderer: ResMut<Renderer>) {
    let depth_format = vk::Format::D32_SFLOAT;

    let depth_image_create_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .format(depth_format)
        .extent(vk::Extent3D {
            width: renderer.swapchain.extent.width,
            height: renderer.swapchain.extent.height,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
        .initial_layout(vk::ImageLayout::UNDEFINED);
    unsafe {
        let depth_image = renderer
            .context
            .device
            .create_image(&depth_image_create_info, None)
            .unwrap();
        let mem_req = renderer
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
            &renderer.context.physical_device.memory_properties,
        )
        .ok_or_else(|| anyhow::anyhow!("No suitable memory type for depth image"))
        .unwrap();

        let depth_alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_req.size)
            .memory_type_index(memory_type);
        let depth_image_memory = renderer
            .context
            .device
            .allocate_memory(&depth_alloc_info, None)
            .unwrap();
        renderer
            .context
            .device
            .bind_image_memory(depth_image, depth_image_memory, 0)
            .unwrap();

        let depth_image_view = renderer
            .context
            .create_image_view(
                depth_image,
                renderer.depth_image.depth_format,
                vk::ImageAspectFlags::DEPTH,
            )
            .unwrap();
        renderer.depth_image.depth_format = depth_format;
        renderer.depth_image.depth_image = depth_image;
        renderer.depth_image.depth_image_memory = depth_image_memory;
        renderer.depth_image.depth_image_view = depth_image_view;
    }
}

pub fn render(mut renderer: ResMut<Renderer>) {
    let frame = renderer.frames[renderer.current_frame].clone();
    unsafe {
        renderer
            .context
            .device
            .wait_for_fences(&[frame.in_flight_fence], true, u64::MAX)
            .unwrap();

        let image_index = renderer
            .swapchain
            .aquire_next_image(frame.image_available_semaphore)
            .unwrap();

        renderer
            .context
            .device
            .reset_fences(&[frame.in_flight_fence])
            .unwrap();

        renderer
            .context
            .device
            .reset_command_buffer(
                frame.command_buffer,
                ash::vk::CommandBufferResetFlags::empty(),
            )
            .unwrap();

        renderer
            .context
            .device
            .begin_command_buffer(
                frame.command_buffer,
                &ash::vk::CommandBufferBeginInfo::default(),
            )
            .unwrap();

        renderer.context.transition_image_layout(
            frame.command_buffer,
            renderer.depth_image.depth_image,
            renderer.image_states.undefined_image_state,
            renderer.image_states.depth_attach_state,
            vk::ImageAspectFlags::DEPTH,
        );

        renderer.context.transition_image_layout(
            frame.command_buffer,
            renderer.swapchain.images[image_index as usize],
            renderer.image_states.undefined_image_state,
            renderer.image_states.renderable_image_state,
            vk::ImageAspectFlags::COLOR,
        );

        renderer.context.begin_rendering(
            frame.command_buffer,
            renderer.swapchain.views[image_index as usize],
            ClearColorValue {
                float32: [0.01, 0.01, 0.01, 1.0],
            },
            vk::Rect2D::default().extent(renderer.swapchain.extent),
            renderer.depth_image.depth_image_view,
            vk::ClearDepthStencilValue {
                depth: 1.0,
                stencil: 0,
            },
        );

        renderer.context.device.cmd_set_viewport(
            frame.command_buffer,
            0,
            &[vk::Viewport::default()
                .width(renderer.swapchain.extent.width as f32)
                .height(renderer.swapchain.extent.height as f32)
                .min_depth(0.0)
                .max_depth(1.0)],
        );

        renderer.context.device.cmd_set_scissor(
            frame.command_buffer,
            0,
            &[vk::Rect2D::default().extent(renderer.swapchain.extent)],
        );

        // let aspect = renderer.swapchain.extent.width as f32 / self.swapchain.extent.height as f32;

        // let view: [[f32; 4]; 4] = get_view_matrix(renderer.camera.clone()).into();
        // let projection: [[f32; 4]; 4] =
        //     get_perspective_projection(renderer.camera.clone(), aspect).into();

        // renderer.push_constant.view_matrix = view;
        // renderer.push_constant.projection_matrix = projection;

        renderer.context.device.cmd_bind_pipeline(
            frame.command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            renderer.pipeline,
        );

        // for index in 0..renderer.index_offset.len() {
        //     let index_offset = renderer.index_offset[index];
        //     renderer.push_constant.chunk_position = index_offset;
        //
        //     let push_data = any_as_u8_slice(&renderer.push_constant);
        //
        //     renderer.context.device.cmd_push_constants(
        //         frame.command_buffer,
        //         renderer.pipeline_layout,
        //         vk::ShaderStageFlags::VERTEX,
        //         0,
        //         push_data,
        //     );
        //
        //     renderer.context.device.cmd_bind_vertex_buffers(
        //         frame.command_buffer,
        //         0,
        //         &[renderer.vertex_buffers[index]],
        //         &[0],
        //     );
        //
        //     renderer.context.device.cmd_bind_index_buffer(
        //         frame.command_buffer,
        //         renderer.index_buffers[index],
        //         0,
        //         vk::IndexType::UINT16,
        //     );
        //     renderer.context.device.cmd_draw_indexed(
        //         frame.command_buffer,
        //         renderer.index_counts[index],
        //         1,
        //         0,
        //         0,
        //         0,
        //     );
        // }
        renderer
            .context
            .device
            .cmd_end_rendering(frame.command_buffer);

        renderer.context.transition_image_layout(
            frame.command_buffer,
            renderer.swapchain.images[image_index as usize],
            renderer.image_states.renderable_image_state,
            renderer.image_states.present_image_state,
            vk::ImageAspectFlags::COLOR,
        );

        renderer
            .context
            .device
            .end_command_buffer(frame.command_buffer)
            .unwrap();

        let image_available_semaphore_slice = &[frame.image_available_semaphore];
        let render_semaphore_slice = &[frame.render_finished_semaphore];
        let command_buffer = &[frame.command_buffer];

        let submit_info = vk::SubmitInfo::default()
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .command_buffers(command_buffer)
            .wait_semaphores(image_available_semaphore_slice)
            .signal_semaphores(render_semaphore_slice);

        renderer
            .context
            .device
            .queue_submit(
                renderer.context.queues[renderer.context.queue_families.graphics as usize],
                &[submit_info],
                frame.in_flight_fence,
            )
            .unwrap();

        renderer
            .swapchain
            .present(image_index, &frame.render_finished_semaphore)
            .unwrap();

        renderer.current_frame = (renderer.current_frame + 1) % renderer.frames.len();
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            // for buffer in renderer.vertex_buffers.iter() {
            //     renderer.context.device.destroy_buffer(*buffer, None);
            // }

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
