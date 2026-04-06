pub mod custom_components;

use apostasy::engine::{
    nodes::world::World,
    start_app,
    windowing::cursor_manager::{CursorLockMode, CursorManager},
};
use apostasy_macros::start;

fn main() {
    start_app().unwrap();
    println!("Hello, world!");
}

#[start]
pub fn start(world: &mut World) {
    let cursor_manager = world.get_node_with_component_mut::<CursorManager>();
    if let Some(mut cursor_manager) = cursor_manager {
        cursor_manager
            .get_component_mut::<CursorManager>()
            .unwrap()
            .cursor_lock_mode = CursorLockMode::LockedHidden;
    }
}
