use crate::engine::{
    ecs::{
        World,
        components::{
            camera::{Camera, get_perspective_projection},
            transform::{Transform, calculate_forward, calculate_up},
        },
        entity::EntityView,
    },
    rendering::models::model::{MeshRenderer, ModelLoader, ModelRenderer, get_model},
};
use std::sync::Arc;

use anyhow::Result;
use ash::vk::{self, DescriptorSet};
use cgmath::{Matrix4, Point3};
use egui_ash_renderer::{DynamicRendering, Options};
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

/// A frame of the renderer
pub struct Frame {
    pub command_buffer: vk::CommandBuffer,
    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,
    pub in_flight_fence: vk::Fence,
}

/// A renderer
pub struct Renderer {
    pub frame_index: usize,
    pub frames: Vec<Frame>,
    pub command_pool: vk::CommandPool,
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub swapchain: Swapchain,
    pub context: Arc<RenderingContext>,

    pub egui_state: egui_winit::State,
    pub egui_renderer: egui_ash_renderer::Renderer,
    pub egui_ctx: egui::Context,
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
            let ubo_binding = vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX);

            let descriptor_set_layout = context.device.create_descriptor_set_layout(
                &vk::DescriptorSetLayoutCreateInfo::default().bindings(&[ubo_binding]),
                None,
            )?;
            let push_constant_range = vk::PushConstantRange::default()
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .offset(0)
                .size(128);

            let pipeline_layout = context.device.create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default()
                    .set_layouts(&[descriptor_set_layout])
                    .push_constant_ranges(&[push_constant_range]),
                None,
            )?;

            let pipeline = context.create_graphics_pipeline(
                vertex_shader,
                fragment_shader,
                swapchain.format,
                swapchain.depth_format,
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

            let inflight_frames_count = swapchain.images.len() as u32;
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
            let egui_state = egui_winit::State::new(
                egui::Context::default(),
                egui::ViewportId::ROOT,
                &window,
                None,
                None,
                None,
            );

            let mut egui_renderer = egui_ash_renderer::Renderer::with_default_allocator(
                &context.instance,
                context.physical_device.handle,
                context.device.clone(),
                DynamicRendering {
                    color_attachment_format: swapchain.format,
                    depth_attachment_format: Some(swapchain.depth_format),
                },
                Options::default(),
            )?;
            egui_renderer.add_user_texture(DescriptorSet::default());

            let egui_ctx = egui::Context::default();
            Ok(Self {
                frame_index: 0,
                frames,
                command_pool,
                pipeline,
                pipeline_layout,
                swapchain,
                context,
                egui_state,
                egui_renderer,
                egui_ctx,
            })
        }
    }

    /// Renders the world from a perspective of a camera
    pub fn render(&mut self, world: &World) -> Result<()> {
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

            // Depth attachment state
            let depth_attachment_state = ImageLayoutState {
                layout: vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
                access: vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                stage: vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
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

            // Transition depth image to depth attachment optimal
            self.context.transition_image_layout(
                frame.command_buffer,
                self.swapchain.depth_image,
                undefined_image_state,
                depth_attachment_state,
                vk::ImageAspectFlags::DEPTH,
            );

            // Begin rendering
            self.context.begin_rendering(
                frame.command_buffer,
                self.swapchain.image_views[image_index as usize],
                self.swapchain.depth_image_view,
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

            let command_buffer = frame.command_buffer;
            let device = &self.context.device;
            let pipeline_layout = self.pipeline_layout;
            let swapchain_extent = self.swapchain.extent;

            // Query for entities with Camera and Transform components
            // Query for entities with Camera and Transform components
            world
                .query()
                .include::<Camera>()
                .include::<Transform>()
                .build()
                .run(|entity_view: EntityView<'_>| {
                    if let (Some(camera), Some(transform)) =
                        (entity_view.get::<Camera>(), entity_view.get::<Transform>())
                    {
                        let aspect = swapchain_extent.width as f32 / swapchain_extent.height as f32;

                        let model = Matrix4::from_scale(1.0);
                        // Cameras position as a point
                        let camera_eye = Point3::new(
                            transform.position.x,
                            transform.position.y,
                            transform.position.z,
                        );

                        // Forward direction from the camera
                        let rotated_forward = calculate_forward(&transform);

                        // Look point of the camera
                        let look_at = Point3::new(
                            camera_eye.x + rotated_forward.x,
                            camera_eye.y + rotated_forward.y,
                            camera_eye.z + rotated_forward.z,
                        );

                        // Get the up direction
                        let rotated_up = calculate_up(&transform);

                        let view = Matrix4::look_at_rh(camera_eye, look_at, rotated_up);

                        let projection = get_perspective_projection(&camera, aspect);
                        // Compute Projection * View * Model
                        let mvp = projection * view * model;

                        // Convert matrices to bytes for push constant
                        let mvp_bytes: [u8; 64] = std::mem::transmute(mvp);
                        let model_bytes: [u8; 64] = std::mem::transmute(model);

                        let mut push_constants = [0u8; 128];
                        push_constants[0..64].copy_from_slice(&mvp_bytes);
                        push_constants[64..128].copy_from_slice(&model_bytes);

                        device.cmd_push_constants(
                            command_buffer,
                            pipeline_layout,
                            vk::ShaderStageFlags::VERTEX,
                            0,
                            &push_constants,
                        );

                        device.cmd_push_constants(
                            command_buffer,
                            pipeline_layout,
                            vk::ShaderStageFlags::VERTEX,
                            0,
                            &push_constants,
                        );

                        world.query().include::<ModelRenderer>().build().run(
                            |entity_view: EntityView<'_>| {
                                world.with_resource::<ModelLoader, _>(|model_loader| {
                                    let model_renderer =
                                        entity_view.get::<ModelRenderer>().unwrap();
                                    let meshes =
                                        get_model(model_renderer.0.as_str(), model_loader).meshes;
                                    for mesh in meshes {
                                        device.cmd_bind_vertex_buffers(
                                            command_buffer,
                                            0,
                                            &[mesh.vertex_buffer],
                                            &[0],
                                        );
                                        device.cmd_bind_index_buffer(
                                            command_buffer,
                                            mesh.index_buffer,
                                            0,
                                            vk::IndexType::UINT32,
                                        );
                                        device.cmd_draw_indexed(
                                            command_buffer,
                                            mesh.index_count,
                                            1,
                                            0,
                                            0,
                                            0,
                                        );
                                    }
                                });
                            },
                        );
                        world.query().include::<MeshRenderer>().build().run(
                            |entity_view: EntityView<'_>| {
                                world.with_resource::<ModelLoader, _>(|model_loader| {
                                    let mesh = entity_view.get::<MeshRenderer>().unwrap().0.clone();
                                    device.cmd_bind_vertex_buffers(
                                        command_buffer,
                                        0,
                                        &[mesh.vertex_buffer],
                                        &[0],
                                    );
                                    device.cmd_bind_index_buffer(
                                        command_buffer,
                                        mesh.index_buffer,
                                        0,
                                        vk::IndexType::UINT32,
                                    );
                                    device.cmd_draw_indexed(
                                        command_buffer,
                                        mesh.index_count,
                                        1,
                                        0,
                                        0,
                                        0,
                                    );
                                });
                            },
                        );

                        // device.cmd_draw(command_buffer, 36, 1, 0, 0);
                    }
                });
            self.context.device.cmd_end_rendering(frame.command_buffer);

            // Transition the image layout from color attachment to present
            self.context.transition_image_layout(
                frame.command_buffer,
                self.swapchain.images[image_index as usize],
                render_image_state,
                present_image_state,
                vk::ImageAspectFlags::COLOR,
            );

            // End the render commands
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

    pub fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
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
