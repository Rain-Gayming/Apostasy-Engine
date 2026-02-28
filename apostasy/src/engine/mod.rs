use crate::{
    self as apostasy,
    engine::{
        nodes::components::collider::{Collider, CollisionEvents},
        windowing::cursor_manager::CursorManager,
    },
};
use anyhow::Result;
use apostasy_macros::fixed_update;
use cgmath::{Vector3, Zero, num_traits::clamp};
use std::{collections::HashMap, sync::Arc};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId},
    event_loop::{ControlFlow, EventLoop},
};

use winit::{
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};

use crate::engine::{
    editor::EditorStorage,
    nodes::{
        Node, World,
        components::camera::Camera,
        components::transform::Transform,
        components::velocity::{Velocity, apply_velocity},
    },
    rendering::{
        models::model::{ModelLoader, ModelRenderer, load_models},
        queue_families::queue_family_picker::single_queue_family,
        renderer::Renderer,
        rendering_context::{RenderingContext, RenderingContextAttributes},
    },
    timer::EngineTimer,
    windowing::WindowManager,
};

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
    pub world: World,
    pub editor: EditorStorage,
    pub model_loader: ModelLoader,
}

impl Engine {
    pub fn new(event_loop: &ActiveEventLoop) -> Result<Self> {
        let primary_window = Arc::new(
            event_loop.create_window(
                Window::default_attributes()
                    .with_title("Apostasy")
                    .with_visible(true),
            )?,
        );
        let primary_window_id = primary_window.id();
        let windows = HashMap::from([(primary_window_id, primary_window.clone())]);

        let rendering_context = Arc::new(RenderingContext::new(RenderingContextAttributes {
            compatability_window: &primary_window,
            queue_family_picker: single_queue_family,
        })?);
        let renderers = windows
            .iter()
            .map(|(id, window)| {
                let renderer = Renderer::new(rendering_context.clone(), window.clone()).unwrap();
                (*id, renderer)
            })
            .collect::<HashMap<WindowId, Renderer>>();

        let timer = EngineTimer::new();

        let mut world = World::new();

        let mut camera = Node::new();
        camera.name = "editor_camera".to_string();
        camera.add_component(Camera::default());
        camera.add_component(Transform::default());
        camera.add_component(Velocity::default());

        let mut cube = Node::new();
        cube.name = "cube".to_string();
        cube.add_component(Transform::default());
        cube.add_component(ModelRenderer::default());
        cube.add_component(Collider::new_static(
            Vector3::new(0.5, 0.5, 0.5),
            Vector3::new(0.0, 0.0, 0.0),
        ));
        cube.get_component_mut::<Transform>().unwrap().position = Vector3::new(0.0, 0.0, 0.0);

        world.add_node(cube);
        world.add_global_node(camera);

        let mut cursor_manager = Node::new();
        cursor_manager.name = "cursor_manager".to_string();
        cursor_manager.add_component(CursorManager::default());
        world.add_global_node(cursor_manager);

        let mut events_node = Node::new();
        events_node.name = "CollisionEvents".to_string();
        events_node.add_component(CollisionEvents::new());
        world.add_global_node(events_node);

        let window_manager = WindowManager {
            windows,
            primary_window_id,
        };

        let mut model_loader = ModelLoader::default();
        let editor = EditorStorage::default();

        load_models(&mut model_loader, &rendering_context);

        Ok(Self {
            renderers,
            rendering_context,
            window_manager,
            timer,
            world,
            editor,
            model_loader,
        })
    }

    pub fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(renderer) = self.renderers.get_mut(&window_id) {
            let window = self.window_manager.windows.get(&window_id).unwrap();
            renderer.window_event(window, event.clone());
        }

        self.world.input_manager.handle_input_event(event.clone());

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
                if let Some(renderer) = self.renderers.get_mut(&window_id) {
                    for window in &self.window_manager.windows {
                        renderer.prepare_egui(window.1, &mut self.world, &mut self.editor);
                    }

                    let _ = renderer.render(&self.world, &mut self.model_loader);
                }
            }
            WindowEvent::KeyboardInput { .. } => {}

            _ => (),
        }
    }

    pub fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        self.world.input_manager.handle_device_event(event.clone());
        if self
            .world
            .input_manager
            .is_mousebind_active("editor_camera_look")
        {
            if !self.world.is_world_hovered {
                return;
            }
            let cursor_manager = self
                .world
                .get_global_node_with_component_mut::<CursorManager>();
            let cursor_manager = cursor_manager
                .unwrap()
                .get_component_mut::<CursorManager>()
                .unwrap();
            cursor_manager.grab_cursor(&mut self.window_manager);
        } else {
            let cursor_manager = self
                .world
                .get_global_node_with_component_mut::<CursorManager>();
            let cursor_manager = cursor_manager
                .unwrap()
                .get_component_mut::<CursorManager>()
                .unwrap();
            cursor_manager.ungrab_cursor(&mut self.window_manager);
        }
    }

    pub fn request_redraw(&mut self) {
        self.world.update();
        self.world.fixed_update(self.timer.tick().fixed_dt);

        for window in &self.window_manager.windows {
            window.1.request_redraw();
        }
        self.world.input_manager.clear_actions();
        self.world.late_update();
    }

    pub fn create_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        attributes: WindowAttributes,
    ) -> Result<WindowId> {
        let window = Arc::new(event_loop.create_window(attributes)?);
        let window_id = window.id();

        let renderer = Renderer::new(self.rendering_context.clone(), window)?;
        self.renderers.insert(window_id, renderer);
        Ok(window_id)
    }
}

#[fixed_update]
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

        velocity.direction *= delta_time;
        apply_velocity(velocity, camera_transform);
        velocity.direction = Vector3::zero();
    }
}
