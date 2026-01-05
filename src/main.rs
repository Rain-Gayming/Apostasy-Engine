use apostasy::engine::ecs::{World, command::Command, component::Component};
use apostasy_macros::Component;

#[derive(Component)]
pub struct Health3(f32);
fn main() {
    let world = World::new();

    // spawn entity
    let entity3 = world.spawn().insert::<Health3>(Health3(2.0));

    world.flush();

    println!("Spawned");

    world.crust.mantle(|mantle| mantle.archetypes());

    entity3.remove(Health3::id());

    world.flush();
    println!("Despawned");
    world.crust.mantle(|mantle| mantle.archetypes());
}
