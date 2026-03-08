use std::mem::transmute;
use std::path::Path;
use std::sync::RwLock;
use std::time::Instant;
use std::{collections::BTreeMap, sync::Arc};

use anyhow::Result;
use ash::vk::{self, DescriptorSet};
use cgmath::{Matrix4, Point3};
use egui::{Context, FontFamily};
use egui_ash_renderer::{DynamicRendering, Options};
use winit::{event::WindowEvent, window::Window};

use crate::engine::assets::handle::Handle;
use crate::engine::assets::server::AssetServer;
use crate::engine::rendering::models::gltf_loader::GltfLoader;
use crate::engine::rendering::models::material::MaterialAsset;
use crate::engine::rendering::models::model::GpuModel;
use crate::engine::rendering::models::shader::ShaderSpirv;
use crate::engine::rendering::models::texture::{GpuTexture, GpuTextureLoader};
use crate::engine::rendering::profiler::{CpuProfiler, FrameData, GpuTimestampPool};
use crate::engine::{
    editor::{EditorStorage, style::style},
    nodes::{
        Node, World,
        components::{
            camera::{Camera, get_perspective_projection},
            light::Light,
            transform::Transform,
        },
        system::EditorUIFunction,
    },
    rendering::{
        models::model::ModelRenderer,
        rendering_context::{ImageLayoutState, RenderingContext},
        swapchain::Swapchain,
    },
};

pub struct Frame {
    pub command_buffer: vk::CommandBuffer,
    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,
    pub in_flight_fence: vk::Fence,
}

pub struct EguiRenderer {
    pub egui_state: egui_winit::State,
    pub egui_renderer: egui_ash_renderer::Renderer,
    pub egui_ctx: egui::Context,
    pub sorted_ui_systems: Vec<&'static UIFunction>,
    pub sorted_editor_ui_systems: Vec<&'static EditorUIFunction>,
}

impl EguiRenderer {
    pub fn new(
        context: &crate::engine::rendering::rendering_context::RenderingContext,
        swapchain: &Swapchain,
        window: &Window,
    ) -> Self {
        let mut egui_renderer = egui_ash_renderer::Renderer::with_default_allocator(
            &context.instance,
            context.physical_device.handle,
            context.device.clone(),
            DynamicRendering {
                color_attachment_format: swapchain.format,
                depth_attachment_format: Some(swapchain.depth_format),
            },
            Options {
                srgb_framebuffer: true,
                ..Default::default()
            },
        )
        .unwrap();
        egui_renderer.add_user_texture(DescriptorSet::default());

        let mut fonts = egui::FontDefinitions::default();
        let mut new_font_family = BTreeMap::new();
        new_font_family.insert(
            FontFamily::Name("fantasy".into()),
            vec!["fantasy".to_owned()],
        );
        fonts.families.append(&mut new_font_family);
        fonts.font_data.insert(
            "fantasy".to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../../../res/fonts/FantasyFont.ttf"
            ))),
        );

        let egui_ctx = egui::Context::default();
        egui_ctx.set_fonts(fonts);
        egui_ctx.set_style(style());

        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            None,
            None,
            None,
        );

        let mut sorted_ui_systems: Vec<&'static UIFunction> =
            inventory::iter::<UIFunction>.into_iter().collect();
        sorted_ui_systems.sort_by_key(|s| s.priority);
        sorted_ui_systems.reverse();

        let mut sorted_editor_ui_systems: Vec<&'static EditorUIFunction> =
            inventory::iter::<EditorUIFunction>.into_iter().collect();
        sorted_editor_ui_systems.sort_by_key(|s| s.priority);
        sorted_editor_ui_systems.reverse();

        Self {
            egui_state,
            egui_renderer,
            egui_ctx,
            sorted_ui_systems,
            sorted_editor_ui_systems,
        }
    }
}

pub struct UIFunction {
    pub name: &'static str,
    pub func: fn(&mut Context),
    pub priority: u32,
}
inventory::collect!(UIFunction);

pub struct Renderer {
    pub frame_index: usize,
    pub frames: Vec<Frame>,
    pub command_pool: vk::CommandPool,
    pub transfer_command_pool: vk::CommandPool,
    pub voxel_pipeline: vk::Pipeline,
    pub model_pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub swapchain: Swapchain,
    pub context: Arc<RenderingContext>,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub egui_renderer: EguiRenderer,
    pub default_descriptor_set: vk::DescriptorSet,
    pub default_ubo: vk::Buffer,
    pub default_ubo_memory: vk::DeviceMemory,

