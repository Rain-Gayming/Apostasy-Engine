use apostasy::engine::ecs::{World, entity::EntityView};
use apostasy_macros::Component;

#[allow(dead_code)]
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

    world.flush();

    let query = world
        .query()
        .with()
        .include::<A>()
        .build()
        .run(|view: EntityView<'_>| {
            let a = view.get::<A>().unwrap().0;
            println!("{}", a);
        });
}
