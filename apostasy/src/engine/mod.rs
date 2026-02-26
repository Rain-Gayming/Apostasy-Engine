use crate::{
    self as apostasy,
    engine::{
        nodes::components::{
            collider::{Collider, CollisionEvent, CollisionEvents},
            physics::Physics,
            player::Player,
            raycast::Raycast,
        },
        windowing::{
            cursor_manager::CursorManager,
            input_manager::{KeyAction, KeyBind, MouseBind},
        },
    },
};
use anyhow::Result;
use apostasy_macros::{fixed_update, input};
use cgmath::{Vector3, num_traits::clamp};
use std::{collections::HashMap, sync::Arc};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, MouseButton},
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
    windowing::{WindowManager, input_manager::InputManager},
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
        camera.name = "camera".to_string();
        camera.add_component(Camera::default());
        camera.add_component(Transform::default());

        let mut player = Node::new();
        player.name = "player".to_string();
        player.add_component(Transform::default());
        player.add_component(Velocity::default());
        player.add_component(Physics::default());
        player.add_component(Collider::default());
        player.add_component(Raycast::default());
        player.add_component(Player::default());
        let transform = player.get_component_mut::<Transform>().unwrap();
        transform.position = Vector3::new(0.0, 0.0, -10.0);
        player.get_component_mut::<Raycast>().unwrap().direction = -transform.calculate_up();

        player.add_child(camera);

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
        world.add_node(player);

        let mut cursor_manager = Node::new();
        cursor_manager.name = "cursor_manager".to_string();
        cursor_manager.add_component(CursorManager::default());
        world.add_node(cursor_manager);

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
        let mut input_manager = InputManager::default();
        input_manager.register_keybind(
            KeyBind::new(PhysicalKey::Code(KeyCode::KeyW), KeyAction::Hold),
            "forward",
        );
        input_manager.register_keybind(
            KeyBind::new(PhysicalKey::Code(KeyCode::KeyS), KeyAction::Hold),
            "backward",
        );
        input_manager.register_keybind(
            KeyBind::new(PhysicalKey::Code(KeyCode::KeyA), KeyAction::Hold),
            "left",
        );
        input_manager.register_keybind(
            KeyBind::new(PhysicalKey::Code(KeyCode::KeyD), KeyAction::Hold),
            "right",
        );
        input_manager.register_keybind(
            KeyBind::new(PhysicalKey::Code(KeyCode::KeyE), KeyAction::Hold),
            "up",
        );
        input_manager.register_keybind(
            KeyBind::new(PhysicalKey::Code(KeyCode::KeyQ), KeyAction::Hold),
            "down",
        );

        input_manager.register_mousebind(
            MouseBind::new(MouseButton::Right, KeyAction::Hold),
            "editor_camera_look",
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

        self.input_manager.handle_input_event(event.clone());

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
        self.input_manager.handle_device_event(event.clone());
        let cursor_manager = self.world.get_node_with_component_mut::<CursorManager>();
        let cursor_manager = cursor_manager.get_component_mut::<CursorManager>().unwrap();
        if self.input_manager.is_mousebind_active("editor_camera_look") {
            cursor_manager.grab_cursor(&mut self.window_manager);
        } else {
            cursor_manager.ungrab_cursor(&mut self.window_manager);
        }
    }

    pub fn request_redraw(&mut self) {
        self.world.update();
        self.world.fixed_update(self.timer.tick().fixed_dt);
        self.world.input(&mut self.input_manager);

        for window in &self.window_manager.windows {
            window.1.request_redraw();
        }
        self.input_manager.clear_actions();
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
    let is_grabbed = {
        let cursor_manager = world.get_node_with_component::<CursorManager>();
        cursor_manager
            .get_component::<CursorManager>()
            .unwrap()
            .is_grabbed
    };

    if !is_grabbed {
        return;
    }

    let mut all = world.get_all_nodes_mut();

    let player_node = all.iter_mut().find(|n| n.name == "player").unwrap();
    let player_transform = player_node.get_component_mut::<Transform>().unwrap() as *mut Transform;
    let player_velocity = player_node.get_component_mut::<Velocity>().unwrap() as *mut Velocity;

    let camera_node = all.iter_mut().find(|n| n.name == "camera").unwrap();
    let camera_transform = camera_node.get_component_mut::<Transform>().unwrap() as *mut Transform;

    let (player_transform, player_velocity, camera_transform) = unsafe {
        (
            &mut *player_transform,
            &mut *player_velocity,
            &mut *camera_transform,
        )
    };

    player_transform.yaw += -input_manager.mouse_delta.0 as f32;
    camera_transform.pitch += -input_manager.mouse_delta.1 as f32;
    camera_transform.pitch = clamp(camera_transform.pitch, -89.0, 89.0);

    player_transform.calculate_rotation();
    camera_transform.calculate_rotation();

    let direction = player_transform.rotation
        * input_manager.input_vector_3d("right", "left", "up", "down", "backward", "forward");

    player_velocity.add_velocity(direction);
}
#[fixed_update]
pub fn fixed_update_handle(world: &mut World, delta_time: f32) {
    let player_name = "player";
    let camera_name = "camera";

    let (player_transform_snap, raycast_snap) = {
        let player = world.get_node_with_name(player_name);
        (
            player.get_component::<Transform>().unwrap().clone(),
            player.get_component::<Raycast>().unwrap().clone(),
        )
    };
    let hit = raycast_snap.cast(&player_transform_snap, world, player_name);

    let mut all = world.get_all_nodes_mut();

    let player_node = all.iter_mut().find(|n| n.name == player_name).unwrap();
    let transform = player_node.get_component_mut::<Transform>().unwrap() as *mut Transform;
    let velocity = player_node.get_component_mut::<Velocity>().unwrap() as *mut Velocity;

    let camera_node = all.iter_mut().find(|n| n.name == camera_name).unwrap();
    let camera_transform = camera_node.get_component_mut::<Transform>().unwrap() as *mut Transform;

    let (transform, velocity, camera_transform) =
        unsafe { (&mut *transform, &mut *velocity, &mut *camera_transform) };

    if hit.is_some() {
        velocity.direction.y = 0.0;
    }

    velocity.direction *= delta_time;
    apply_velocity(velocity, transform);
    velocity.direction = Vector3::new(0.0, 0.0, 0.0);
}
