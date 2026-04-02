use crate::{
    self as apostasy,
    engine::{
        assets::server::AssetServer,
        nodes::{
            components::{
                camera::get_perspective_projection,
                raycast::{pick, ray_from_mouse},
            },
            scene_serialization::SceneLoader,
            world::World,
        },
        rendering::{
            models::{material::MaterialLoader, shader::ShaderLoader},
            pipeline_settings::PipelineSettings,
        },
        windowing::cursor_manager::CursorManager,
    },
    log,
};
use anyhow::Result;
use apostasy_macros::{editor_fixed_update, editor_ui};
use cgmath::{Matrix4, Quaternion, Vector2, Vector3, Zero, num_traits::clamp};
use egui::Context;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Instant,
};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, MouseButton},
    event_loop::{ControlFlow, EventLoop},
};

use winit::{
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};

use crate::engine::{
    editor::{
        EditorStorage,
        terrain_editor::{TerrainEditMode, TerrainEditTool},
    },
    nodes::{
        components::camera::Camera,
        components::terrain::Terrain,
        components::transform::Transform,
        components::velocity::{Velocity, apply_velocity},
    },
    rendering::{
        queue_families::queue_family_picker::single_queue_family,
        renderer::Renderer,
        rendering_context::{RenderingContext, RenderingContextAttributes},
    },
    timer::EngineTimer,
    windowing::WindowManager,
};

pub mod assets;
pub mod editor;
pub mod nodes;
pub mod rendering;
pub mod timer;
pub mod windowing;

/// Render application
pub struct Application {
    engine: Option<Engine>,
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.engine = Some(Engine::new(event_loop).unwrap());
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let Some(engine) = self.engine.as_mut() {
            engine.window_event(event_loop, window_id, event.clone());
        }
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let Some(engine) = self.engine.as_mut() {
            engine.device_event(event_loop, device_id, event.clone());
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(engine) = &mut self.engine {
            engine.request_redraw();
        }
    }

    fn suspended(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        event_loop.exit();
    }
}

pub fn start_app() -> Result<()> {
    tracing_subscriber::fmt::init();
    let mut app = Application { engine: None };

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app)?;

    Ok(())
}
/// The render engine, contains all the data for rendering, windowing and their logic
pub struct Engine {
    pub renderers: HashMap<WindowId, Renderer>,
    pub rendering_context: Arc<RenderingContext>,
    pub window_manager: WindowManager,
    pub timer: EngineTimer,
    pub world: Arc<RwLock<World>>,
    pub editor: EditorStorage,
    pub asset_server: Arc<RwLock<AssetServer>>,
    pub pipeline_settings: PipelineSettings,
    pending_windows: Vec<(WindowId, Arc<Window>)>,
    renderers_initialized: bool,
}

impl Engine {
    pub fn new(event_loop: &ActiveEventLoop) -> Result<Self> {
        let primary_window = Arc::new(
            event_loop.create_window(
                Window::default_attributes()
                    .with_decorations(false)
                    .with_transparent(true)
                    .with_resizable(true)
                    .with_title("Apostasy Engine")
                    .with_visible(true),
            )?,
        );
        let primary_window_id = primary_window.id();
        let windows = HashMap::from([(primary_window_id, primary_window.clone())]);

        let rendering_context = Arc::new(RenderingContext::new(RenderingContextAttributes {
            compatability_window: &primary_window,
            queue_family_picker: single_queue_family,
        })?);

        let timer = EngineTimer::new();

        let mut world = World::new();
        world.setup_default_global_nodes();

        let window_manager = WindowManager {
            windows,
            primary_window_id,
        };

        let asset_server = Arc::new(RwLock::new(AssetServer::new("res/")));
        {
            let mut asset_server = asset_server.write().unwrap();
            asset_server.register_loader(ShaderLoader);
            asset_server.register_loader(MaterialLoader);
            asset_server.register_loader(SceneLoader);
        }

        let world = Arc::new(RwLock::new(world));
        let editor =
            EditorStorage::default(asset_server.clone(), world.clone(), Default::default());

        let pending_windows = vec![(primary_window_id, primary_window.clone())];

        Ok(Self {
            renderers: HashMap::new(),
            rendering_context,
            window_manager,
            timer,
            world,
            editor,
            asset_server,
            pipeline_settings: PipelineSettings::default(),
            pending_windows,
            renderers_initialized: false,
        })
    }

    pub fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(renderer) = self.renderers.get_mut(&window_id) {
            let window = self.window_manager.windows.get(&window_id).unwrap();
            renderer.window_event(window, event.clone());
        }

