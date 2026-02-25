use crate::{
    self as apostasy,
    engine::windowing::input_manager::{KeyAction, KeyBind, handle_input_event, register_keybind},
};
use anyhow::Result;
use apostasy_macros::{fixed_update, input};
use cgmath::{Vector3, num_traits::clamp};
use std::{collections::HashMap, sync::Arc};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
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
        camera::Camera,
        transform::{Transform, calculate_rotation},
        velocity::{Velocity, add_velocity, apply_velocity},
    },
    rendering::{
        models::model::{ModelLoader, ModelRenderer, load_models},
        queue_families::queue_family_picker::single_queue_family,
        renderer::Renderer,
        rendering_context::{RenderingContext, RenderingContextAttributes},
    },
    timer::EngineTimer,
    windowing::{
        WindowManager,
        input_manager::{InputManager, clear_actions, handle_device_event, input_vector_3d},
    },
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
    pub input_manager: InputManager,
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
        camera.add_component(Camera::default());
        camera.add_component(Transform::default());
        camera.add_component(Velocity::default());
        camera.get_component_mut::<Transform>().unwrap().position = Vector3::new(0.0, 0.0, -10.0);

        let mut cube = Node::new();
        cube.add_component(Transform::default());
        cube.add_component(ModelRenderer::default());
        cube.get_component_mut::<Transform>().unwrap().position = Vector3::new(0.0, 0.0, 0.0);

        world.add_node(camera);
        world.add_node(cube);

        let window_manager = WindowManager {
            windows,
            primary_window_id,
        };

        let mut model_loader = ModelLoader::default();
        let editor = EditorStorage::default();

        load_models(&mut model_loader, &rendering_context);
        let mut input_manager = InputManager::default();
        register_keybind(
            &mut input_manager,
            KeyBind::new(PhysicalKey::Code(KeyCode::KeyW), KeyAction::Hold),
            "forward",
        );
        register_keybind(
            &mut input_manager,
            KeyBind::new(PhysicalKey::Code(KeyCode::KeyS), KeyAction::Hold),
            "backward",
        );
        register_keybind(
            &mut input_manager,
            KeyBind::new(PhysicalKey::Code(KeyCode::KeyA), KeyAction::Hold),
            "left",
        );
        register_keybind(
            &mut input_manager,
            KeyBind::new(PhysicalKey::Code(KeyCode::KeyD), KeyAction::Hold),
            "right",
        );
        register_keybind(
            &mut input_manager,
            KeyBind::new(PhysicalKey::Code(KeyCode::KeyE), KeyAction::Hold),
            "up",
        );
        register_keybind(
            &mut input_manager,
            KeyBind::new(PhysicalKey::Code(KeyCode::KeyQ), KeyAction::Hold),
            "down",
        );

        Ok(Self {
            renderers,
            rendering_context,
            window_manager,
            timer,
            world,
            editor,
            model_loader,
            input_manager,
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

        handle_input_event(&mut self.input_manager, event.clone());

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
                        renderer.prepare_egui(window.1);
                    }

                    let _ = renderer.render(&self.world, &mut self.model_loader);
                }
            }
            WindowEvent::KeyboardInput { .. } => {
                self.world.input(&mut self.input_manager);
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
        handle_device_event(&mut self.input_manager, event.clone());
        self.world.input(&mut self.input_manager);
    }

    pub fn request_redraw(&mut self) {
        self.world.update();
        self.world.fixed_update(self.timer.tick().fixed_dt);
        for window in &self.window_manager.windows {
            window.1.request_redraw();
        }
        clear_actions(&mut self.input_manager);
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

#[input]
pub fn input_handle(world: &mut World, input_manager: &mut InputManager) {
    let nodes = world.get_all_nodes_mut();

    let mut camera: Option<&mut Node> = None;
    for node in nodes {
        if node.get_component::<Camera>().is_some() {
            camera = Some(node);
        }
    }

    if camera.is_none() {
        return;
    }
    let camera = camera.unwrap();
    let (transform, velocity) =
        camera.get_components_mut::<(Option<&mut Transform>, Option<&mut Velocity>)>();
    let velocity = velocity.unwrap();
    let transform = transform.unwrap();

    let direction = transform.rotation
        * input_vector_3d(
            input_manager,
            "right",
            "left",
            "up",
            "down",
            "backward",
            "forward",
        );

    add_velocity(velocity, direction * 0.01);
    transform.yaw += -input_manager.mouse_delta.0 as f32;
    transform.pitch += -input_manager.mouse_delta.1 as f32;

    calculate_rotation(transform);
    transform.pitch = clamp(transform.pitch, -89.0, 89.0);
}

#[fixed_update]
pub fn fixed_update_handle(world: &mut World, delta_time: f32) {
    let camera = world.get_node_with_component_mut::<Camera>();

    let (transform, velocity) =
        camera.get_components_mut::<(Option<&mut Transform>, Option<&mut Velocity>)>();
    let velocity = velocity.unwrap();
    let transform = transform.unwrap();

    apply_velocity(velocity, transform);
    velocity.direction = Vector3::new(0.0, 0.0, 0.0);
}
