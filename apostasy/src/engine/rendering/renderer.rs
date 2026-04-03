use std::mem::transmute;
use std::sync::RwLock;
use std::time::Instant;
use std::{collections::BTreeMap, sync::Arc};

use anyhow::Result;
use ash::vk::{self, DescriptorSet};
use cgmath::{Matrix4, Point3, SquareMatrix};
use egui::{Context, FontFamily};
use egui_ash_renderer::{DynamicRendering, Options};
use winit::{event::WindowEvent, window::Window};

use crate::engine::assets::handle::Handle;
use crate::engine::assets::server::AssetServer;
use crate::engine::nodes::components::collider::{Collider, CollisionEvents};
use crate::engine::nodes::components::terrain::{Terrain, TerrainChunkGpu, TerrainVertex};
use crate::engine::nodes::world::World;
use crate::engine::rendering::debug_renderer::{DebugLineVertex, DebugRenderer};
use crate::engine::rendering::models::gltf_loader::GltfLoader;
use crate::engine::rendering::models::material::MaterialAsset;
use crate::engine::rendering::models::model::GpuModel;
use crate::engine::rendering::models::shader::ShaderSpirv;
use crate::engine::rendering::models::texture::{GpuTexture, GpuTextureLoader};
use crate::engine::rendering::models::vertex::Vertex;
use crate::engine::rendering::pipeline_settings::PipelineSettings;
use crate::engine::rendering::profiler::{CpuProfiler, FrameData, GpuTimestampPool};
use crate::engine::{
    editor::{EditorStorage, style::style},
    nodes::{
        Node,
        components::{
            camera::{Camera, get_perspective_projection},
            light::Light,
            skybox::Skybox,
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
            FontFamily::Name("jetbrains".into()),
            vec!["jetbrains".to_owned()],
        );
        fonts.families.append(&mut new_font_family);
        fonts.font_data.insert(
            "jetbrains".to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../../../../res/.engine/fonts/JetBrainsMonoNerdFont-Medium.ttf"
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

/// A container for frames and their data
pub struct Frames {
    pub frame_index: usize,
    pub frames: Vec<Frame>,
}

/// A container for a pipeline and it's data
pub struct Pipeline {
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
}

/// A container for a descriptor and it's data
pub struct Descriptor {
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_set: vk::DescriptorSet,
}

/// A container for a UBO
pub struct Ubo {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
}

/// A container for the profilers render data
pub struct ProfilerInfo {
    pub gpu_timestamps: GpuTimestampPool,
    pub cpu_profiler: CpuProfiler,
    pub pending_frame_data: Option<FrameData>,
    pub global_frame_counter: u64,
}

/// A container for the states a texture/image can be in
pub struct ImageStates {
    pub undefined_image_state: ImageLayoutState,
    pub render_image_state: ImageLayoutState,
    pub depth_attachment_state: ImageLayoutState,
    pub present_image_state: ImageLayoutState,
}

pub struct Renderer {
    pub command_pool: vk::CommandPool,
    pub transfer_command_pool: vk::CommandPool,
    pub swapchain: Swapchain,
    pub context: Arc<RenderingContext>,
    pub egui_renderer: EguiRenderer,

    pub frames: Frames,
    pub pipeline: Pipeline,
    pub skybox_pipeline: vk::Pipeline,
    pub descriptor: Descriptor,
    pub ubo: Ubo,
    pub skybox_vertex_buffer: vk::Buffer,
    pub skybox_vertex_buffer_memory: vk::DeviceMemory,
    pub skybox_index_buffer: vk::Buffer,
    pub skybox_index_buffer_memory: vk::DeviceMemory,
    pub profiler_info: ProfilerInfo,
    pub image_states: ImageStates,

    pub debug_renderer: DebugRenderer,
    pub debug_pipeline_layout: vk::PipelineLayout,
}

impl Renderer {
    #[allow(unnecessary_transmutes)]
    pub fn new(
        context: Arc<RenderingContext>,
        window: Arc<Window>,
        asset_server: &mut Arc<RwLock<AssetServer>>,
        pipeline_settings: PipelineSettings,
    ) -> Result<Self> {
        let mut swapchain = Swapchain::new(context.clone(), window.clone())?;
        swapchain.resize()?;

        let mut asset_server = asset_server.write().unwrap();

        let mvs_handle: Handle<ShaderSpirv> = asset_server.load("shaders/model_vert.spv")?;
        let mfs_handle: Handle<ShaderSpirv> = asset_server.load("shaders/model_frag.spv")?;

        let model_vertex_shader = asset_server
            .get(mvs_handle)
            .unwrap()
            .create_module(&context.device)?;
        let model_fragment_shader = asset_server
            .get(mfs_handle)
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

            let model_pipeline = context.create_graphics_pipeline(
                model_vertex_shader,
                model_fragment_shader,
                swapchain.format,
                swapchain.depth_format,
                pipeline_layout,
                Default::default(),
                pipeline_settings,
            )?;

            let skybox_vs_handle: Handle<ShaderSpirv> =
                asset_server.load("shaders/skybox_vert.spv")?;
            let skybox_fs_handle: Handle<ShaderSpirv> =
                asset_server.load("shaders/skybox_frag.spv")?;

            let skybox_vertex_shader = asset_server
                .get(skybox_vs_handle)
                .unwrap()
                .create_module(&context.device)?;
            let skybox_fragment_shader = asset_server
                .get(skybox_fs_handle)
                .unwrap()
                .create_module(&context.device)?;

            let mut skybox_pipeline_settings = pipeline_settings;
            skybox_pipeline_settings.depth_settings.depth_test_enabled = false;
            skybox_pipeline_settings.rasterization_settings.cull_mode = vk::CullModeFlags::NONE;

            let skybox_pipeline = context.create_graphics_pipeline(
                skybox_vertex_shader,
                skybox_fragment_shader,
                swapchain.format,
                swapchain.depth_format,
                pipeline_layout,
                Default::default(),
                skybox_pipeline_settings,
            )?;

            let debug_vs_handle: Handle<ShaderSpirv> =
                asset_server.load("shaders/debug_line_vert.spv")?;
            let debug_fs_handle: Handle<ShaderSpirv> =
                asset_server.load("shaders/debug_line_frag.spv")?;

            let debug_vertex_shader = asset_server
                .get(debug_vs_handle)
                .unwrap()
                .create_module(&context.device)?;
            let debug_fragment_shader = asset_server
                .get(debug_fs_handle)
                .unwrap()
                .create_module(&context.device)?;

            let debug_vertex_binding = vk::VertexInputBindingDescription::default()
                .binding(0)
                .stride(std::mem::size_of::<DebugLineVertex>() as u32)
                .input_rate(vk::VertexInputRate::VERTEX);

            let debug_vertex_attrs = [
                vk::VertexInputAttributeDescription::default()
                    .binding(0)
                    .location(0)
                    .format(vk::Format::R32G32B32_SFLOAT)
                    .offset(0),
                vk::VertexInputAttributeDescription::default()
                    .binding(0)
                    .location(1)
                    .format(vk::Format::R32G32B32A32_SFLOAT)
                    .offset(std::mem::size_of::<[f32; 3]>() as u32),
            ];

            let debug_push_constant_range = vk::PushConstantRange::default()
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .offset(0)
                .size(64);

            let debug_pipeline_layout = context.device.create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default()
                    .push_constant_ranges(&[debug_push_constant_range]),
                None,
            )?;

            let debug_pipeline = context.create_debug_pipeline(
                debug_vertex_shader,
                debug_fragment_shader,
                swapchain.format,
                swapchain.depth_format,
                debug_pipeline_layout,
                &debug_vertex_binding,
                &debug_vertex_attrs,
                pipeline_settings.clone(),
            )?;

            let debug_renderer = DebugRenderer {
                lines: Vec::new(),
                vertex_buffer: vk::Buffer::null(),
                vertex_buffer_memory: vk::DeviceMemory::null(),
                vertex_capacity: 0,
                pipeline: debug_pipeline,
                enabled: true,
                pipeline_layout: debug_pipeline_layout,
            };

            context
                .device
                .destroy_shader_module(skybox_vertex_shader, None);
            context
                .device
                .destroy_shader_module(skybox_fragment_shader, None);

            context
                .device
                .destroy_shader_module(model_vertex_shader, None);
            context
                .device
                .destroy_shader_module(model_fragment_shader, None);

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

            let upload_buffer = |data: &[u8],
                                 usage: vk::BufferUsageFlags|
             -> Result<(vk::Buffer, vk::DeviceMemory)> {
                let size = data.len() as vk::DeviceSize;
                let (staging_buffer, staging_memory) = context.create_buffer(
                    size,
                    vk::BufferUsageFlags::TRANSFER_SRC,
                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                )?;

                let data_ptr = context.device.map_memory(
                    staging_memory,
                    0,
                    size,
                    vk::MemoryMapFlags::empty(),
                )? as *mut u8;
                std::ptr::copy_nonoverlapping(data.as_ptr(), data_ptr, data.len());
                context.device.unmap_memory(staging_memory);

                let (buffer, buffer_memory) = context.create_buffer(
                    size,
                    usage | vk::BufferUsageFlags::TRANSFER_DST,
                    vk::MemoryPropertyFlags::DEVICE_LOCAL,
                )?;

                let queue = context.queues[context.queue_families.transfer as usize];
                let cmd = context.begin_single_time_commands(transfer_command_pool);
                context.device.cmd_copy_buffer(
                    cmd,
                    staging_buffer,
                    buffer,
                    &[vk::BufferCopy::default().size(size)],
                );
                context.end_single_time_commands(cmd, queue, transfer_command_pool);

                context.device.destroy_buffer(staging_buffer, None);
                context.device.free_memory(staging_memory, None);
                Ok((buffer, buffer_memory))
            };

            let skybox_vertices = [
                Vertex {
                    position: [-1.0, -1.0, 0.0],
                    normal: [0.0, 0.0, 0.0],
                    tex_coord: [0.0, 0.0],
                },
                Vertex {
                    position: [1.0, -1.0, 0.0],
                    normal: [0.0, 0.0, 0.0],
                    tex_coord: [1.0, 0.0],
                },
                Vertex {
                    position: [1.0, 1.0, 0.0],
                    normal: [0.0, 0.0, 0.0],
                    tex_coord: [1.0, 1.0],
                },
                Vertex {
                    position: [-1.0, 1.0, 0.0],
                    normal: [0.0, 0.0, 0.0],
                    tex_coord: [0.0, 1.0],
                },
            ];
            let skybox_indices: [u32; 6] = [0, 1, 2, 2, 3, 0];

            let skybox_vertex_data = std::slice::from_raw_parts(
                skybox_vertices.as_ptr() as *const u8,
                skybox_vertices.len() * std::mem::size_of::<Vertex>(),
            );
            let skybox_index_data = std::slice::from_raw_parts(
                skybox_indices.as_ptr() as *const u8,
                skybox_indices.len() * std::mem::size_of::<u32>(),
            );

            let (skybox_vertex_buffer, skybox_vertex_buffer_memory) =
                upload_buffer(skybox_vertex_data, vk::BufferUsageFlags::VERTEX_BUFFER)?;
            let (skybox_index_buffer, skybox_index_buffer_memory) =
                upload_buffer(skybox_index_data, vk::BufferUsageFlags::INDEX_BUFFER)?;

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

            let white_texture = context.load_texture(
                "res/.engine/white.png",
                transfer_command_pool,
                descriptor_pool,
                descriptor_set_layout,
                default_ubo,
                pipeline_settings,
            )?;

            let default_descriptor_set = white_texture.descriptor_set;

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

            let image_states: ImageStates = ImageStates {
                undefined_image_state,
                render_image_state,
                depth_attachment_state,
                present_image_state,
            };

            asset_server.register_loader(GltfLoader::new(context.clone(), command_pool));

            asset_server.register_loader(GpuTextureLoader::new(
                context.clone(),
                command_pool,
                descriptor_pool,
                descriptor_set_layout,
                default_ubo,
                pipeline_settings,
            ));

            let profiler_info = ProfilerInfo {
                gpu_timestamps,
                cpu_profiler: CpuProfiler::new(),
                global_frame_counter: 0,
                pending_frame_data: None,
            };

            let ubo = Ubo {
                buffer: default_ubo,
                memory: default_ubo_mem,
            };
            let descriptor = Descriptor {
                descriptor_pool,
                descriptor_set_layout,
                descriptor_set: default_descriptor_set,
            };

            let pipeline = Pipeline {
                pipeline: model_pipeline,
                layout: pipeline_layout,
            };
            let frames = Frames {
                frame_index: 0,
                frames,
            };
            Ok(Self {
                frames,
                command_pool,
                swapchain,
                context,
                egui_renderer,
                transfer_command_pool,
                descriptor,
                pipeline,
                skybox_pipeline,
                ubo,
                skybox_vertex_buffer,
                skybox_vertex_buffer_memory,
                skybox_index_buffer,
                skybox_index_buffer_memory,
                profiler_info,
                image_states,

                debug_renderer,
                debug_pipeline_layout,
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
        self.profiler_info.cpu_profiler.begin("Frame Total");

        let frame = &mut self.frames.frames[self.frames.frame_index];
        unsafe {
            // CPU: fence wait
            self.profiler_info.cpu_profiler.begin("Fence Wait");
            self.context
                .device
                .wait_for_fences(&[frame.in_flight_fence], true, u64::MAX)?;
            self.profiler_info.cpu_profiler.end(); // Fence Wait

            self.context.device.reset_fences(&[frame.in_flight_fence])?;
            self.context
                .device
                .reset_command_buffer(frame.command_buffer, vk::CommandBufferResetFlags::empty())?;

            if self.swapchain.is_dirty {
                self.swapchain.resize()?;
                println!("Swapchain resized");
            }

            //  CPU: egui tessellation
            self.profiler_info.cpu_profiler.begin("egui Tessellate");

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

            self.profiler_info.cpu_profiler.end(); // egui Tessellate

            // egui image acquire
            let frame = &mut self.frames.frames[self.frames.frame_index];

            // CPU: image acquire
            self.profiler_info.cpu_profiler.begin("Acquire Image");
            let image_index = self
                .swapchain
                .acquire_next_image(frame.image_available_semaphore)?;
            self.profiler_info.cpu_profiler.end(); // Acquire Image

            // start rendering
            self.context.device.begin_command_buffer(
                frame.command_buffer,
                &vk::CommandBufferBeginInfo::default(),
            )?;

            // GPU: reset query pool, start frame
            self.profiler_info
                .gpu_timestamps
                .begin_frame(&self.context.device, frame.command_buffer);

            // image layout states
            self.context.transition_image_layout(
                frame.command_buffer,
                self.swapchain.images[image_index as usize],
                self.image_states.undefined_image_state,
                self.image_states.render_image_state,
                vk::ImageAspectFlags::COLOR,
            );
            self.context.transition_image_layout(
                frame.command_buffer,
                self.swapchain.depth_image,
                self.image_states.undefined_image_state,
                self.image_states.depth_attachment_state,
                vk::ImageAspectFlags::DEPTH,
            );

            // CPU: scene record / GPU: Geometry Pass
            self.profiler_info.cpu_profiler.begin("Record Scene");
            self.profiler_info.gpu_timestamps.begin_scope(
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
            let pipeline_layout = self.pipeline.layout;
            let swapchain_extent = self.swapchain.extent;

            // find the camera node
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

            if let Some(mut skybox_node) = world.get_node_with_component_mut::<Skybox>() {
                let skybox = skybox_node.get_component_mut::<Skybox>().unwrap();
                if skybox.texture_handle.is_none() {
                    let asset_server = asset_server.write().unwrap();
                    if let Ok(texture_handle) =
                        asset_server.load::<GpuTexture>(skybox.texture_path.clone())
                    {
                        skybox.texture_handle = Some(texture_handle);
                    } else {
                        println!("Skybox texture failed to load: {}", skybox.texture_path);
                    }
                }

                if let Some(texture_handle) = skybox.texture_handle {
                    let asset_server = asset_server.write().unwrap();
                    if let Some(texture) = asset_server.get_cloned::<GpuTexture>(texture_handle) {
                        device.cmd_bind_pipeline(
                            command_buffer,
                            vk::PipelineBindPoint::GRAPHICS,
                            self.skybox_pipeline,
                        );

                        let mut push_constants = [0u8; 256];
                        {
                            let (inv_projection, camera_rotation) = if let Some(camera_node) =
                                camera_node
                            {
                                let aspect =
                                    swapchain_extent.width as f32 / swapchain_extent.height as f32;
                                let transform = camera_node.get_component::<Transform>().unwrap();
                                let camera = camera_node.get_component::<Camera>().unwrap();
                                let projection = get_perspective_projection(camera, aspect);
                                let inv_projection =
                                    projection.invert().unwrap_or(Matrix4::from_scale(1.0));
                                let camera_rotation = Matrix4::from(transform.global_rotation);
                                (inv_projection, camera_rotation)
                            } else {
                                (Matrix4::from_scale(1.0), Matrix4::from_scale(1.0))
                            };

                            let inv_proj_bytes: [u8; 64] = transmute(inv_projection);
                            let camera_rotation_bytes: [u8; 64] = transmute(camera_rotation);
                            push_constants[0..64].copy_from_slice(&inv_proj_bytes);
                            push_constants[64..128].copy_from_slice(&camera_rotation_bytes);

                            if let Some(transform) = skybox_node.get_component::<Transform>() {
                                let rotation = Matrix4::from(transform.rotation);
                                let rotation_bytes: [u8; 64] = transmute(rotation);
                                push_constants[128..192].copy_from_slice(&rotation_bytes);
                            }
                        }
                        device.cmd_push_constants(
                            command_buffer,
                            pipeline_layout,
                            vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                            0,
                            &push_constants,
                        );
                        device.cmd_bind_descriptor_sets(
                            command_buffer,
                            vk::PipelineBindPoint::GRAPHICS,
                            pipeline_layout,
                            0,
                            &[texture.descriptor_set],
                            &[],
                        );
                        device.cmd_bind_vertex_buffers(
                            command_buffer,
                            0,
                            &[self.skybox_vertex_buffer],
                            &[0],
                        );
                        device.cmd_bind_index_buffer(
                            command_buffer,
                            self.skybox_index_buffer,
                            0,
                            vk::IndexType::UINT32,
                        );
                        device.cmd_draw_indexed(command_buffer, 6, 1, 0, 0, 0);
                    }
                }
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
                    self.pipeline.pipeline,
                );

                device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    pipeline_layout,
                    0,
                    &[self.descriptor.descriptor_set],
                    &[],
                );

                // find the first light in the scene
                let mut light: Option<Light> = None;
                let mut light_transform: Option<Transform> = None;
                for node in world.get_all_nodes() {
                    light = node.get_component::<Light>().cloned();
                    if light.is_some() {
                        light_transform = node.get_component::<Transform>().cloned();
                        break;
                    }
                }

                // write light into push constants once, shared by models and terrain
                if let (Some(l), Some(lt)) = (&light, &light_transform) {
                    let light_pos = [
                        lt.global_position.x,
                        lt.global_position.y,
                        lt.global_position.z,
                        l.strength,
                    ];
                    let light_pos_bytes: [u8; 16] = transmute(light_pos);
                    push_constants[224..240].copy_from_slice(&light_pos_bytes);
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

                // terrain: upload dirty chunks then draw
                let terrain_node_ids: Vec<u64> = world
                    .get_all_world_nodes()
                    .iter()
                    .filter(|n| n.has_component::<Terrain>())
                    .map(|n| n.id)
                    .collect();

                for terrain_node_id in terrain_node_ids {
                    let node = world.get_node_mut(terrain_node_id);
                    let node_transform = node.get_component::<Transform>().cloned();
                    let terrain = node.get_component_mut::<Terrain>().unwrap();

                    let chunk_count = terrain.chunks.len();
                    if terrain.gpu_chunks.len() < chunk_count {
                        terrain
                            .gpu_chunks
                            .resize_with(chunk_count, TerrainChunkGpu::default);
                    } else if terrain.gpu_chunks.len() > chunk_count {
                        for gpu in terrain.gpu_chunks.drain(chunk_count..) {
                            if gpu.vertex_buffer != vk::Buffer::null() {
                                self.context.device.destroy_buffer(gpu.vertex_buffer, None);
                                self.context
                                    .device
                                    .free_memory(gpu.vertex_buffer_memory, None);
                            }
                            if gpu.index_buffer != vk::Buffer::null() {
                                self.context.device.destroy_buffer(gpu.index_buffer, None);
                                self.context
                                    .device
                                    .free_memory(gpu.index_buffer_memory, None);
                            }
                        }
                    }

                    for (i, chunk) in terrain.chunks.iter_mut().enumerate() {
                        if !chunk.gpu_dirty {
                            continue;
                        }
                        let Some(mesh) = &chunk.mesh_handle else {
                            continue;
                        };
                        if mesh.vertices.is_empty() {
                            continue;
                        }

                        let gpu = &mut terrain.gpu_chunks[i];

                        // destroy old buffers before re-uploading
                        if gpu.vertex_buffer != vk::Buffer::null() {
                            self.context.device.destroy_buffer(gpu.vertex_buffer, None);
                            self.context
                                .device
                                .free_memory(gpu.vertex_buffer_memory, None);
                            self.context.device.destroy_buffer(gpu.index_buffer, None);
                            self.context
                                .device
                                .free_memory(gpu.index_buffer_memory, None);
                            *gpu = TerrainChunkGpu::default();
                        }
                        let verts_bytes = std::slice::from_raw_parts(
                            mesh.vertices.as_ptr() as *const u8,
                            mesh.vertices.len() * std::mem::size_of::<TerrainVertex>(),
                        );
                        let idx_bytes = std::slice::from_raw_parts(
                            mesh.indices.as_ptr() as *const u8,
                            mesh.indices.len() * std::mem::size_of::<u32>(),
                        );

                        let upload =
                            |data: &[u8],
                             usage: vk::BufferUsageFlags|
                             -> Result<(vk::Buffer, vk::DeviceMemory)> {
                                let size = data.len() as vk::DeviceSize;
                                let (staging, staging_mem) = self.context.create_buffer(
                                    size,
                                    vk::BufferUsageFlags::TRANSFER_SRC,
                                    vk::MemoryPropertyFlags::HOST_VISIBLE
                                        | vk::MemoryPropertyFlags::HOST_COHERENT,
                                )?;
                                let ptr = self.context.device.map_memory(
                                    staging_mem,
                                    0,
                                    size,
                                    vk::MemoryMapFlags::empty(),
                                )? as *mut u8;
                                std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
                                self.context.device.unmap_memory(staging_mem);

                                let (buffer, buffer_mem) = self.context.create_buffer(
                                    size,
                                    usage | vk::BufferUsageFlags::TRANSFER_DST,
                                    vk::MemoryPropertyFlags::DEVICE_LOCAL,
                                )?;
                                let queue = self.context.queues
                                    [self.context.queue_families.transfer as usize];
                                let cmd = self
                                    .context
                                    .begin_single_time_commands(self.transfer_command_pool);
                                self.context.device.cmd_copy_buffer(
                                    cmd,
                                    staging,
                                    buffer,
                                    &[vk::BufferCopy::default().size(size)],
                                );
                                self.context.end_single_time_commands(
                                    cmd,
                                    queue,
                                    self.transfer_command_pool,
                                );
                                self.context.device.destroy_buffer(staging, None);
                                self.context.device.free_memory(staging_mem, None);
                                Ok((buffer, buffer_mem))
                            };

                        if let Ok((vb, vbm)) =
                            upload(verts_bytes, vk::BufferUsageFlags::VERTEX_BUFFER)
                        {
                            gpu.vertex_buffer = vb;
                            gpu.vertex_buffer_memory = vbm;
                        }
                        if let Ok((ib, ibm)) = upload(idx_bytes, vk::BufferUsageFlags::INDEX_BUFFER)
                        {
                            gpu.index_buffer = ib;
                            gpu.index_buffer_memory = ibm;
                        }
                        gpu.index_count = mesh.indices.len() as u32;
                        chunk.gpu_dirty = false;
                    }

                    // pack transform into push constants
                    let offset = node_transform
                        .as_ref()
                        .map(|t| {
                            [
                                t.global_position.x,
                                t.global_position.y,
                                t.global_position.z,
                                0.0f32,
                            ]
                        })
                        .unwrap_or([0.0; 4]);
                    let rotation = node_transform
                        .as_ref()
                        .map(|t| {
                            [
                                t.global_rotation.v.x,
                                t.global_rotation.v.y,
                                t.global_rotation.v.z,
                                t.global_rotation.s,
                            ]
                        })
                        .unwrap_or([0.0, 0.0, 0.0, 1.0]);
                    let scale = node_transform
                        .as_ref()
                        .map(|t| [t.global_scale.x, t.global_scale.y, t.global_scale.z, 0.0f32])
                        .unwrap_or([1.0, 1.0, 1.0, 0.0]);

                    let offset_bytes: [u8; 16] = transmute(offset);
                    let rotation_bytes: [u8; 16] = transmute(rotation);
                    let scale_bytes: [u8; 16] = transmute(scale);
                    push_constants[128..144].copy_from_slice(&offset_bytes);
                    push_constants[144..160].copy_from_slice(&rotation_bytes);
                    push_constants[160..176].copy_from_slice(&scale_bytes);

                    // flat base colour until terrain texturing is added
                    let base_color: [f32; 4] = [0.6, 0.55, 0.45, 1.0];
                    let base_bytes: [u8; 16] = transmute(base_color);
                    let metallic_bytes: [u8; 4] = f32::to_ne_bytes(0.0f32);
                    let roughness_bytes: [u8; 4] = f32::to_ne_bytes(0.9f32);
                    push_constants[176..192].copy_from_slice(&base_bytes);
                    push_constants[192..196].copy_from_slice(&metallic_bytes);
                    push_constants[196..200].copy_from_slice(&roughness_bytes);

                    device.cmd_push_constants(
                        command_buffer,
                        pipeline_layout,
                        vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                        0,
                        &push_constants,
                    );
                    device.cmd_bind_descriptor_sets(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        pipeline_layout,
                        0,
                        &[self.descriptor.descriptor_set],
                        &[],
                    );

                    for gpu in terrain.gpu_chunks.iter() {
                        if gpu.vertex_buffer == vk::Buffer::null() || gpu.index_count == 0 {
                            continue;
                        }
                        device.cmd_bind_vertex_buffers(
                            command_buffer,
                            0,
                            &[gpu.vertex_buffer],
                            &[0],
                        );
                        device.cmd_bind_index_buffer(
                            command_buffer,
                            gpu.index_buffer,
                            0,
                            vk::IndexType::UINT32,
                        );
                        device.cmd_draw_indexed(command_buffer, gpu.index_count, 1, 0, 0, 0);
                    }
                }
                if self.debug_renderer.enabled {
                    self.debug_renderer.clear();

                    let collision_events = world
                        .global_nodes
                        .iter()
                        .find_map(|n| n.get_component::<CollisionEvents>())
                        .cloned()
                        .unwrap_or_default();

                    for node in world.get_all_nodes() {
                        if let (Some(transform), Some(collider)) = (
                            node.get_component::<Transform>(),
                            node.get_component::<Collider>(),
                        ) {
                            self.debug_renderer.draw_collider(
                                collider,
                                transform.global_position,
                                transform.global_rotation,
                                transform.global_scale,
                                &collision_events,
                                &node.name,
                            );
                        }
                    }

                    // Upload and draw — unchanged from your current code
                    if !self.debug_renderer.lines.is_empty() {
                        let vertices: Vec<DebugLineVertex> = self
                            .debug_renderer
                            .lines
                            .iter()
                            .flat_map(|l| {
                                [
                                    DebugLineVertex {
                                        position: [l.start.x, l.start.y, l.start.z],
                                        color: l.color,
                                    },
                                    DebugLineVertex {
                                        position: [l.end.x, l.end.y, l.end.z],
                                        color: l.color,
                                    },
                                ]
                            })
                            .collect();

                        let byte_size = (vertices.len() * std::mem::size_of::<DebugLineVertex>())
                            as vk::DeviceSize;

                        if vertices.len() > self.debug_renderer.vertex_capacity {
                            if self.debug_renderer.vertex_buffer != vk::Buffer::null() {
                                self.context
                                    .device
                                    .destroy_buffer(self.debug_renderer.vertex_buffer, None);
                                self.context
                                    .device
                                    .free_memory(self.debug_renderer.vertex_buffer_memory, None);
                            }
                            let (buf, mem) = self.context.create_buffer(
                                byte_size,
                                vk::BufferUsageFlags::VERTEX_BUFFER,
                                vk::MemoryPropertyFlags::HOST_VISIBLE
                                    | vk::MemoryPropertyFlags::HOST_COHERENT,
                            )?;
                            self.debug_renderer.vertex_buffer = buf;
                            self.debug_renderer.vertex_buffer_memory = mem;
                            self.debug_renderer.vertex_capacity = vertices.len();
                        }

                        let ptr = self.context.device.map_memory(
                            self.debug_renderer.vertex_buffer_memory,
                            0,
                            byte_size,
                            vk::MemoryMapFlags::empty(),
                        )? as *mut DebugLineVertex;
                        std::ptr::copy_nonoverlapping(vertices.as_ptr(), ptr, vertices.len());
                        self.context
                            .device
                            .unmap_memory(self.debug_renderer.vertex_buffer_memory);

                        device.cmd_bind_pipeline(
                            command_buffer,
                            vk::PipelineBindPoint::GRAPHICS,
                            self.debug_renderer.pipeline,
                        );
                        device.cmd_bind_vertex_buffers(
                            command_buffer,
                            0,
                            &[self.debug_renderer.vertex_buffer],
                            &[0],
                        );

                        let mvp_bytes: [u8; 64] = transmute(mvp);
                        device.cmd_push_constants(
                            command_buffer,
                            self.debug_renderer.pipeline_layout,
                            vk::ShaderStageFlags::VERTEX,
                            0,
                            &mvp_bytes,
                        );
                        device.cmd_draw(command_buffer, vertices.len() as u32, 1, 0, 0);
                    }
                }
            }

            self.context.device.cmd_end_rendering(frame.command_buffer);

            // GPU: close Geometry Pass / CPU: close Record Scene
            self.profiler_info
                .gpu_timestamps
                .end_scope(&self.context.device, frame.command_buffer);
            self.profiler_info.cpu_profiler.end(); // Record Scene

            // CPU: egui draw  /  GPU: egui Pass
            self.profiler_info.cpu_profiler.begin("egui Draw");
            self.profiler_info.gpu_timestamps.begin_scope(
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
            self.profiler_info
                .gpu_timestamps
                .end_scope(&self.context.device, frame.command_buffer);
            self.profiler_info.cpu_profiler.end(); // egui Draw

            // Present transition
            self.context.transition_image_layout(
                frame.command_buffer,
                self.swapchain.images[image_index as usize],
                self.image_states.render_image_state,
                self.image_states.present_image_state,
                vk::ImageAspectFlags::COLOR,
            );

            //  CPU: submit & present
            self.profiler_info.cpu_profiler.begin("Submit & Present");
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
            self.profiler_info.cpu_profiler.end();
            self.profiler_info.cpu_profiler.end();

            // Resolve GPU timestamps
            self.context
                .device
                .wait_for_fences(&[frame.in_flight_fence], true, u64::MAX)?;
            let gpu_scopes = self
                .profiler_info
                .gpu_timestamps
                .resolve(&self.context.device);

            //  Build and store FrameData
            let cpu_scopes = self.profiler_info.cpu_profiler.drain();
            let frame_time_ms = frame_wall_start.elapsed().as_secs_f64() * 1000.0;
            let cpu_total_ms: f64 = cpu_scopes
                .iter()
                .filter(|s| s.depth == 0)
                .map(|s| s.duration_ms)
                .sum();
            let gpu_total_ms: f64 = gpu_scopes.iter().map(|s| s.duration_ms).sum();
            self.profiler_info.global_frame_counter += 1;
            self.frames.frame_index = (self.frames.frame_index + 1) % self.frames.frames.len();

            // record frame data
            self.profiler_info.pending_frame_data = Some(FrameData {
                frame_index: self.profiler_info.global_frame_counter,
                frame_time_ms,
                cpu_scopes,
                gpu_scopes,
                cpu_total_ms,
                gpu_total_ms,
            });
            Ok(())
        }
    }

    pub fn rebuild_pipeline(
        &mut self,
        asset_server: &mut Arc<RwLock<AssetServer>>,
        pipeline_settings: PipelineSettings,
    ) -> Result<()> {
        unsafe { self.context.device.device_wait_idle()? };
        let mut asset_server = asset_server.write().unwrap();

        let mvs_handle: Handle<ShaderSpirv> = asset_server.load("shaders/model_vert.spv")?;
        let mfs_handle: Handle<ShaderSpirv> = asset_server.load("shaders/model_frag.spv")?;
        let skybox_vs_handle: Handle<ShaderSpirv> = asset_server.load("shaders/skybox_vert.spv")?;
        let skybox_fs_handle: Handle<ShaderSpirv> = asset_server.load("shaders/skybox_frag.spv")?;
        let debug_vs_handle: Handle<ShaderSpirv> =
            asset_server.load("shaders/debug_line_vert.spv")?;
        let debug_fs_handle: Handle<ShaderSpirv> =
            asset_server.load("shaders/debug_line_frag.spv")?;

        let model_vertex_shader = asset_server
            .get(mvs_handle)
            .unwrap()
            .create_module(&self.context.device)?;
        let model_fragment_shader = asset_server
            .get(mfs_handle)
            .unwrap()
            .create_module(&self.context.device)?;
        let skybox_vertex_shader = asset_server
            .get(skybox_vs_handle)
            .unwrap()
            .create_module(&self.context.device)?;
        let skybox_fragment_shader = asset_server
            .get(skybox_fs_handle)
            .unwrap()
            .create_module(&self.context.device)?;
        let debug_vertex_shader = asset_server
            .get(debug_vs_handle)
            .unwrap()
            .create_module(&self.context.device)?;
        let debug_fragment_shader = asset_server
            .get(debug_fs_handle)
            .unwrap()
            .create_module(&self.context.device)?;

        asset_server.register_loader(GpuTextureLoader::new(
            self.context.clone(),
            self.command_pool,
            self.descriptor.descriptor_pool,
            self.descriptor.descriptor_set_layout,
            self.ubo.buffer,
            pipeline_settings,
        ));

        let results = asset_server.reload_all::<GpuTexture>();
        for (path, result) in results {
            if let Err(e) = result {
                eprintln!("Failed to reload {}: {e}", path.display());
            }
        }

        let mut skybox_pipeline_settings = pipeline_settings;
        skybox_pipeline_settings.depth_settings.depth_test_enabled = false;
        skybox_pipeline_settings.rasterization_settings.cull_mode = vk::CullModeFlags::NONE;

        let debug_vertex_binding = vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(std::mem::size_of::<DebugLineVertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX);

        let debug_vertex_attrs = [
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(0),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32A32_SFLOAT)
                .offset(std::mem::size_of::<[f32; 3]>() as u32),
        ];

        unsafe {
            let new_pipeline = self.context.create_graphics_pipeline(
                model_vertex_shader,
                model_fragment_shader,
                self.swapchain.format,
                self.swapchain.depth_format,
                self.pipeline.layout,
                Default::default(),
                pipeline_settings,
            )?;
            let new_skybox_pipeline = self.context.create_graphics_pipeline(
                skybox_vertex_shader,
                skybox_fragment_shader,
                self.swapchain.format,
                self.swapchain.depth_format,
                self.pipeline.layout,
                Default::default(),
                skybox_pipeline_settings,
            )?;
            let new_debug_pipeline = self.context.create_debug_pipeline(
                debug_vertex_shader,
                debug_fragment_shader,
                self.swapchain.format,
                self.swapchain.depth_format,
                self.debug_renderer.pipeline_layout,
                &debug_vertex_binding,
                &debug_vertex_attrs,
                pipeline_settings,
            )?;

            self.context
                .device
                .destroy_shader_module(model_vertex_shader, None);
            self.context
                .device
                .destroy_shader_module(model_fragment_shader, None);
            self.context
                .device
                .destroy_shader_module(skybox_vertex_shader, None);
            self.context
                .device
                .destroy_shader_module(skybox_fragment_shader, None);
            self.context
                .device
                .destroy_shader_module(debug_vertex_shader, None);
            self.context
                .device
                .destroy_shader_module(debug_fragment_shader, None);

            let old_pipeline = std::mem::replace(&mut self.pipeline.pipeline, new_pipeline);
            self.context.device.destroy_pipeline(old_pipeline, None);

            let old_skybox_pipeline =
                std::mem::replace(&mut self.skybox_pipeline, new_skybox_pipeline);
            self.context
                .device
                .destroy_pipeline(old_skybox_pipeline, None);

            let old_debug_pipeline =
                std::mem::replace(&mut self.debug_renderer.pipeline, new_debug_pipeline);
            self.context
                .device
                .destroy_pipeline(old_debug_pipeline, None);
        }

        Ok(())
    }

    pub fn prepare_egui(&mut self, window: &Window, world: &mut World, editor: &mut EditorStorage) {
        let raw_input = self.egui_renderer.egui_state.take_egui_input(window);
        self.egui_renderer.egui_ctx.begin_pass(raw_input);

        if let Some(frame_data) = self.profiler_info.pending_frame_data.take() {
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
            self.profiler_info
                .gpu_timestamps
                .destroy(&self.context.device);

            self.frames.frames.drain(..).for_each(|frame| {
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
                .destroy_pipeline(self.pipeline.pipeline, None);
            self.context
                .device
                .destroy_pipeline(self.skybox_pipeline, None);
            self.context
                .device
                .destroy_buffer(self.skybox_vertex_buffer, None);
            self.context
                .device
                .free_memory(self.skybox_vertex_buffer_memory, None);
            self.context
                .device
                .destroy_buffer(self.skybox_index_buffer, None);
            self.context
                .device
                .free_memory(self.skybox_index_buffer_memory, None);
            self.context
                .device
                .destroy_pipeline_layout(self.pipeline.layout, None);
        }
    }
}

fn resolve_texture(name: &String, server: &AssetServer) -> Option<Handle<GpuTexture>> {
    server.load_cached::<GpuTexture>(name).ok()
}
