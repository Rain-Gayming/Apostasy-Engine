// What is a bundle?
// - a collection of components that can be added at once
// - create an add_bundle function
//   - loop through all components
//   - add their values to the entity

use crate::app::engine::ecs::component::Component;

pub trait Bundle {
    fn get_bundle_components() -> Vec<Box<dyn Component>> {
        vec![]
    }
}
