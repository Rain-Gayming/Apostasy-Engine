use apostasy::engine::ecs::world::World;

#[derive(Component)]
pub struct Health(f32);

fn main() {
    let world = World::new();

    // spawn entity
    let entity = world.spawn();
}