        {
            let mut world = self.world.write().unwrap();
            world.input_manager.handle_input_event(event.clone());
        }

        match event.clone() {
            WindowEvent::Resized(_size) => {
                if let Some(renderer) = self.renderers.get_mut(&window_id) {
                    renderer.resize().unwrap();
                }
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                if let Some(renderer) = self.renderers.get_mut(&window_id) {
                    renderer.resize().unwrap();
                }
            }
            WindowEvent::RedrawRequested => {
                if !self.renderers_initialized && !self.pending_windows.is_empty() {
                    let pending = std::mem::take(&mut self.pending_windows);
                    for (id, window) in pending {
                        match Renderer::new(
                            self.rendering_context.clone(),
                            window,
                            &mut self.asset_server,
                            self.pipeline_settings,
                        ) {
                            Ok(renderer) => {
                                self.renderers.insert(id, renderer);
                            }
                            Err(e) => {
                                eprintln!("Renderer init failed, deferring: {e}");
                                return; // try again next frame
                            }
                        }
                    }
                    self.renderers_initialized = true;
                }
                self.render_frame(window_id);
                if self.editor.should_close {
                    // persist layout before exiting
                    self.editor.save_layout();
                    event_loop.exit();
                }
            }
            WindowEvent::KeyboardInput { .. } => {}

            WindowEvent::CloseRequested => {
                // save editor layout if available
                self.editor.save_layout();
                event_loop.exit();
            }

            _ => (),
        }
    }

    pub fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        {
            let mut world = self.world.write().unwrap();
            world.input_manager.handle_device_event(event.clone());
            if world
                .input_manager
                .is_mousebind_active("editor_camera_look")
            {
                if !world.is_world_hovered {
                    return;
                }
                let cursor_manager = world.get_global_node_with_component_mut::<CursorManager>();
                let cursor_manager = cursor_manager
                    .unwrap()
                    .get_component_mut::<CursorManager>()
                    .unwrap();
                cursor_manager.grab_cursor(&mut self.window_manager);
            } else {
                let cursor_manager = world.get_global_node_with_component_mut::<CursorManager>();
                let cursor_manager = cursor_manager
                    .unwrap()
                    .get_component_mut::<CursorManager>()
                    .unwrap();
                cursor_manager.ungrab_cursor(&mut self.window_manager);
            }
        }
    }

    pub fn update(&mut self) {
        let mut world = self.world.write().unwrap();
        world.update();

        let delta_time = self.timer.tick().fixed_dt;
        if self.editor.is_editor_open {
            world.editor_fixed_update(delta_time);
        } else {
            world.fixed_update(delta_time);
        }

        if self.editor.should_update_renderer {
            self.pipeline_settings = self.editor.pipeline_settings.clone();

            for renderer in self.renderers.values_mut() {
                if let Err(e) = renderer.rebuild_pipeline(&mut self.asset_server, self.pipeline_settings) {
                    eprintln!("Failed to rebuild pipeline for renderer: {e}");
                }
                log!("Updating pipeline");
            }

            self.editor.should_update_renderer = false;
        }

        for window in &self.window_manager.windows {
            window.1.request_redraw();
            world.window_size = Vector2::new(
                window.1.inner_size().width as f32,
                window.1.inner_size().height as f32,
            );
        }

        world.input_manager.clear_actions();
        world.late_update();
    }

    pub fn request_redraw(&mut self) {
        self.update();
    }

    pub fn render_frame(&mut self, window_id: WindowId) {
        if !self.renderers_initialized && !self.pending_windows.is_empty() {
            let pending = std::mem::take(&mut self.pending_windows);
            for (id, window) in pending {
                match Renderer::new(
                    self.rendering_context.clone(),
                    window,
                    &mut self.asset_server,
                    self.pipeline_settings,
                ) {
                    Ok(renderer) => {
                        self.renderers.insert(id, renderer);
                    }
                    Err(e) => {
                        eprintln!("Renderer init failed, deferring: {e}");
                        return; // try again next frame
                    }
                }
            }
            self.renderers_initialized = true;
        }

        if let Some(renderer) = self.renderers.get_mut(&window_id) {
            {
                let mut world = self.world.write().unwrap();
                for window in &self.window_manager.windows {
                    renderer.prepare_egui(window.1, &mut world, &mut self.editor);

                    if self.editor.should_close {
                        self.editor.save_layout();
                        // Note: caller should handle event_loop exit
                    }
                }
            }

            {
                let mut world = self.world.write().unwrap();
                let _ = renderer.render(
                    &mut world,
                    &self.asset_server,
                    self.editor.is_editor_open,
                );
            }
        }
    }

    pub fn create_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        attributes: WindowAttributes,
    ) -> Result<WindowId> {
        let window = Arc::new(event_loop.create_window(attributes)?);
        let window_id = window.id();

        let renderer = Renderer::new(
            self.rendering_context.clone(),
            window,
            &mut self.asset_server,
            self.pipeline_settings,
        )?;
        self.renderers.insert(window_id, renderer);
        Ok(window_id)
    }
}

