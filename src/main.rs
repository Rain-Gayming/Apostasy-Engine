use apostasy::engine::ecs::World;
use apostasy_macros::Component;

#[derive(Component)]
pub struct Health(f32);

fn main() {
    let mut world = World::new();

    // spawn entity
    let entity = world.spawn();

    dbg!(entity);
}
