use std::{any::TypeId, fmt::Debug};

use crate::app::engine::ecs::{component::Component, entities::Entity};

pub trait ComponentColumn: Component {
    fn new_empty_column(&self) -> Box<dyn ComponentColumn>;
    fn eq_dyn(&self, other: &dyn ComponentColumn) -> bool;
}

impl<T: 'static + PartialEq> ComponentColumn for Vec<T> {
    fn new_empty_column(&self) -> Box<dyn ComponentColumn> {
        Box::new(Vec::<T>::new())
    }

    fn eq_dyn(&self, other: &dyn ComponentColumn) -> bool {
        // Try to downcast other to Vec<T>
        if let Some(other_vec) = other.as_any().downcast_ref::<Vec<T>>() {
            self == other_vec
        } else {
            false
        }
    }
}

impl Debug for dyn ComponentColumn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Point")
            .field("type", &self.type_id())
            .finish()
    }
}

impl PartialEq for dyn ComponentColumn {
    fn eq(&self, other: &Self) -> bool {
        self.eq_dyn(other)
    }
}

/// A data struct that holds entities
/// and only entities with the specified components in the `columns` data
#[derive(Debug)]
pub struct Archetype {
    /// entities in this archetype
    pub entities: Vec<Entity>,

    /// the type ids of the components in this struct
    pub component_types: Vec<TypeId>,

    /// the components the entities must have to be in this archetype
    pub columns: Vec<Box<dyn ComponentColumn>>,
}

impl Archetype {
    /// Takes an input archetype,
    /// if it doesn't have the column of type <T> then it will create a new archetype out of it
    pub fn new_from_add<T: 'static + PartialEq>(from_archetype: &Archetype) -> Archetype {
        let mut columns: Vec<_> = from_archetype
            .columns
            .iter()
            .map(|column| column.new_empty_column())
            .collect();

        let mut component_types: Vec<_> = from_archetype
            .columns
            .iter()
            .map(|column| column.type_id())
            .collect();

        columns.push(Box::new(Vec::<T>::new()));
        component_types.push(TypeId::of::<T>());

        Archetype {
            entities: Vec::new(),
            component_types,
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

        let mut component_types: Vec<_> = from_archetype
            .columns
            .iter()
            .map(|column| column.type_id())
            .collect();

        columns.remove(idx);

        Archetype {
            entities: Vec::new(),
            component_types,
            columns,
        }
    }

    /// Takes in a component type, loops through the current archetype,
    /// if it has the component, return true, otherwise return false
    pub fn contains_component<T: Component>(&self) -> bool {
        let columns: Vec<_> = self
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
    pub fn contains_columns(&self, columns: &Vec<Box<dyn ComponentColumn>>) -> bool {
        let self_columns: Vec<_> = self
            .columns
            .iter()
            .map(|column| column.new_empty_column())
            .collect();

        let other_columns: Vec<_> = columns
            .iter()
            .map(|column| column.new_empty_column())
            .collect();

        self_columns == other_columns
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

    let component_types: Vec<_> = columns.iter().map(|column| column.type_id()).collect();

    Archetype {
        entities: Vec::new(),
        component_types,
        columns,
    }
}

impl ColumnsBuilder {
    /// Takes in a type <T> and adds it to it's own ComponentColumns
    /// Returns itself
    pub fn with_column_type<T: 'static + PartialEq>(&mut self) -> &mut Self {
        self.0.push(Box::new(Vec::<T>::new()));
        self
    }

    /// Adds a new column to the builder
    pub fn add_column(&mut self, column: Box<dyn ComponentColumn>) -> &mut Self {
        self.0.push(column);
        self
    }
}
