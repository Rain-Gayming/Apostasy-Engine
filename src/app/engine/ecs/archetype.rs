use std::any::Any;

use crate::app::engine::ecs::entities::Entity;
pub trait ComponentColumn: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn new_empty_column(&self) -> Box<dyn ComponentColumn>;
}

impl<T: 'static> ComponentColumn for Vec<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn new_empty_column(&self) -> Box<dyn ComponentColumn> {
        Box::new(Vec::<T>::new())
    }
}

pub struct Archetype {
    pub entities: Vec<Entity>,
    pub columns: Vec<Box<dyn ComponentColumn>>,
}

impl Archetype {
    pub fn new_from_add<T: 'static>(from_archetype: &Archetype) -> Archetype {
        let mut columns: Vec<_> = from_archetype
            .columns
            .iter()
            .map(|column| column.new_empty_column())
            .collect();

        assert!(
            columns
                .iter()
                .find(|column| column.as_any().is::<Vec<T>>())
                .is_none()
        );
        columns.push(Box::new(Vec::<T>::new()));

        Archetype {
            entities: Vec::new(),
            columns,
        }
    }

    pub fn new_from_remove<T: 'static>(from_archetype: &Archetype) -> Archetype {
        let mut columns: Vec<_> = from_archetype
            .columns
            .iter()
            .map(|column| column.new_empty_column())
            .collect();

        let idx = columns
            .iter()
            .position(|column| column.as_any().is::<Vec<T>>())
            .unwrap();

        columns.remove(idx);

        Archetype {
            entities: Vec::new(),
            columns,
        }
    }
    pub fn builder() -> ColumnsBuilder {
        ColumnsBuilder(Vec::new())
    }

    pub fn new_from_columns(columns: ColumnsBuilder) -> Archetype {
        Archetype {
            entities: Vec::new(),
            columns: columns.0,
        }
    }
}

pub struct ColumnsBuilder(Vec<Box<dyn ComponentColumn>>);

impl ColumnsBuilder {
    pub fn with_column_type<T: 'static>(mut self) -> Self {
        if let Some(_) = self
            .0
            .iter()
            .find(|col| col.as_any().type_id() == std::any::TypeId::of::<Vec<T>>())
        {
            panic!("Attempted to create invalid archetype");
        }

        self.0.push(Box::new(Vec::<T>::new()));
        self
    }
}