    // Profiler
    pub gpu_timestamps: GpuTimestampPool,
    pub cpu_profiler: CpuProfiler,
    pub pending_frame_data: Option<FrameData>,
    global_frame_counter: u64,

    // image states
    pub undefined_image_state: ImageLayoutState,
    pub render_image_state: ImageLayoutState,
    pub depth_attachment_state: ImageLayoutState,
    pub present_image_state: ImageLayoutState,
}

impl Renderer {
    #[allow(unnecessary_transmutes)]
    pub fn new(
        context: Arc<RenderingContext>,
        window: Arc<Window>,
        asset_server: &mut Arc<RwLock<AssetServer>>,
    ) -> Result<Self> {
        let mut swapchain = Swapchain::new(context.clone(), window.clone())?;
        swapchain.resize()?;

        let mut asset_server = asset_server.write().unwrap();

        let mvs_handle: Handle<ShaderSpirv> = asset_server.load("shaders/model_vert.spv")?;
        let mfs_handle: Handle<ShaderSpirv> = asset_server.load("shaders/model_frag.spv")?;
        let vvs_handle: Handle<ShaderSpirv> = asset_server.load("shaders/voxel_vert.spv")?;
        let vfs_handle: Handle<ShaderSpirv> = asset_server.load("shaders/voxel_frag.spv")?;

        let model_vertex_shader = asset_server
            .get(mvs_handle)
            .unwrap()
            .create_module(&context.device)?;
        let model_fragment_shader = asset_server
            .get(mfs_handle)
            .unwrap()
            .create_module(&context.device)?;
        let voxel_vertex_shader = asset_server
            .get(vvs_handle)
            .unwrap()
            .create_module(&context.device)?;
        let voxel_fragment_shader = asset_server
            .get(vfs_handle)
            .unwrap()
            .create_module(&context.device)?;

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
                    .max_sets(200)
                    .pool_sizes(&[
                        vk::DescriptorPoolSize {
                            ty: vk::DescriptorType::UNIFORM_BUFFER,
                            descriptor_count: 100,
                        },
                        vk::DescriptorPoolSize {
                            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                            descriptor_count: 100,
                        },
                    ]),
                None,
            )?;

            let push_constant_range = vk::PushConstantRange::default()
                .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
                .offset(0)
                .size(256);

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

