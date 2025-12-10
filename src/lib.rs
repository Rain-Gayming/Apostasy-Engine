pub mod engine;
pub mod utils;

pub struct Sigma {}

pub mod prelude {
    use crate::engine::ecs::component::*;
    use crate::engine::ecs::entity::*;
    use crate::engine::ecs::query::*;
    use crate::engine::ecs::world::*;
}
