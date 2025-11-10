use std::collections::HashMap;

use anyhow::Result;
use winit::event_loop::EventLoop;

use crate::app::{App, engine::ecs::systems::*};

pub mod app;
pub mod game;
pub mod tests;

// fn main() -> Result<()> {
//     tracing_subscriber::fmt::init();
//     let mut app = App::default();
//
//
//
//
//     let event_loop = EventLoop::new()?;
//     event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
//     event_loop.run_app(&mut app)?;
//     Ok(())
// }
fn main() {
    let mut scheduler = Scheduler::default();

    scheduler.add_system(foo);
    scheduler.add_system(bar);

    scheduler.add_resource(12i32);
    scheduler.add_resource("Hello, world!");

    scheduler.run();
}

fn foo(mut int: ResMut<i32>) {
    *int += 1;
}

fn bar(statement: Res<&'static str>, num: Res<i32>) {
    println!("{} My lucky number is: {}", *statement, *num);
}
