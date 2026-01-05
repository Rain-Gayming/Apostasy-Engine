use apostasy::engine::ecs::{World, command::Command, component::Component, entity::EntityView};
use apostasy_macros::Component;

#[derive(Component)]
pub struct A(f32);
#[derive(Component)]
pub struct B();
#[derive(Component)]
pub struct C();
fn main() {
    let world = World::new();

    // spawn entity
    world.spawn().insert(A(0.0));
    world.spawn().insert(A(0.0)).insert(B());
    world.spawn().insert(A(0.0)).insert(B());
    world.spawn().insert(A(0.0)).insert(B());
    world.spawn().insert(A(0.0)).insert(B()).insert(C());
    world.spawn().insert(A(0.0)).insert(B()).insert(C());
    world.spawn().insert(A(0.0)).insert(B()).insert(C());

    world.flush();

    world
        .query()
        .with()
        .include::<A>()
        .build()
        .run(|view: EntityView<'_>| {
            println!("before: {}", view.get_mut::<A>().unwrap().0);
            view.get_mut::<A>().unwrap().0 += 1.0;
            println!("after: {}", view.get_mut::<A>().unwrap().0);
        });
}
