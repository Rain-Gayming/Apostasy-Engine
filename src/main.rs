use std::any::{Any, TypeId};
use std::sync::Arc;

use anyhow::Result;
use cgmath::Vector3;
use winit::event_loop::{ControlFlow, EventLoop};

use crate::app::App;
use crate::app::engine::ecs::components::position_component::PositionComponent;
use crate::app::engine::ecs::components::velocity_component::VelocityComponent;
use crate::app::engine::ecs::query;
use crate::app::engine::ecs::resource::{ResMut, Resource};
use crate::app::engine::ecs::systems::SystemCallType;
use crate::app::engine::renderer::{Renderer, render, resize, update_depth_buffer};
use crate::app::engine::rendering_context::{
    RenderingContext, RenderingContextAttributes, queue_family_picker,
};

pub mod app;
pub mod game;
pub mod tests;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let event_loop = EventLoop::new()?;

    let mut app = App::new(&event_loop);

    let window = Arc::new(event_loop.create_window(Default::default())?);
    let rendering_context = Arc::new(RenderingContext::new(RenderingContextAttributes {
        compatability_window: &window,
        queue_family_picker: queue_family_picker::single_queue_family,
    })?);

    let renderer = Renderer::new(rendering_context, window).unwrap();

    app.world.add_resource(renderer);

    // call start systems
    app.world.add_system(SystemCallType::Update, render);
    app.world.add_system(SystemCallType::WindowChanged, resize);
    app.world
        .add_system(SystemCallType::WindowChanged, update_depth_buffer);

    // TODO: make event loop run start somehow?
    app.world.run(SystemCallType::Start);

    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app)?;
    Ok(())
}
