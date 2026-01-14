use std::sync::Arc;

use anyhow::Result;
use ash::vk;
use winit::{
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

const ENGINE_SHADER_DIR: &str = "apostasy/res/shaders/";

use crate::engine::rendering::{
    rendering_context::{ImageLayoutState, RenderingContext},
    swapchain::Swapchain,
};

pub struct Frame {
    pub command_buffer: vk::CommandBuffer,
    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,
    pub in_flight_fence: vk::Fence,
}

pub struct Renderer {
    pub frame_index: usize,
    pub frames: Vec<Frame>,
    pub command_pool: vk::CommandPool,
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub swapchain: Swapchain,
    pub context: Arc<RenderingContext>,
}

pub fn load_engine_shader_module(
    context: &RenderingContext,
    path: &str,
) -> Result<vk::ShaderModule> {
    let code = std::fs::read(format!("{}{}", ENGINE_SHADER_DIR, path))?;
    context.create_shader_module(&code)
}
impl Renderer {
    pub fn new(context: Arc<RenderingContext>, window: Arc<Window>) -> Result<Self> {
        let mut swapchain = Swapchain::new(context.clone(), window.clone())?;
        swapchain.resize().unwrap();

        let vertex_shader = load_engine_shader_module(context.as_ref(), "vert.spv")?;
        let fragment_shader = load_engine_shader_module(context.as_ref(), "frag.spv")?;

        unsafe {
            let pipeline_layout = context
                .device
                .create_pipeline_layout(&vk::PipelineLayoutCreateInfo::default(), None)?;

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
                &vk::CommandPoolCreateInfo::default()
                    .queue_family_index(context.queue_families.graphics)
                    .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
                None,
            )?;

            let inflight_frames_count = 1;
            let command_buffers = context.device.allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::default()
                    .command_buffer_count(inflight_frames_count)
                    .command_pool(command_pool)
                    .level(vk::CommandBufferLevel::PRIMARY),
            )?;

            let mut frames = Vec::with_capacity(command_buffers.len());
            for command_buffer in command_buffers.iter() {
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
                    command_buffer: *command_buffer,
                    image_available_semaphore,
                    render_finished_semaphore,
                    in_flight_fence,
                });
            }

            swapchain.resize().unwrap();

            Ok(Self {
                frame_index: 0,
                frames,
                command_pool,
                pipeline,
                pipeline_layout,
                swapchain,
                context,
            })
        }
    }

    pub fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
    }

    pub fn render(&mut self) -> Result<()> {
        let frame = &mut self.frames[self.frame_index];
        unsafe {
            // Wait for the image to be available
            self.context
                .device
                .wait_for_fences(&[frame.in_flight_fence], true, u64::MAX)?;
            self.context.device.reset_fences(&[frame.in_flight_fence])?;
            self.context
                .device
                .reset_command_buffer(frame.command_buffer, vk::CommandBufferResetFlags::empty())?;

            if self.swapchain.is_dirty {
                self.swapchain.resize()?;
            }

            // Acquire next image
            let image_index = self
                .swapchain
                .acquire_next_image(frame.image_available_semaphore)?;

            // Begin the render commands
            self.context.device.begin_command_buffer(
                frame.command_buffer,
                &vk::CommandBufferBeginInfo::default(),
            )?;

            // Undefined image state
            let undefined_image_state = ImageLayoutState {
                layout: vk::ImageLayout::UNDEFINED,
                access: vk::AccessFlags::empty(),
                stage: vk::PipelineStageFlags::TOP_OF_PIPE,
                queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            };

            // Rendering image state
            let render_image_state = ImageLayoutState {
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                access: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                stage: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            };

            // Presentable image state
            let present_image_state = ImageLayoutState {
                layout: vk::ImageLayout::PRESENT_SRC_KHR,
                access: vk::AccessFlags::empty(),
                stage: vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            };

            // Transtion the image layout from undefined to color attachment
            self.context.transition_image_layout(
                frame.command_buffer,
                self.swapchain.images[image_index as usize],
                undefined_image_state,
                render_image_state,
                vk::ImageAspectFlags::COLOR,
            );

            // Begin rendering
            self.context.begin_rendering(
                frame.command_buffer,
                self.swapchain.image_views[image_index as usize],
                vk::ClearColorValue {
                    float32: [0.01, 0.01, 0.01, 1.0],
                },
                vk::Rect2D::default().extent(self.swapchain.extent),
            )?;

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

            self.context.device.cmd_bind_pipeline(
                frame.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );

            self.context
                .device
                .cmd_draw(frame.command_buffer, 3, 1, 0, 0);

            self.context.device.cmd_end_rendering(frame.command_buffer);

            // Transition the image layout from color attachment to present
            self.context.transition_image_layout(
                frame.command_buffer,
                self.swapchain.images[image_index as usize],
                render_image_state,
                present_image_state,
                vk::ImageAspectFlags::COLOR,
            );
            self.context
                .device
                .end_command_buffer(frame.command_buffer)?;

            // Submit command buffer
            self.context.device.queue_submit(
                self.context.queues[self.context.queue_families.graphics as usize],
                &[vk::SubmitInfo::default()
                    .command_buffers(&[frame.command_buffer])
                    .wait_semaphores(&[frame.image_available_semaphore])
                    .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                    .signal_semaphores(&[frame.render_finished_semaphore])],
                frame.in_flight_fence,
            )?;
            self.swapchain
                .present_image(image_index, frame.render_finished_semaphore)?;
            self.frame_index = (self.frame_index + 1) % self.frames.len();
            Ok(())
        }
    }

    pub fn resize(&mut self) -> Result<()> {
        self.swapchain.resize()
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.context.device.device_wait_idle().unwrap();

            self.frames.drain(..).for_each(|frame| {
                self.context
                    .device
                    .destroy_semaphore(frame.image_available_semaphore, None);
                self.context
                    .device
                    .destroy_semaphore(frame.render_finished_semaphore, None);
                self.context
                    .device
                    .destroy_fence(frame.in_flight_fence, None);
                self.context
                    .device
                    .free_command_buffers(self.command_pool, &[frame.command_buffer]);
            });

            self.context
                .device
                .destroy_command_pool(self.command_pool, None);
            self.context.device.destroy_pipeline(self.pipeline, None);
            self.context
                .device
                .destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}