            let transfer_command_pool = context.device.create_command_pool(
                &vk::CommandPoolCreateInfo::default()
                    .queue_family_index(context.queue_families.transfer)
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

            swapchain.resize()?;

            let egui_renderer = EguiRenderer::new(&context, &swapchain, &window);

            let (default_ubo, default_ubo_mem) = context.create_buffer(
                256,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;

            let default_descriptor_set = context.device.allocate_descriptor_sets(
                &vk::DescriptorSetAllocateInfo::default()
                    .descriptor_pool(descriptor_pool)
                    .set_layouts(&[descriptor_set_layout]),
            )?[0];

            let ubo_info = vk::DescriptorBufferInfo::default()
                .buffer(default_ubo)
                .offset(0)
                .range(256);

            context.device.update_descriptor_sets(
                &[vk::WriteDescriptorSet::default()
                    .dst_set(default_descriptor_set)
                    .dst_binding(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(&[ubo_info])],
                &[],
            );

            let mut gpu_timestamps = GpuTimestampPool::new(&context.device, context.queues[0]);
            let phys_props = context
                .instance
                .get_physical_device_properties(context.physical_device.handle);
            gpu_timestamps.set_timestamp_period(phys_props.limits.timestamp_period);

            let undefined_image_state = ImageLayoutState {
                layout: vk::ImageLayout::UNDEFINED,
                access: vk::AccessFlags::empty(),
                stage: vk::PipelineStageFlags::TOP_OF_PIPE,
                queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            };
            let render_image_state = ImageLayoutState {
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                access: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                stage: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            };
            let depth_attachment_state = ImageLayoutState {
                layout: vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
                access: vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                stage: vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            };
            let present_image_state = ImageLayoutState {
                layout: vk::ImageLayout::PRESENT_SRC_KHR,
                access: vk::AccessFlags::empty(),
                stage: vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            };

            asset_server.register_loader(GltfLoader::new(context.clone(), command_pool));
            // ---- CHANGED: pass default_ubo into loader ----
            asset_server.register_loader(GpuTextureLoader::new(
                context.clone(),
                command_pool,
                descriptor_pool,
                descriptor_set_layout,
                default_ubo,
            ));

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
                transfer_command_pool,
                default_descriptor_set,
                default_ubo,
                default_ubo_memory: default_ubo_mem,
                gpu_timestamps,
                cpu_profiler: CpuProfiler::new(),
                global_frame_counter: 0,
                pending_frame_data: None,
                undefined_image_state,
                render_image_state,
                depth_attachment_state,
                present_image_state,
            })
        }
    }
    pub fn render(
        &mut self,
        world: &mut World,
        asset_server: &Arc<RwLock<AssetServer>>,
        is_editor: bool,
    ) -> Result<()> {
        //CPU: whole-frame wall clock
        let frame_wall_start = Instant::now();
        self.cpu_profiler.begin("Frame Total");

        let frame = &mut self.frames[self.frame_index];
        unsafe {
            // CPU: fence wait
            self.cpu_profiler.begin("Fence Wait");
            self.context
                .device
                .wait_for_fences(&[frame.in_flight_fence], true, u64::MAX)?;
            self.cpu_profiler.end(); // Fence Wait

            self.context.device.reset_fences(&[frame.in_flight_fence])?;
            self.context
                .device
                .reset_command_buffer(frame.command_buffer, vk::CommandBufferResetFlags::empty())?;

            if self.swapchain.is_dirty {
                self.swapchain.resize()?;
                println!("Swapchain resized");
            }

            //  CPU: egui tessellation
            self.cpu_profiler.begin("egui Tessellate");

            // egui render pass
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

            self.cpu_profiler.end(); // egui Tessellate

            // egui image acquire
            let frame = &mut self.frames[self.frame_index];

            // CPU: image acquire
            self.cpu_profiler.begin("Acquire Image");
            let image_index = self
                .swapchain
                .acquire_next_image(frame.image_available_semaphore)?;
            self.cpu_profiler.end(); // Acquire Image

            // start rendering
            self.context.device.begin_command_buffer(
                frame.command_buffer,
                &vk::CommandBufferBeginInfo::default(),
            )?;

            // GPU: reset query pool, start frame
            self.gpu_timestamps
                .begin_frame(&self.context.device, frame.command_buffer);

            // image layout states

            self.context.transition_image_layout(
                frame.command_buffer,
                self.swapchain.images[image_index as usize],
                self.undefined_image_state,
                self.render_image_state,
                vk::ImageAspectFlags::COLOR,
            );
            self.context.transition_image_layout(
                frame.command_buffer,
                self.swapchain.depth_image,
                self.undefined_image_state,
                self.depth_attachment_state,
                vk::ImageAspectFlags::DEPTH,
            );

            // CPU: scene record / GPU: Geometry Pass
            self.cpu_profiler.begin("Record Scene");
            self.gpu_timestamps.begin_scope(
                &self.context.device,
                frame.command_buffer,
                "Geometry Pass",
            );

            self.context.begin_rendering(
                frame.command_buffer,
                self.swapchain.image_views[image_index as usize],
                self.swapchain.depth_image_view,
                vk::ClearColorValue {
                    float32: [0.01, 0.01, 0.4, 1.0],
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

            let mut camera_node: Option<&Node> = None;
            if !is_editor {
                for node in world.get_all_world_nodes() {
                    if node.get_component::<Camera>().is_some() {
                        camera_node = Some(node);
                    }
                }
            }
            if camera_node.is_none()
                && let Some(node) = world.get_global_node_with_component::<Camera>()
                && is_editor
            {
                camera_node = Some(node);
            }

            if let Some(camera_node) = camera_node {
                let aspect = swapchain_extent.width as f32 / swapchain_extent.height as f32;
                let transform = camera_node.get_component::<Transform>().unwrap();
                let camera = camera_node.get_component::<Camera>().unwrap();

                let model = Matrix4::from_scale(1.0);
                let camera_eye = Point3::new(
                    transform.global_position.x,
                    transform.global_position.y,
                    transform.global_position.z,
                );
                let rotated_forward = transform.calculate_global_forward();
                let rotated_up = transform.calculate_global_up();
                let look_at = Point3::new(
                    camera_eye.x + rotated_forward.x,
                    camera_eye.y + rotated_forward.y,
                    camera_eye.z + rotated_forward.z,
                );
                let view = Matrix4::look_at_rh(camera_eye, look_at, rotated_up);
                let projection = get_perspective_projection(camera, aspect);
                let mvp = projection * view * model;

                let mvp_bytes: [u8; 64] = transmute(mvp);
                let model_bytes: [u8; 64] = transmute(model);
                let mut push_constants = [0u8; 256];
                push_constants[0..64].copy_from_slice(&mvp_bytes);
                push_constants[64..128].copy_from_slice(&model_bytes);

                device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.model_pipeline,
                );

                device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    pipeline_layout,
                    0,
                    &[self.default_descriptor_set],
                    &[],
                );

                let mut light: Option<Light> = None;
                let mut light_transform: Option<Transform> = None;
                for node in world.get_all_nodes() {
                    light = node.get_component::<Light>().cloned();
                    if light.is_some() {
                        light_transform = node.get_component::<Transform>().cloned();
                        break;
                    }
                }

                for node in world.get_all_nodes_mut() {
                    let transform = node.get_component::<Transform>().cloned();
                    if let (Some(transform), Some(model_renderer)) =
                        (transform, node.get_component_mut::<ModelRenderer>())
                    {
                        let offset = [
                            transform.global_position.x,
                            transform.global_position.y,
                            transform.global_position.z,
                            0.0f32,
                        ];
                        let offset_bytes: [u8; 16] = transmute(offset);
                        push_constants[128..144].copy_from_slice(&offset_bytes);

                        let rotation = [
                            transform.global_rotation.v.x,
                            transform.global_rotation.v.y,
                            transform.global_rotation.v.z,
                            transform.global_rotation.s,
                        ];
                        let rotation_bytes: [u8; 16] = transmute(rotation);
                        push_constants[144..160].copy_from_slice(&rotation_bytes);

                        let scale = [
                            transform.global_scale.x,
                            transform.global_scale.y,
                            transform.global_scale.z,
                            0.0f32,
                        ];
                        let scale_bytes: [u8; 16] = transmute(scale);
                        push_constants[160..176].copy_from_slice(&scale_bytes);

                        let model_name = model_renderer.loaded_model.clone();

                        let model: Option<Handle<GpuModel>>;
                        // if let Some(model) = model_renderer.model_handle {
                        //     model = Some(model);
                        // } else {
                        //     let model_handle: Handle<GpuModel> = asset_server.load(model_name)?;
                        //     let model = asset_server.get(model_handle).unwrap().clone();
                        //     model_renderer.model_handle = Some(model_handle);
                        //     model = Some(model);
                        // }

                        {
                            let asset_server = asset_server.write().unwrap();
                            if model_renderer.model_handle.is_some() {
                                model = model_renderer.model_handle;
                            } else {
                                let model_handle: Handle<GpuModel> =
                                    asset_server.load(model_name)?;
                                model_renderer.model_handle = Some(model_handle);
                                model = Some(model_handle);
                            }

                            let model = model.unwrap();
                            let model = asset_server.get(model).unwrap().clone();
                            for mesh in model.meshes.clone() {
                                let mut mat_handle: Option<Handle<MaterialAsset>> = None;

                                // if theres no set material path, try load the meshes material
                                if model_renderer.material_path.is_empty() {
                                    {
                                        if let Some(&h) = model_renderer
                                            .mesh_material_handles
                                            .get(&mesh.material_name)
                                        {
                                            mat_handle = Some(h);
                                        } else {
                                            let h = asset_server.insert(MaterialAsset::default());
                                            model_renderer
                                                .mesh_material_handles
                                                .insert(mesh.material_name.clone(), h);
                                            mat_handle = Some(h);
                                        }
                                    };
                                } else {
                                    let material_handle = asset_server.load::<MaterialAsset>(
                                        model_renderer.material_path.clone(),
                                    );
                                    if material_handle.is_ok() {
                                        mat_handle = Some(material_handle.unwrap());
                                        model_renderer.material_handle = Some(mat_handle.unwrap());
                                    } else {
                                        println!("material not loaded from handle path");
                                        if let Some(ref mat) = model_renderer.material {
                                            match model_renderer.material_handle {
                                                Some(h) => mat_handle = Some(h),
                                                None => {
                                                    let h = asset_server.insert(mat.clone());
                                                    model_renderer.material_handle = Some(h);
                                                    mat_handle = Some(h);
                                                }
                                            }
                                        }
                                    }
                                }

                                let mat_handle = mat_handle.unwrap();

                                {
                                    let (
                                        albedo_name,
                                        metallic_name,
                                        roughness_name,
                                        normal_name,
                                        emissive_name,
                                    ) = {
                                        let mat = asset_server
                                            .get_cloned::<MaterialAsset>(mat_handle)
                                            .unwrap();
                                        (
                                            mat.albedo_texture_name.clone(),
                                            mat.metallic_texture_name.clone(),
                                            mat.roughness_texture_name.clone(),
                                            mat.normal_texture_name.clone(),
                                            mat.emissive_texture_name.clone(),
                                        )
                                    };

                                    let albedo_h = resolve_texture(&albedo_name, &asset_server);

                                    let metallic_h = resolve_texture(&metallic_name, &asset_server);
                                    let roughness_h =
                                        resolve_texture(&roughness_name, &asset_server);
                                    let normal_h = resolve_texture(&normal_name, &asset_server);
                                    let emissive_h = resolve_texture(&emissive_name, &asset_server);

                                    let mut mat = asset_server.get_mut(mat_handle).unwrap();

                                    mat.albedo_handle = if albedo_h.is_none() {
                                        let albedo_h = resolve_texture(
                                            &".engine/temp.png".to_string(),
                                            &asset_server,
                                        );
                                        albedo_h
                                    } else {
                                        albedo_h
                                    };
                                    mat.metallic_handle = metallic_h;
                                    mat.roughness_handle = roughness_h;
                                    mat.normal_handle = normal_h;
                                    mat.emissive_handle = emissive_h;

                                    mat.textures_resolved = true;
                                }

                                let mat = asset_server
                                    .get_cloned::<MaterialAsset>(mat_handle)
                                    .unwrap();

                                if let Some(albedo_h) = mat.albedo_handle {
                                    let albedo_texture =
                                        asset_server.get_cloned::<GpuTexture>(albedo_h).unwrap();
                                    let albedo_descriptor_set = albedo_texture.descriptor_set;

                                    device.cmd_bind_descriptor_sets(
                                        command_buffer,
                                        vk::PipelineBindPoint::GRAPHICS,
                                        pipeline_layout,
                                        0,
                                        &[albedo_descriptor_set],
                                        &[],
                                    );
                                }

                                let base_bytes: [u8; 16] = transmute(mat.base_color);
                                let metallic_bytes: [u8; 4] = f32::to_ne_bytes(mat.metallic);
                                let roughness_bytes: [u8; 4] = f32::to_ne_bytes(mat.roughness);
                                let emissive_bytes: [u8; 12] = transmute(mat.emissive);

                                push_constants[176..192].copy_from_slice(&base_bytes);
                                push_constants[192..196].copy_from_slice(&metallic_bytes);
                                push_constants[196..200].copy_from_slice(&roughness_bytes);
                                push_constants[208..220].copy_from_slice(&emissive_bytes);
                                push_constants[220..224].copy_from_slice(&[0u8; 4]);

                                if let (Some(light), Some(lt)) = (&light, &light_transform) {
                                    let light_pos = [
                                        lt.global_position.x,
                                        lt.global_position.y,
                                        lt.global_position.z,
                                        light.strength,
                                    ];
                                    let light_pos_bytes: [u8; 16] = transmute(light_pos);
                                    push_constants[224..240].copy_from_slice(&light_pos_bytes);
                                }

                                device.cmd_push_constants(
                                    command_buffer,
                                    pipeline_layout,
                                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                                    0,
                                    &push_constants,
                                );

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
                        }
                    }
                }
            }

            self.context.device.cmd_end_rendering(frame.command_buffer);

            // GPU: close Geometry Pass / CPU: close Record Scene
            self.gpu_timestamps
                .end_scope(&self.context.device, frame.command_buffer);
            self.cpu_profiler.end(); // Record Scene

            // CPU: egui draw  /  GPU: egui Pass
            self.cpu_profiler.begin("egui Draw");
            self.gpu_timestamps.begin_scope(
                &self.context.device,
                frame.command_buffer,
                "egui Pass",
            );

            let color_attachment = vk::RenderingAttachmentInfo::default()
                .image_view(self.swapchain.image_views[image_index as usize])
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(vk::AttachmentStoreOp::STORE);

            let rendering_info = vk::RenderingInfo::default()
                .render_area(vk::Rect2D::default().extent(self.swapchain.extent))
                .layer_count(1)
                .color_attachments(std::slice::from_ref(&color_attachment));

            self.context
                .device
                .cmd_begin_rendering(frame.command_buffer, &rendering_info);

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

            self.egui_renderer.egui_renderer.cmd_draw(
                frame.command_buffer,
                self.swapchain.extent,
                full_output.pixels_per_point,
                &clipped_primitives,
            )?;

            self.context.device.cmd_end_rendering(frame.command_buffer);

            // GPU: close egui Pass / CPU: close egui Draw
            self.gpu_timestamps
                .end_scope(&self.context.device, frame.command_buffer);
            self.cpu_profiler.end(); // egui Draw

            // Present transition
            self.context.transition_image_layout(
                frame.command_buffer,
                self.swapchain.images[image_index as usize],
                self.render_image_state,
                self.present_image_state,
                vk::ImageAspectFlags::COLOR,
            );

            // ── CPU: submit & present ─────────────────────────────────────────
            self.cpu_profiler.begin("Submit & Present");
            self.context
                .device
                .end_command_buffer(frame.command_buffer)?;

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

            // Close top-level CPU scopes
            self.cpu_profiler.end();
            self.cpu_profiler.end();

            // Resolve GPU timestamps
            self.context
                .device
                .wait_for_fences(&[frame.in_flight_fence], true, u64::MAX)?;
            let gpu_scopes = self.gpu_timestamps.resolve(&self.context.device);

            //  Build and store FrameData
            let cpu_scopes = self.cpu_profiler.drain();
            let frame_time_ms = frame_wall_start.elapsed().as_secs_f64() * 1000.0;
            let cpu_total_ms: f64 = cpu_scopes
                .iter()
                .filter(|s| s.depth == 0)
                .map(|s| s.duration_ms)
                .sum();
            let gpu_total_ms: f64 = gpu_scopes.iter().map(|s| s.duration_ms).sum();
            self.global_frame_counter += 1;
            self.frame_index = (self.frame_index + 1) % self.frames.len();

            // record frame data
            self.pending_frame_data = Some(FrameData {
                frame_index: self.global_frame_counter,
                frame_time_ms,
                cpu_scopes,
                gpu_scopes,
                cpu_total_ms,
                gpu_total_ms,
            });
            Ok(())
        }
    }

    pub fn prepare_egui(&mut self, window: &Window, world: &mut World, editor: &mut EditorStorage) {
        let raw_input = self.egui_renderer.egui_state.take_egui_input(window);
        self.egui_renderer.egui_ctx.begin_pass(raw_input);

        if let Some(frame_data) = self.pending_frame_data.take() {
            if !editor.profiler.paused {
                editor.profiler.history.push(frame_data);
            }
        }

        for system in &self.egui_renderer.sorted_ui_systems {
            (system.func)(&mut self.egui_renderer.egui_ctx);
        }
        for system in &self.egui_renderer.sorted_editor_ui_systems {
            (system.func)(&mut self.egui_renderer.egui_ctx, world, editor);
        }
    }

    pub fn window_event(&mut self, window: &Window, event: WindowEvent) -> bool {
        self.egui_renderer
            .egui_state
            .on_window_event(window, &event)
            .consumed
    }

    pub fn resize(&mut self) -> Result<()> {
        self.swapchain.resize()
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            if let Err(e) = self.context.device.device_wait_idle() {
                eprintln!(
                    "Warning: device_wait_idle failed during renderer drop: {:?}",
                    e
                );
            }

            // Destroy GPU timestamp query pool
            self.gpu_timestamps.destroy(&self.context.device);

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

fn resolve_texture(name: &String, server: &AssetServer) -> Option<Handle<GpuTexture>> {
    server.load_cached::<GpuTexture>(name).ok()
}
