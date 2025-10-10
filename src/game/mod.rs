use crate::game::world::{new_world, World};

pub mod game_constants;
pub mod world;

pub struct Game {
    pub world: World,
}

pub fn initialize_game() -> Game {
    let world = new_world();

    Game { world }
}
