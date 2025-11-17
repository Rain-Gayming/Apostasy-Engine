pub mod engine;

use crate::{
    app::engine::{
        Engine,
        ecs::{
            ECSWorld, components::velocity_component::VelocityComponent, systems::SystemCallType,
        },
    },
    game::{Game, initialize_game},
};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

pub struct App {
    pub engine: Engine,
    pub game: Game,
    pub world: ECSWorld,
}

impl App {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        Self {
            engine: Engine::new(event_loop).unwrap(),
            game: initialize_game(),
            world: ECSWorld::default(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(_) => {
                self.world.scheduler.run(SystemCallType::WindowChanged);
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                self.world.scheduler.run(SystemCallType::WindowChanged);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.world.scheduler.run(SystemCallType::Input);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.engine.request_redraw();
        // self.game.update(&mut self.engine.renderer);
        self.world
            .create_entity()
            .with_component::<VelocityComponent>(VelocityComponent::default());
        self.world.run(SystemCallType::Update);
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        self.engine.device_event(event.clone());

        self.game.device_event(event);
        self.world.scheduler.run(SystemCallType::Input);
    }
}
