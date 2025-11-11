mod test {

    use crate::app::engine::ecs::ECSWorld;
    #[deny(dead_code)]
    use crate::app::engine::ecs::{archetype::*, component::Component, resources::Resource};

    struct TestResource(f32);
    impl Resource for TestResource {}

    struct WorldSize {
        pub width: f32,
        pub height: f32,
    }
    struct NewComponent(f32);
    impl Component for NewComponent {}

    #[test]
    #[should_panic]
    fn add_preexisting() {
        let archetype = Archetype {
            entities: Vec::new(),
            columns: Vec::new(),
        };
        let archetype = Archetype::new_from_add::<u32>(&archetype);
        let archetype = Archetype::new_from_add::<u32>(&archetype);
    }

    #[test]
    #[should_panic]
    fn remove_unpresent() {
        let archetype = Archetype {
            entities: Vec::new(),
            columns: Vec::new(),
        };
        let archetype = Archetype::new_from_remove::<u32>(&archetype);
    }

    #[test]
    #[should_panic]
    fn remove_unpresent_2() {
        let archetype = Archetype {
            entities: Vec::new(),
            columns: Vec::new(),
        };
        let archetype = Archetype::new_from_add::<u64>(&archetype);
        let archetype = Archetype::new_from_remove::<u32>(&archetype);
    }

    #[test]
    fn add_removes() {
        let archetype = Archetype {
            entities: Vec::new(),
            columns: Vec::new(),
        };

        let archetype = Archetype::new_from_add::<u32>(&archetype);
        assert!(archetype.columns.len() == 1);
        assert!(
            archetype
                .columns
                .iter()
                .find(|col| col.as_any().is::<Vec<u32>>())
                .is_some()
        );

        let archetype = Archetype::new_from_add::<u64>(&archetype);
        assert!(archetype.columns.len() == 2);
        assert!(
            archetype
                .columns
                .iter()
                .find(|col| col.as_any().is::<Vec<u32>>())
                .is_some()
        );
        assert!(
            archetype
                .columns
                .iter()
                .find(|col| col.as_any().is::<Vec<u64>>())
                .is_some()
        );

        let archetype = Archetype::new_from_remove::<u32>(&archetype);
        assert!(archetype.columns.len() == 1);
        assert!(
            archetype
                .columns
                .iter()
                .find(|col| col.as_any().is::<Vec<u64>>())
                .is_some()
        );
    }

    #[test]
    fn columns_builder() {
        let archetype = Archetype::new_from_columns(
            Archetype::builder()
                .with_column_type::<u32>()
                .with_column_type::<u64>()
                .with_column_type::<bool>(),
        );

        assert!(archetype.columns.len() == 3);
        assert!(
            archetype
                .columns
                .iter()
                .find(|col| col.as_any().is::<Vec<u32>>())
                .is_some()
        );
        assert!(
            archetype
                .columns
                .iter()
                .find(|col| col.as_any().is::<Vec<u64>>())
                .is_some()
        );
        assert!(
            archetype
                .columns
                .iter()
                .find(|col| col.as_any().is::<Vec<bool>>())
                .is_some()
        );
    }

    #[test]
    #[should_panic]
    fn columns_builder_duplicate() {
        let archetype = Archetype::new_from_columns(
            Archetype::builder()
                .with_column_type::<u32>()
                .with_column_type::<u32>(),
        );
    }
}
