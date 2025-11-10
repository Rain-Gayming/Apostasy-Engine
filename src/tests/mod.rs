mod test {

    #[deny(dead_code)]
    use crate::app::engine::ecs::{component::Component, resources::Resource, *};

    struct TestResource(f32);
    impl Resource for TestResource {}

    struct WorldSize {
        pub width: f32,
        pub height: f32,
    }
    struct NewComponent(f32);
    impl Component for NewComponent {}

    #[test]
    fn create_entity() {
        let mut world = ECSWorld::default();

        let new_entity = world
            .create_entity()
            .add_component::<NewComponent>(NewComponent(59.0));
        let new_component = new_entity.get_component_ref::<NewComponent>().unwrap();
        assert_eq!(new_component.0, 59.0);
    }

    #[test]
    fn add_system_test() {
        let mut world = ECSWorld::default();
        world.add_system(test_system());
        world.run_systems();
    }
    fn test_system() {
        println!("sigma");
    }

    #[test]
    fn get_component_mutalby() {
        let mut world = ECSWorld::default();

        let new_entity = world
            .create_entity()
            .add_component::<NewComponent>(NewComponent(59.0));
        let new_component = new_entity.get_component_mut::<NewComponent>().unwrap();

        new_component.0 += 10.0;

        assert_eq!(new_component.0, 69.0);
    }

    #[test]
    fn add_resource() {
        let mut world = ECSWorld::default();
        let test_resource = TestResource(32.0);
        world.add_resource(test_resource);
    }

    #[test]
    fn get_resource_mut() {
        let mut world = ECSWorld::default();
        let test_resource = TestResource(32.0);
        world.add_resource(test_resource);

        let get_resource = world.get_resource_mut::<TestResource>().unwrap();
        get_resource.0 += 32.0;

        assert_eq!(get_resource.0, 64.0);
    }

    #[test]
    fn get_resource_ref() {
        let mut world = ECSWorld::default();
        let test_resource = TestResource(32.0);
        world.add_resource(test_resource);

        let get_resource = world.get_resource_ref::<TestResource>().unwrap();
        assert_eq!(get_resource.0, 32.0);
    }

    #[test]
    fn remove_resource() {
        let mut world = ECSWorld::default();
        let test_resource = TestResource(32.0);
        world.add_resource(test_resource);

        world.remove_resource::<TestResource>();
        assert!(world.get_resource_ref::<TestResource>().is_none());
    }
}
