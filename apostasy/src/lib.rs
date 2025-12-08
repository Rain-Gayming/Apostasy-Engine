use apostasy_macros::Component;

pub mod engine;
pub mod utils;

pub mod prelude {
    use crate::engine::ecs::component::*;
    use crate::engine::ecs::entity::*;
    use crate::engine::ecs::query::*;
    use crate::engine::ecs::world::*;
}

#[derive(Component)]
pub struct Sigma {}
