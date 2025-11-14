use std::{any::TypeId, collections::HashMap, fmt::Debug};

use crate::app::engine::ecs::{component::Component, entities::Entity};

pub trait ComponentColumn: Component {
    fn new_empty_column(&self) -> Box<dyn ComponentColumn>;
    fn eq_dyn(&self, other: &dyn ComponentColumn) -> bool;
}

impl<T: 'static + PartialEq + Clone> ComponentColumn for Vec<T> {
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
    pub components: HashMap<TypeId, Vec<Box<dyn Component>>>,

    /// the components the entities must have to be in this archetype
    pub columns: Vec<Box<dyn ComponentColumn>>,
}

impl Archetype {
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
pub struct ColumnsBuilder(
    pub Vec<Box<dyn ComponentColumn>>,
    pub Vec<Box<dyn Component>>,
);

/// Returns an empty ColumnBuilder
pub fn new_column_builder() -> ColumnsBuilder {
    ColumnsBuilder(Vec::new(), Vec::new())
}

/// Creates a new archetype from the column builder specified
pub fn new_archetype_from_builder(columns_builder: &mut ColumnsBuilder) -> Archetype {
    // get the columns
    let columns: Vec<Box<dyn ComponentColumn>> = columns_builder
        .0
        .iter()
        .map(|column| column.new_empty_column())
        .collect();

    // get the component types
    let component_types: Vec<TypeId> = columns_builder
        .1
        .iter()
        .map(|comp| (*(*comp)).type_id())
        .collect();

    // get the components into a hashmap
    let mut components: HashMap<TypeId, Vec<Box<dyn Component>>> = HashMap::new();
    for component in columns_builder.1.iter() {
        components.insert((*(*component)).type_id(), vec![component.clone()]);
    }

    Archetype {
        entities: Vec::new(),
        component_types,
        components,
        columns,
    }
}

impl ColumnsBuilder {
    /// Takes in a type <T> and adds it to it's own ComponentColumns
    /// Returns itself
    pub fn with_column_type<T: 'static + PartialEq + Clone>(&mut self) -> &mut Self {
        self.0.push(Box::new(Vec::<T>::new()));
        self
    }

    /// Adds a new column to the builder
    pub fn add_column(&mut self, column: Box<dyn ComponentColumn>) -> &mut Self {
        self.0.push(column);
        self
    }
}
