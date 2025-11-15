use anyhow::Result;
use winit::event_loop::EventLoop;

use crate::app::{App, engine::ecs::ECSWorld};

pub mod app;
pub mod game;
pub mod tests;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let mut app = App::default();

    let mut world = ECSWorld::default();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop.run_app(&mut app)?;
    Ok(())
}
