use apostasy::engine::{
    ecs::{World, entity::EntityView},
    rendering::{Application, start_renderer},
};
use apostasy_macros::Component;

#[allow(dead_code)]
#[derive(Component)]
pub struct A(f32);
#[derive(Component)]
pub struct B();

fn main() {
    let world = World::new();

    world.spawn().insert(A(0.0)).insert(B());

    start_renderer();

    world
        .query()
        .include::<A>()
        .include::<B>()
        .build()
        .run(|view: EntityView<'_>| {
            let a = view.get::<A>().unwrap().0 + 1.0;
            println!("{}", a);
        });
}
