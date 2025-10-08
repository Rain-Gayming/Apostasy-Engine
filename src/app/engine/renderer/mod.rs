use std::sync::{Arc, Mutex};

pub mod camera;
mod swapchain;

use anyhow::Result;
use ash::vk::{self, ClearColorValue};
use winit::window::Window;

use crate::app::engine::renderer::camera::{get_perspective_projection, get_view_matrix, Camera};
use crate::app::engine::renderer::swapchain::Swapchain;
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
    command_pool: ash::vk::CommandPool,
    pipeline: ash::vk::Pipeline,
    pipeline_layout: ash::vk::PipelineLayout,
    swapchain: Swapchain,
    context: Arc<RenderingContext>,
    camera: Arc<Mutex<Camera>>,
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

        let vertex_shader = load_shader_module(&context, "vert.spv")?;
        let fragment_shader = load_shader_module(&context, "frag.spv")?;

        unsafe {
            let pipeline_layout = context
                .device
                .create_pipeline_layout(&ash::vk::PipelineLayoutCreateInfo::default(), None)?;

            let pipeline = context.create_graphics_pipeline(
                vertex_shader,
                fragment_shader,
                swapchain.extent,
                swapchain.format,
                pipeline_layout,
                Default::default(),
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
            for (_index, &command_buffer) in command_buffers.iter().enumerate() {
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
            })
        }
    }

    pub fn resize(&mut self) -> Result<()> {
        self.swapchain.resize()
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

            // transition image layout to be used for color attachmant
            self.context.transition_image_layout(
                frame.command_buffer,
                self.swapchain.images[image_index as usize],
                undefined_image_state,
                renderable_state,
                vk::ImageAspectFlags::COLOR,
            );

            // transition image layout to be presented for rendering
            self.context.transition_image_layout(
                frame.command_buffer,
                self.swapchain.images[image_index as usize],
                renderable_state,
                present_image_state,
                vk::ImageAspectFlags::COLOR,
            );

            self.context.begin_rendering(
                frame.command_buffer,
                self.swapchain.views[image_index as usize],
                ClearColorValue {
                    float32: [0.01, 0.01, 0.01, 1.0],
                },
                vk::Rect2D::default().extent(self.swapchain.extent),
            );

            self.context.device.cmd_set_viewport(
                frame.command_buffer,
                0,
                &[vk::Viewport::default()
                    .width(self.swapchain.extent.width as f32)
                    .height(self.swapchain.extent.height as f32)],
            );

            self.context.device.cmd_set_scissor(
                frame.command_buffer,
                0,
                &[vk::Rect2D::default().extent(self.swapchain.extent)],
            );
            self.context.device.cmd_bind_pipeline(
                frame.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
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

            let push_constant_range = vk::PushConstantRange {
                stage_flags: vk::ShaderStageFlags::VERTEX,
                offset: 0,
                size: std::mem::size_of::<[[f32; 4]; 4]>() as u32,
            };
            let pipeline_layout = self.context.device.create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default()
                    .push_constant_ranges(&[push_constant_range]),
                None,
            )?;

            self.context.device.cmd_push_constants(
                frame.command_buffer,
                pipeline_layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                view_bytes,
            );
            self.context.device.cmd_push_constants(
                frame.command_buffer,
                pipeline_layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                projection_bytes,
            );
            self.context.device.cmd_push_constants(
                frame.command_buffer,
                pipeline_layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                view_bytes,
            );

            self.context
                .device
                .cmd_draw(frame.command_buffer, 6, 1, 0, 0);

            self.context.device.cmd_end_rendering(frame.command_buffer);

            self.context
                .device
                .end_command_buffer(frame.command_buffer)?;

            self.context.device.queue_submit(
                self.context.queues[self.context.queue_families.graphics as usize],
                &[ash::vk::SubmitInfo::default()
                    .wait_semaphores(&[frame.image_available_semaphore])
                    .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                    .command_buffers(&[frame.command_buffer])
                    .signal_semaphores(&[frame.render_finished_semaphore])],
                frame.in_flight_fence,
            )?;

            self.swapchain
                .present(image_index, &frame.render_finished_semaphore)?;

            Ok(())
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.context.device.destroy_pipeline(self.pipeline, None);
            self.context
                .device
                .destroy_pipeline_layout(self.pipeline_layout, None);
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
