pub mod engine;
pub mod utils;
pub use apostasy_macros::Component;

#[derive(Component)]
pub struct Sigma {}

pub mod prelude {
    use crate::engine::ecs::component::*;
    use crate::engine::ecs::entity::*;
    use crate::engine::ecs::query::*;
    use crate::engine::ecs::world::*;
}
