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
    fn debug() {
        println!("i have 30000 fpses");
    }
}
