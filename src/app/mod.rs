pub mod engine;

use crate::{
    app::engine::{
        Engine,
        ecs::{ECSWorld, components::velocity_component::VelocityComponent},
    },
    game::{Game, initialize_game},
};
use winit::{application::ApplicationHandler, event::WindowEvent};

#[derive(Default)]
pub struct App {
    pub engine: Option<Engine>,
    pub game: Option<Game>,
    pub world: Option<ECSWorld>,
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.game = Some(initialize_game());
        self.engine = Some(Engine::new(event_loop).unwrap());
        self.world = Some(ECSWorld::default());
    }

    fn suspended(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.engine = None;
        self.game = None
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(_) => {
                // self.renderer.resize().unwrap();
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                // self.renderer.resize().unwrap();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // send input over to the game
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(engine) = &mut self.engine {
            engine.request_redraw();
        }
        if let Some(game) = &mut self.game {
            game.update(&mut self.engine.as_mut().unwrap().renderer);
        }
        if let Some(world) = &mut self.world {
            world
                .create_entity()
                .with_component::<VelocityComponent>(VelocityComponent::default());
            world.run();
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let Some(engine) = &mut self.engine {
            engine.device_event(event.clone());

            if let Some(game) = &mut self.game {
                game.device_event(event);
            }
        }
    }
}
