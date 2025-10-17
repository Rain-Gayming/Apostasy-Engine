pub mod engine;

use crate::{
    app::engine::Engine,
    game::{initialize_game, Game},
};
use winit::application::ApplicationHandler;

#[derive(Default)]
pub struct App {
    pub engine: Option<Engine>,
    pub game: Option<Game>,
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.game = Some(initialize_game());
        self.engine = Some(
            Engine::new(
                event_loop,
                self.game.as_ref().unwrap().player.camera.clone(),
            )
            .unwrap(),
        );
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
        if let Some(engine) = &mut self.engine {
            engine.window_event(event_loop, event.clone());

            if let Some(game) = &mut self.game {
                game.window_event(event, &mut engine.input_manager);
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(engine) = &mut self.engine {
            engine.request_redraw();
        }
        if let Some(game) = &mut self.game {
            game.update(&mut self.engine.as_mut().unwrap().renderer);
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
                game.device_event(event, &mut engine.input_manager);
            }
        }
    }
}