#[editor_fixed_update]
pub fn editor_camera_handle(world: &mut World, delta_time: f32) {
    if !world
        .get_global_node_with_component::<CursorManager>()
        .unwrap()
        .get_component::<CursorManager>()
        .unwrap()
        .is_grabbed
        || !world.is_world_hovered
    {
        return;
    }

    let mouse_delta = world.input_manager.mouse_delta;
    let input_dir = world
        .input_manager
        .input_vector_3d("right", "left", "up", "down", "backward", "forward");

    let editor_camera = world.get_global_node_with_component_mut::<Camera>();

    if let Some(editor_camera) = editor_camera {
        let (camera_transform, velocity) =
            editor_camera.get_components_mut::<(&mut Transform, &mut Velocity)>();
        camera_transform.rotation_euler.y -= mouse_delta.0 as f32;
        camera_transform.rotation_euler.x = clamp(
            camera_transform.rotation_euler.x - mouse_delta.1 as f32,
            -89.0,
            89.0,
        );

        camera_transform.calculate_rotation();

        let direction = camera_transform.global_rotation * input_dir;

        velocity.add_velocity(direction * delta_time);

        apply_velocity(velocity, camera_transform);
        velocity.direction = Vector3::zero();
    }
}
#[editor_fixed_update]
pub fn editor_camera_mouse_handle(world: &mut World, delta_time: f32) {
    if !world.is_world_hovered {
        return;
    }

    let is_camera_move_active = world
        .input_manager
        .is_mousebind_active("editor_camera_move");
    let mouse_delta = world.input_manager.mouse_delta;
    let scroll_delta = world.input_manager.scroll_delta;

    let editor_camera = world.get_global_node_with_component_mut::<Camera>();

    if let Some(editor_camera) = editor_camera {
        let (camera_transform, velocity) =
            editor_camera.get_components_mut::<(&mut Transform, &mut Velocity)>();

        let mut direction = Vector3::zero();

        if is_camera_move_active {
            direction += camera_transform.calculate_global_right() * mouse_delta.0 as f32 * 2.0;
            direction -= camera_transform.calculate_global_up() * mouse_delta.1 as f32 * 2.0;
        }

        if scroll_delta.1 != 0.0 {
            direction += camera_transform.calculate_global_forward() * scroll_delta.1 * 15.0;
        }

        velocity.add_velocity(direction * delta_time);

        apply_velocity(velocity, camera_transform);
        velocity.direction = Vector3::zero();
    }
}

