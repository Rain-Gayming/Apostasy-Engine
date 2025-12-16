pub mod engine;
pub mod utils;

pub struct Sigma {}

pub mod prelude {
    use crate::engine::ecs::component::*;
    use crate::engine::ecs::entity::*;
    use crate::engine::ecs::query::*;
    use crate::engine::ecs::world::*;
}

#[cfg(test)]
mod tests {
    use crate::engine::ecs::world::World;

    #[test]
    fn entity_spawning() {
        let mut world = World::new();
        world.spawn();
        world
            .crust
            .mantle(|mantle| assert!(mantle.core.archetypes.slots.len() != 0));
    }
}
