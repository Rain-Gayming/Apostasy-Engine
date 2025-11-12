use std::any::Any;

use crate::app::engine::ecs::{component::Component, entities::Entity};

pub trait ComponentColumn: Component {
    fn as_any(&self) -> &dyn Component;
    fn as_any_mut(&mut self) -> &mut dyn Component;
    fn new_empty_column(&self) -> Box<dyn ComponentColumn>;
}

impl<T: 'static> ComponentColumn for Vec<T> {
    fn as_any(&self) -> &dyn Component {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Component {
        self
    }
    fn new_empty_column(&self) -> Box<dyn ComponentColumn> {
        Box::new(Vec::<T>::new())
    }
}

/// A data struct that holds entities
/// and only entities with the specified components in the `columns` data
pub struct Archetype {
    /// entities in this archetype
    pub entities: Vec<Entity>,
    /// the components the entities must have to be in this archetype
    pub columns: Vec<Box<dyn ComponentColumn>>,
}

impl Archetype {
    /// Takes an input archetype,
    /// if it doesn't have the column of type <T> then it will create a new archetype out of it
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

    /// Takes an input archetype,
    /// if it has the column of type <T> it will remove it and give a new archetype
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

    /// Returns an empty ColumnBuilder
    pub fn builder() -> ColumnsBuilder {
        ColumnsBuilder(Vec::new())
    }

    /// Creates a new archetype from the column builder specified
    pub fn new_from_columns(columns: ColumnsBuilder) -> Archetype {
        Archetype {
            entities: Vec::new(),
            columns: columns.0,
        }
    }
}

/// A struct that stores a vec of component columns,
/// used only to create a brand new archetype
pub struct ColumnsBuilder(Vec<Box<dyn ComponentColumn>>);

impl ColumnsBuilder {
    /// Takes in a type <T> and adds it to it's own ComponentColumns
    /// Returns itself
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