#[editor_ui]
pub fn raycast_visualiser(_context: &mut Context, world: &mut World, editor: &mut EditorStorage) {
    if !world.is_world_hovered {
        return;
    }

    let is_left_mouse_pressed = world.input_manager.is_mousebind_active("left_mouse");
    let is_left_mouse_held = world.input_manager.mouse_held.contains(&MouseButton::Left);
    let mouse_position = world.input_manager.mouse_position;

    let aspect = world.window_size.x / world.window_size.y;
    let window_size = world.window_size;

    let (projection, camera_position, camera_rotation) =
        if let Some(editor_camera) = world.get_global_node_with_component::<Camera>() {
            let camera_transform = editor_camera.get_component::<Transform>().unwrap();
            let camera = editor_camera.get_component::<Camera>().unwrap();

            (
                get_perspective_projection(camera, aspect),
                camera_transform.position,
                camera_transform.rotation,
            )
        } else {
            return;
        };

    if is_left_mouse_held {
        if editor.is_terrain_editor_open {
            let now = Instant::now();
            let elapsed = now
                .duration_since(editor.terrain_editor_settings.last_paint)
                .as_secs_f32()
                .min(0.05);

            if elapsed > 0.0 {
                match editor.terrain_editor_settings.edit_tool {
                    TerrainEditTool::Edit => {
                        if let Some((terrain_node_id, chunk_index, vertex_x, vertex_z)) =
                            raycast_terrain_edit_target(
                                mouse_position.x as f32,
                                mouse_position.y as f32,
                                window_size.x,
                                window_size.y,
                                projection,
                                camera_position,
                                camera_rotation,
                                world,
                            )
                        {
                            let node = world.get_node_mut(terrain_node_id);
                            let (terrain, _) =
                                node.get_components_mut::<(&mut Terrain, &mut Transform)>();
                            let delta = match editor.terrain_editor_settings.edit_mode {
                                TerrainEditMode::Raise => {
                                    editor.terrain_editor_settings.edit_strength * elapsed
                                }
                                TerrainEditMode::Lower => {
                                    -editor.terrain_editor_settings.edit_strength * elapsed
                                }
                            };

                            editor.terrain_editor_settings.last_paint = now;
                            terrain.selected_chunk = chunk_index as u32;
                            terrain.selected_vertex_x = vertex_x;
                            terrain.selected_vertex_z = vertex_z;
                            terrain.apply_brush(
                                chunk_index,
                                vertex_x,
                                vertex_z,
                                editor.terrain_editor_settings.brush_radius,
                                delta,
                            );
                        }
                    }
                    TerrainEditTool::Smooth => {
                        if let Some((terrain_node_id, chunk_index, vertex_x, vertex_z)) =
                            raycast_terrain_edit_target(
                                mouse_position.x as f32,
                                mouse_position.y as f32,
                                window_size.x,
                                window_size.y,
                                projection,
                                camera_position,
                                camera_rotation,
                                world,
                            )
                        {
                            let node = world.get_node_mut(terrain_node_id);
                            let (terrain, _) =
                                node.get_components_mut::<(&mut Terrain, &mut Transform)>();

                            let strength =
                                (editor.terrain_editor_settings.edit_strength * elapsed * 0.05)
                                    .clamp(0.0, 1.0);

                            editor.terrain_editor_settings.last_paint = now;
                            terrain.selected_chunk = chunk_index as u32;
                            terrain.selected_vertex_x = vertex_x;
                            terrain.selected_vertex_z = vertex_z;
                            terrain.smooth_brush(
                                chunk_index,
                                vertex_x,
                                vertex_z,
                                editor.terrain_editor_settings.brush_radius,
                                strength,
                            );
                        }
                    }
                    TerrainEditTool::PaintNew => {
                        // Create a new chunk at the clicked plane position, but do not paint it.
                        if let Some((terrain_node_id, hit_world)) = raycast_terrain_plane_target(
                            mouse_position.x as f32,
                            mouse_position.y as f32,
                            window_size.x,
                            window_size.y,
                            projection,
                            camera_position,
                            camera_rotation,
                            world,
                        ) {
                            let node = world.get_node_mut(terrain_node_id);
                            let (terrain, transform) =
                                node.get_components_mut::<(&mut Terrain, &mut Transform)>();
                            let (origin, vertex_x, vertex_z) =
                                terrain.world_point_to_chunk_origin(transform, hit_world);
                            let chunk_index = terrain.add_chunk(origin);

                            editor.terrain_editor_settings.last_paint = now;
                            terrain.selected_chunk = chunk_index as u32;
                            terrain.selected_vertex_x = vertex_x;
                            terrain.selected_vertex_z = vertex_z;
                        }
                    }
                    TerrainEditTool::Delete => {}
                }
            }
        }
    }

    if is_left_mouse_pressed && editor.is_terrain_editor_open {
        match editor.terrain_editor_settings.edit_tool {
            TerrainEditTool::Delete => {
                // Delete the chunk tile under the click position.
                if let Some((terrain_node_id, hit_world)) = raycast_terrain_plane_target(
                    mouse_position.x as f32,
                    mouse_position.y as f32,
                    window_size.x,
                    window_size.y,
                    projection,
                    camera_position,
                    camera_rotation,
                    world,
                ) {
                    let node = world.get_node_mut(terrain_node_id);
                    let (terrain, transform) =
                        node.get_components_mut::<(&mut Terrain, &mut Transform)>();
                    let (origin, _, _) = terrain.world_point_to_chunk_origin(transform, hit_world);
                    if let Some(chunk_index) = terrain.chunk_index_for_origin(origin) {
                        terrain.delete_chunk(chunk_index);
                    }
                }
            }
            TerrainEditTool::PaintNew => {
                if let Some((terrain_node_id, hit_world)) = raycast_terrain_plane_target(
                    mouse_position.x as f32,
                    mouse_position.y as f32,
                    window_size.x,
                    window_size.y,
                    projection,
                    camera_position,
                    camera_rotation,
                    world,
                ) {
                    let node = world.get_node_mut(terrain_node_id);
                    let (terrain, transform) =
                        node.get_components_mut::<(&mut Terrain, &mut Transform)>();
                    let (origin, vertex_x, vertex_z) =
                        terrain.world_point_to_chunk_origin(transform, hit_world);
                    let chunk_index = terrain.add_chunk(origin);

                    editor.terrain_editor_settings.last_paint = Instant::now();
                    terrain.selected_chunk = chunk_index as u32;
                    terrain.selected_vertex_x = vertex_x;
                    terrain.selected_vertex_z = vertex_z;
                }
            }
            _ => {}
        }
    }

    if is_left_mouse_pressed {
        if let Some(hit) = pick(
            mouse_position.x as f32,
            mouse_position.y as f32,
            window_size.x,
            window_size.y,
            projection,
            camera_position,
            camera_rotation,
            &world.get_all_nodes(),
            "camera",
        ) {
            println!("Hit: {} at distance {:.2}", hit.node_name, hit.distance);
            let node_hit = world.get_node_with_name(hit.node_name.as_str());
            if let Some(node_hit) = node_hit {
                editor.selected_node = Some(node_hit.id);
            }
        }
    }
}

