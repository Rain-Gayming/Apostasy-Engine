use apostasy::engine::ecs::{World, command::Command};
use apostasy_macros::Component;

#[derive(Component)]
pub struct Health(f32);
#[derive(Component)]
pub struct Health2(f32);

fn main() {
    let world = World::new();

    // spawn entity
    let entity = world.spawn().insert::<Health>(Health(0.0));
    let entity2 = world
        .spawn()
        .insert::<Health>(Health(0.0))
        .insert::<Health2>(Health2(2.0));
    let entity3 = world.spawn();

    world.flush();

    println!("Spawned");

    world.crust.mantle(|mantle| mantle.archetypes());

    world.despawn(entity2.entity);
    world.flush();
    println!("Despawned");
    world.crust.mantle(|mantle| mantle.archetypes());
}
