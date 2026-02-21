use crate::engine::{
    ecs::{
        World,
        components::{
            camera::{Camera, get_perspective_projection},
            transform::{Transform, VoxelChunkTransform, calculate_forward, calculate_up},
        },
        entity::EntityView,
        system::{EguiRenderer, UIFunction},
    },
    rendering::models::{
        model::{MeshRenderer, ModelLoader, ModelRenderer, get_model},
        vertex::VertexType,
    },
};
use std::{collections::BTreeMap, sync::Arc};

use anyhow::Result;
use ash::vk::{self, DescriptorSet};
use cgmath::{Matrix4, Point3};
use egui::FontFamily;
use egui_ash_renderer::{DynamicRendering, Options};
use winit::{event::WindowEvent, window::Window};

const ENGINE_SHADER_DIR: &str = "res/shaders/";

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
    pub model_pipeline: vk::Pipeline,
    pub voxel_pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub swapchain: Swapchain,
    pub context: Arc<RenderingContext>,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set_layout: vk::DescriptorSetLayout,

    pub egui_renderer: EguiRenderer,
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

        let model_vertex_shader = load_engine_shader_module(context.as_ref(), "model_vert.spv")?;
        let model_fragment_shader = load_engine_shader_module(context.as_ref(), "model_frag.spv")?;

        let voxel_vertex_shader = load_engine_shader_module(context.as_ref(), "voxel_vert.spv")?;
        let voxel_fragment_shader = load_engine_shader_module(context.as_ref(), "voxel_frag.spv")?;

        unsafe {
            let ubo_binding = vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX);
            let sampler_binding = vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT);

            let descriptor_set_layout = context.device.create_descriptor_set_layout(
                &vk::DescriptorSetLayoutCreateInfo::default()
                    .bindings(&[ubo_binding, sampler_binding]),
                None,
            )?;

            let descriptor_pool = context.device.create_descriptor_pool(
                &vk::DescriptorPoolCreateInfo::default()
                    .max_sets(100)
                    .pool_sizes(&[vk::DescriptorPoolSize {
                        ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                        descriptor_count: 100,
                    }]),
                None,
            )?;

            let push_constant_range = vk::PushConstantRange::default()
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .offset(0)
                .size(140);

            let pipeline_layout = context.device.create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default()
                    .set_layouts(&[descriptor_set_layout])
                    .push_constant_ranges(&[push_constant_range]),
                None,
            )?;

            let model_pipeline = context.create_model_graphics_pipeline(
                model_vertex_shader,
                model_fragment_shader,
                swapchain.format,
                swapchain.depth_format,
                pipeline_layout,
                Default::default(),
            )?;

            let voxel_pipeline = context.create_voxel_graphics_pipeline(
                voxel_vertex_shader,
                voxel_fragment_shader,
                swapchain.format,
                swapchain.depth_format,
                pipeline_layout,
                Default::default(),
            )?;

            context
                .device
                .destroy_shader_module(model_vertex_shader, None);
            context
                .device
                .destroy_shader_module(model_fragment_shader, None);
            context
                .device
                .destroy_shader_module(voxel_vertex_shader, None);
            context
                .device
                .destroy_shader_module(voxel_fragment_shader, None);

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

            let egui_renderer = EguiRenderer::new(&context, &swapchain, &window);

            Ok(Self {
                frame_index: 0,
                frames,
                command_pool,
                model_pipeline,
                voxel_pipeline,
                pipeline_layout,
                swapchain,
                context,
                descriptor_pool,
                descriptor_set_layout,
                egui_renderer,
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
                println!("Swapchain resized");
            }

            let full_output = self.egui_renderer.egui_ctx.end_pass();
            let clipped_primitives = self
                .egui_renderer
                .egui_ctx
                .tessellate(full_output.shapes, full_output.pixels_per_point);
            let texture_updates: Vec<(egui::TextureId, egui::epaint::ImageDelta)> = full_output
                .textures_delta
                .set
                .iter()
                .map(|(id, delta)| (*id, delta.clone()))
                .collect();
            if !texture_updates.is_empty() {
                self.egui_renderer.egui_renderer.set_textures(
                    self.context.queues[self.context.queue_families.graphics as usize],
                    self.command_pool,
                    &texture_updates,
                )?;
            }

            let frame = &mut self.frames[self.frame_index];

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

            // Transition the image layout from undefined to color attachment
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

            // Texture loading
            world
                .query()
                .include::<ModelRenderer>()
                .include::<Transform>()
                .build()
                .run(|entity_view: EntityView<'_>| {
                    world.with_resource_mut::<ModelLoader, _, _>(|model_loader| {
                        let model_renderer = entity_view.get::<ModelRenderer>().unwrap();

                        let mut model_name = model_renderer.0.clone();
                        model_name.push_str(".glb");

                        let model = get_model(&model_name, model_loader);
                        for mesh in &mut model.meshes {
                            if mesh.material.base_color_texture.is_none()
                                && let Some(ref texture_name) = mesh.material.texture_name.clone()
                            {
                                let texture = self
                                    .context
                                    .load_texture(
                                        &texture_name,
                                        self.command_pool,
                                        self.descriptor_pool,
                                        self.descriptor_set_layout,
                                    )
                                    .unwrap();

                                mesh.material.base_color_texture = Some(texture);
                                println!("texture loaded");
                                println!("{}", mesh.material.base_color_texture.is_some());
                            }
                        }
                    });
                });

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

            let command_buffer = frame.command_buffer;
            let device = &self.context.device;
            let pipeline_layout = self.pipeline_layout;
            let swapchain_extent = self.swapchain.extent;

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
                        // Camera's position as a point
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

                        let mut push_constants = [0u8; 140];
                        push_constants[0..64].copy_from_slice(&mvp_bytes);
                        push_constants[64..128].copy_from_slice(&model_bytes);

                        // Render Model pipeline objects
                        device.cmd_bind_pipeline(
                            command_buffer,
                            vk::PipelineBindPoint::GRAPHICS,
                            self.model_pipeline,
                        );

                        // Render ModelRenderer entities
                        world
                            .query()
                            .include::<ModelRenderer>()
                            .include::<Transform>()
                            .build()
                            .run(|entity_view: EntityView<'_>| {
                                world.with_resource_mut::<ModelLoader, _, _>(|model_loader| {
                                    // Add position offset
                                    if let Some(transform) = entity_view.get::<Transform>() {
                                        // Add position offset
                                        let offset = [
                                            transform.position.x,
                                            transform.position.y,
                                            transform.position.z,
                                        ];
                                        let offset_bytes: [u8; 12] = std::mem::transmute(offset);
                                        push_constants[128..140].copy_from_slice(&offset_bytes);
                                    }

                                    device.cmd_push_constants(
                                        command_buffer,
                                        pipeline_layout,
                                        vk::ShaderStageFlags::VERTEX,
                                        0,
                                        &push_constants,
                                    );

                                    let model_renderer =
                                        entity_view.get::<ModelRenderer>().unwrap();

                                    let mut model_name = model_renderer.0.clone();
                                    model_name.push_str(".glb");

                                    let model = get_model(&model_name, model_loader);
                                    for mesh in &mut model.meshes {
                                        if let Some(ref texture) = mesh.material.base_color_texture
                                        {
                                            device.cmd_bind_descriptor_sets(
                                                command_buffer,
                                                vk::PipelineBindPoint::GRAPHICS,
                                                pipeline_layout,
                                                0,
                                                &[texture.descriptor_set],
                                                &[],
                                            );
                                        }

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
                            });

                        // Render MeshRenderer entities with Model vertex type
                        world
                            .query()
                            .include::<MeshRenderer>()
                            .include::<Transform>()
                            .build()
                            .run(|entity_view: EntityView<'_>| {
                                if let Some(transform) = entity_view.get::<Transform>() {
                                    // Add position offset
                                    let offset = [
                                        transform.position.x,
                                        transform.position.y,
                                        transform.position.z,
                                    ];
                                    let offset_bytes: [u8; 12] = std::mem::transmute(offset);
                                    push_constants[128..140].copy_from_slice(&offset_bytes);
                                    device.cmd_push_constants(
                                        command_buffer,
                                        pipeline_layout,
                                        vk::ShaderStageFlags::VERTEX,
                                        0,
                                        &push_constants,
                                    );
                                }

                                let mesh = entity_view.get::<MeshRenderer>().unwrap().0.clone();

                                if mesh.vertex_type == VertexType::Model {
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

                        // Render Voxel pipeline objects
                        device.cmd_bind_pipeline(
                            command_buffer,
                            vk::PipelineBindPoint::GRAPHICS,
                            self.voxel_pipeline,
                        );

                        device.cmd_push_constants(
                            command_buffer,
                            pipeline_layout,
                            vk::ShaderStageFlags::VERTEX,
                            0,
                            &push_constants,
                        );

                        // Render MeshRenderer entities with Voxel vertex type
                        world
                            .query()
                            .include::<MeshRenderer>()
                            .include::<VoxelChunkTransform>()
                            .build()
                            .run(|entity_view: EntityView<'_>| {
                                // Add position offset
                                if let Some(transform) = entity_view.get::<VoxelChunkTransform>() {
                                    // Add position offset
                                    let offset = [
                                        transform.position.x as f32,
                                        transform.position.y as f32,
                                        transform.position.z as f32,
                                    ];
                                    let offset_bytes: [u8; 12] = std::mem::transmute(offset);
                                    push_constants[128..140].copy_from_slice(&offset_bytes);
                                    device.cmd_push_constants(
                                        command_buffer,
                                        pipeline_layout,
                                        vk::ShaderStageFlags::VERTEX,
                                        0,
                                        &push_constants,
                                    );
                                }
                                let mesh = entity_view.get::<MeshRenderer>().unwrap().0.clone();
                                if mesh.vertex_type == VertexType::Voxel {
                                    device.cmd_bind_vertex_buffers(
                                        command_buffer,
                                        1,
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
                    }
                });

            self.context.device.cmd_end_rendering(frame.command_buffer);

            // Render egui

            // Begin rendering again for egui
            let color_attachment = vk::RenderingAttachmentInfo::default()
                .image_view(self.swapchain.image_views[image_index as usize])
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::LOAD) // Load existing content
                .store_op(vk::AttachmentStoreOp::STORE);

            let rendering_info = vk::RenderingInfo::default()
                .render_area(vk::Rect2D::default().extent(self.swapchain.extent))
                .layer_count(1)
                .color_attachments(std::slice::from_ref(&color_attachment));

            self.context
                .device
                .cmd_begin_rendering(frame.command_buffer, &rendering_info);

            // Set viewport and scissor for egui
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

            // Render egui
            self.egui_renderer.egui_renderer.cmd_draw(
                frame.command_buffer,
                self.swapchain.extent,
                full_output.pixels_per_point,
                &clipped_primitives,
            )?;
            self.context.device.cmd_end_rendering(frame.command_buffer);
            // egui render end
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

    pub fn window_event(&mut self, window: &Window, event: WindowEvent) -> bool {
        let response = self
            .egui_renderer
            .egui_state
            .on_window_event(window, &event);

        response.consumed
    }

    pub fn resize(&mut self) -> Result<()> {
        self.swapchain.resize()
    }

    /// Prepares egui for rendering
    pub fn prepare_egui(&mut self, window: &Window, world: &mut World) {
        let raw_input = self.egui_renderer.egui_state.take_egui_input(window);
        self.egui_renderer.egui_ctx.begin_pass(raw_input);

        for system in &self.egui_renderer.sorted_ui_systems {
            (system.func)(&mut self.egui_renderer.egui_ctx, world);
        }
    }
}
//
// #[ui]
// fn test(context: &mut Context, world: &mut World) {
//     egui::Window::new("Debug Info")
//         .default_pos([10.0, 10.0])
//         .show(&context, |ui| {
//             ui.heading("Engine Stats");
//             ui.separator();
//             ui.label(format!(
//                 "Entity Count: {}",
//                 world
//                     .crust
//                     .mantle(|mantle| mantle.core.entity_index.lock().len())
//             ));
//             ui.label(format!("Archetypes: {}", world.debug_archetypes()));
//             ui.label(format!(
//                 "FPS: {}",
//                 world.with_resource(|fps: &FPSCounter| fps.fps())
//             ));
//             ui.separator();
//         });
// }

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
            self.context
                .device
                .destroy_pipeline(self.model_pipeline, None);
            self.context
                .device
                .destroy_pipeline(self.voxel_pipeline, None);
            self.context
                .device
                .destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}