fn ray_plane_intersection(
    origin: Vector3<f32>,
    direction: Vector3<f32>,
    plane_y: f32,
) -> Option<(Vector3<f32>, f32)> {
    if direction.y.abs() < 1e-6 {
        return None;
    }

    let t = (plane_y - origin.y) / direction.y;
    if t < 0.0 {
        return None;
    }

    Some((origin + direction * t, t))
}

fn raycast_terrain_edit_target(
    mouse_x: f32,
    mouse_y: f32,
    viewport_width: f32,
    viewport_height: f32,
    projection: Matrix4<f32>,
    camera_position: Vector3<f32>,
    camera_rotation: Quaternion<f32>,
    world: &World,
) -> Option<(u64, usize, u32, u32)> {
    let (origin, direction) = ray_from_mouse(
        mouse_x,
        mouse_y,
        viewport_width,
        viewport_height,
        projection,
        camera_position,
        camera_rotation,
    );

    let mut best_hit: Option<(f32, u64, usize, u32, u32)> = None;

    for node in world.get_all_nodes() {
        if !node.has_component::<Terrain>() {
            continue;
        }

        let transform = match node.get_component::<Transform>() {
            Some(transform) => transform,
            None => continue,
        };

        let terrain = match node.get_component::<Terrain>() {
            Some(terrain) => terrain,
            None => continue,
        };

        if let Some((hit_world, distance)) =
            ray_plane_intersection(origin, direction, transform.global_position.y)
        {
            if let Some((chunk_index, vertex_x, vertex_z)) =
                terrain.world_point_to_vertex(transform, hit_world)
            {
                let add_hit = match &best_hit {
                    Some((best_distance, _, _, _, _)) => distance < *best_distance,
                    None => true,
                };

                if add_hit {
                    best_hit = Some((distance, node.id, chunk_index, vertex_x, vertex_z));
                }
            }
        }
    }

    best_hit.map(|(_, node_id, chunk_index, vertex_x, vertex_z)| {
        (node_id, chunk_index, vertex_x, vertex_z)
    })
}

fn raycast_terrain_plane_target(
    mouse_x: f32,
    mouse_y: f32,
    viewport_width: f32,
    viewport_height: f32,
    projection: Matrix4<f32>,
    camera_position: Vector3<f32>,
    camera_rotation: Quaternion<f32>,
    world: &World,
) -> Option<(u64, Vector3<f32>)> {
    let (origin, direction) = ray_from_mouse(
        mouse_x,
        mouse_y,
        viewport_width,
        viewport_height,
        projection,
        camera_position,
        camera_rotation,
    );

    let mut best_hit: Option<(f32, u64, Vector3<f32>)> = None;

    for node in world.get_all_nodes() {
        if !node.has_component::<Terrain>() {
            continue;
        }

        let transform = match node.get_component::<Transform>() {
            Some(transform) => transform,
            None => continue,
        };

        if let Some((hit_world, distance)) =
            ray_plane_intersection(origin, direction, transform.global_position.y)
        {
            let add_hit = match &best_hit {
                Some((best_distance, _, _)) => distance < *best_distance,
                None => true,
            };

            if add_hit {
                best_hit = Some((distance, node.id, hit_world));
            }
        }
    }

    best_hit.map(|(_, node_id, hit_world)| (node_id, hit_world))
}
