// use std::collections::HashMap;
//
// use anyhow::Result;
// use winit::event_loop::EventLoop;

use crate::app::engine::ecs::{
    ECSWorld,
    components::{position::PositionComponent, velocity::VelocityComponent},
    entities::Entity,
    query::Query,
    resources::{Res, ResMut, Resource},
};

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
    let mut world = ECSWorld::default();

    world.add_resource(WorldSize(0.0));
    world
        .create_entity()
        .with_component::<VelocityComponent>(VelocityComponent::default())
        .with_component::<PositionComponent>(PositionComponent::default());
    world
        .create_entity()
        .with_component::<VelocityComponent>(VelocityComponent::default());
    world
        .create_entity()
        .with_component::<VelocityComponent>(VelocityComponent::default());
    world
        .create_entity()
        .with_component::<VelocityComponent>(VelocityComponent::default());

    let query = Query::new(&world).with::<VelocityComponent>();
    for entity in query.iter() {
        let vel = world.get_component::<VelocityComponent>(entity).unwrap();
        print_velocity(vel);
    }
    // // dbg!(query);
    // dbg!(world.archetypes);

    // world.add_system(add_to_world_size);
    // world.add_system(print_world_size);
    //
    // world.run();
}

fn print_velocity(velocity_component: &VelocityComponent) {
    println!("{:?}", velocity_component.velocity);
}

fn add_to_world_size(mut world_size: ResMut<WorldSize>) {
    world_size.0 += 100.0;
}

fn print_world_size(world_size: Res<WorldSize>) {
    println!("world size: {}", world_size.0);
}

struct WorldSize(f32);
impl Resource for WorldSize {}
