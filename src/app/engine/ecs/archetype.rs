use std::any::Any;

use crate::app::engine::ecs::{component::Component, entities::Entity};

pub trait ComponentColumn: Component {
    fn new_empty_column(&self) -> Box<dyn ComponentColumn>;
}

impl<T: 'static> ComponentColumn for Vec<T> {
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

    /// Takes in a component type, loops through the current archetype,
    /// if it has the component, return true, otherwise return false
    pub fn contains_component<T: Component>(&self) -> bool {
        let mut columns: Vec<_> = self
            .columns
            .iter()
            .map(|column| column.new_empty_column())
            .collect();

        columns
            .iter()
            .find(|column| column.as_any().is::<Vec<T>>())
            .is_some()
    }

    /// Takes in a component column, loops through the current archetype,
    /// if it has the column, return true, otherwise return false
    pub fn contains_columns(&self, columns: Vec<Box<dyn ComponentColumn>>) -> bool {
        let self_columns: Vec<_> = self
            .columns
            .iter()
            .map(|column| column.new_empty_column())
            .collect();

        let other_columns: Vec<_> = columns
            .iter()
            .map(|column| column.new_empty_column())
            .collect();
    }
}

/// A struct that stores a vec of component columns,
/// used only to create a brand new archetype
pub struct ColumnsBuilder(pub Vec<Box<dyn ComponentColumn>>);

/// Returns an empty ColumnBuilder
pub fn new_column_builder() -> ColumnsBuilder {
    ColumnsBuilder(Vec::new())
}

/// Creates a new archetype from the column builder specified
pub fn new_archetype_from_builder(columns: &mut ColumnsBuilder) -> Archetype {
    let columns: Vec<Box<dyn ComponentColumn>> = columns
        .0
        .iter()
        .map(|column| column.new_empty_column())
        .collect();

    Archetype {
        entities: Vec::new(),
        columns,
    }
}

impl ColumnsBuilder {
    /// Takes in a type <T> and adds it to it's own ComponentColumns
    /// Returns itself
    pub fn with_column_type<T: 'static>(&mut self) -> &mut Self {
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

    /// Adds a new column to the builder
    pub fn add_column(&mut self, column: Box<dyn ComponentColumn>) -> &mut Self {
        self.0.push(column);
        self
    }
}
