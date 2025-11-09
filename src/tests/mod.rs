use super::*;
use crate::app::engine::ecs::{ECSWorld, resources::Resource};
use std::any::TypeId;
//
// #[cfg(test)]
// fn create_and_get_resource_immutably() {
//     use crate::app::engine::ecs::ECSWorld;
//
//     let mut world = ECSWorld::default();
//
//     world.add_resource(FpsResource(60));
//     // if let Some(fps) = worldget_resource::<FpsResource>() {
//     //     assert_eq!(*fps, &60);
//     // }
// }
//
// struct FpsResource(pub u32);
//
// impl std::ops::Deref for FpsResource {
//     type Target = u32;
//
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }
//
// #[test]
// fn add_resource() {
//     let resources = init_resource();
//
//     let stored_resource = resources.data.get(&TypeId::of::<WorldSize>()).unwrap();
//     let extrated_resource = stored_resource.downcast_ref::<WorldSize>().unwrap();
//     assert_eq!(100.0, extrated_resource.width);
// }
//
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

// fn init_resource() -> Resource{
//     let mut resources = Resources::default();
//     let world_size = WorldSize {
//         width: 100.0,
//         height: 100.0,
//     };
//
//     resources.add(world_size);
//
//     resources
// }

struct WorldSize {
    pub width: f32,
    pub height: f32,
}
