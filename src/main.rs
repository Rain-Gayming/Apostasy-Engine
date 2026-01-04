use apostasy::engine::ecs::World;
use apostasy_macros::Component;

#[derive(Component)]
pub struct Health();

#[derive(Component)]
pub struct Stamina();

#[derive(Component)]
pub struct Magicka();

fn main() {
    let world = World::new();

    // spawn entity
    // let entity = world.spawn();
    // let entity2 = world.spawn();
    // let entity3 = world.spawn();

    world.flush();

    dbg!(world.crust.mantle(|mantle| mantle.archetypes()));
}
