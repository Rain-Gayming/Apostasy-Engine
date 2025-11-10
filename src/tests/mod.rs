use std::any::TypeId;

use crate::app::engine::ecs::ECSWorld;
use crate::app::engine::ecs::component::Component;

#[test]
fn create_entity() {
    let mut world = ECSWorld::default();

    let new_entity = world
        .create_entity()
        .with_component::<NewComponent>(NewComponent(59.0));
    let new_component = new_entity.get_component_ref::<NewComponent>().unwrap();
    assert_eq!(new_component.0, 59.0);
}

#[test]
fn add_resource() {
    let mut world = ECSWorld::default();
    let world_size = WorldSize {
        width: 100.0,
        height: 100.0,
    };
    world.add_resource(world_size);

    let stored_resource = world.get_resource_ref::<WorldSize>().unwrap();
    assert_eq!(100.0, stored_resource.width);
}

struct NewComponent(f32);
impl Component for NewComponent {}

// #[test]
// fn get_resource() {
//     let resources = init_resource();
//
//     if let Some(extracted_resource) = resources.get_ref::<WorldSize>() {
//         assert_eq!(extracted_resource.width, 100.0);
//     }
// }
//
// #[test]
// fn get_resource_mut() {
//     let mut resources = init_resource();
//     {
//         let world_size: &mut WorldSize = resources.get_mut::<WorldSize>().unwrap();
//         world_size.height = 200.0;
//     }
//     let world_size = resources.get_ref::<WorldSize>().unwrap();
//     assert_eq!(world_size.height, 200.0);
// }
//
// #[test]
// fn remove_resource() {
//     let mut world = init_world();
//     world.remove_resource::<WorldSize>();
//     let deleted_resource = world.get_resource::<WorldSize>();
//     assert!(deleted_resource.is_none());
// }
//
// fn init_world() -> ECSWorld {
//     ECSWorld {
//         resources: init_resource(),
//     }
// }

struct WorldSize {
    pub width: f32,
    pub height: f32,
}
