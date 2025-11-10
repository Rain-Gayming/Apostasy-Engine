use std::collections::HashMap;

use anyhow::Result;
use winit::event_loop::EventLoop;

use crate::app::{App, engine::ecs::systems::*};

pub mod app;
pub mod game;
pub mod tests;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let mut app = App::default();

    let mut scheduler = Scheduler {
        systems: vec![],
        resources: HashMap::default(),
    };

    scheduler.add_system(foo);
    scheduler.add_resource(12i32);

    scheduler.run();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop.run_app(&mut app)?;
    Ok(())
}

pub fn foo(int: i32) {
    println!("int! {int}");
}
