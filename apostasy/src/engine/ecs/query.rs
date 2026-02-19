use crate as apostasy;
use crate::engine::ecs::component::Component;
use crate::engine::ecs::entity::EntityView;
use crate::engine::ecs::{Mantle, World};
use apostasy_macros::Component;

/// What the query is to do with the specific component
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum QueryAccess {
    Noop,
    Include,
    Exclude,
}

/// A component added to a query
#[derive(Clone, Debug)]
pub struct QueryComponent {
    pub access: QueryAccess,
    pub id: u64,
}

impl Default for QueryComponent {
    fn default() -> Self {
        Self {
            id: 0,
            access: QueryAccess::Noop,
        }
    }
}

/// A query of components in the world
pub struct Query {
    pub world: World,
    pub components: Vec<QueryComponent>,
}

#[derive(Component, Default)]
struct QueryState {}

#[allow(private_bounds)]
trait QueryClosure {
    fn run(self, query: &Query, state: &QueryState);
}
#[allow(unused_variables)]
impl<F: FnMut(EntityView<'_>)> QueryClosure for F {
    fn run(mut self, query: &Query, state: &QueryState) {
        let entity_locations: Vec<_> = query.world.crust.mantle(|mantle| {
            mantle
                .core
                .entity_index
                .lock()
                .slots
                .iter()
                .filter_map(|slot| slot.data)
                .collect()
        });

        for entity_location in entity_locations {
            let entity_view = query.world.entity_from_location(entity_location);

            if entity_view.is_some() {
                query.world.crust.mantle(|mantle| {
                    let archetype = mantle
                        .core
                        .archetypes
                        .get(entity_location.archetype)
                        .unwrap();
                });
                // Check if entity matches the query
                let matches = query.components.iter().all(|qc| {
                    let has_component = entity_view.unwrap().get_id(qc.id).is_some();

                    match qc.access {
                        QueryAccess::Include => has_component,
                        QueryAccess::Exclude => !has_component,
                        QueryAccess::Noop => true,
                    }
                });

                if matches {
                    self(entity_view.unwrap());
                }
            }
        }
    }
}

impl Query {
    /// Runs the query
    #[allow(private_bounds)]
    pub fn run<F: FnMut(EntityView<'_>)>(&self, func: F) {
        let cache = QueryState {};
        func.run(self, &cache);
    }

    /// Runs a query and allows resources to be accessed, use:
    /// ```rust
    ///     #[derive(Resource)]
    ///     struct MyResource {
    ///         pub value: i32,
    ///     }
    ///
    ///     fn foo(){
    ///         world
    ///             .query()
    ///             .include::<Transform>()
    ///             .build()
    ///             .run_with_resources(|entity, mantle| {
    ///                 let resources = mantle.resources.read();
    ///                 if let Some(my_resource) = resources.get::<MyResource>() {
    ///                     println!("Time: {}, ", my_resource.value,);
    ///                 }
    ///             });
    ///     }
    /// ```
    pub fn run_with_resources<F: FnMut(EntityView<'_>, &Mantle)>(&self, mut func: F) {
        let entity_locations: Vec<_> = self.world.crust.mantle(|mantle| {
            mantle
                .core
                .entity_index
                .lock()
                .slots
                .iter()
                .filter_map(|slot| slot.data)
                .collect()
        });

        for entity_location in entity_locations {
            let entity_view = self.world.entity_from_location(entity_location);

            if entity_view.is_some() {
                let matches = self.components.iter().all(|qc| {
                    let has_component = entity_view.unwrap().get_id(qc.id).is_some();
                    match qc.access {
                        QueryAccess::Include => has_component,
                        QueryAccess::Exclude => !has_component,
                        QueryAccess::Noop => true,
                    }
                });

                if matches {
                    self.world.crust.mantle(|mantle| {
                        func(entity_view.unwrap(), mantle);
                    });
                }
            }
        }
    }
}

impl Clone for Query {
    fn clone(&self) -> Self {
        Self {
            components: self.components.clone(),
            world: World {
                crust: self.world.crust.clone(),
                rendering_context: self.world.rendering_context.clone(),
                packages: self.world.packages.clone(),
            },
        }
    }
}

/// Constructer for a query
#[allow(dead_code)]
pub struct QueryBuilder {
    query: Query,
    size: usize,
}

impl QueryBuilder {
    /// Creates a new QueryBuilder and Query,
    /// called through World
    pub fn new(world: World) -> Self {
        QueryBuilder {
            query: Query {
                world,
                components: Vec::new(),
            },
            size: 0,
        }
    }

    /// Includes a specific component from the query, use:
    /// ```rust
    ///     #[derive(Component)]
    ///     struct A();
    ///     fn foo(){
    ///         world
    ///             .query()
    ///             .include(A::id())
    ///     }
    /// ```
    pub fn include<C: Component>(mut self) -> Self {
        self.query.components.push(QueryComponent::default());
        let Some(query_component) = self.query.components.last_mut() else {
            panic!("Must add a component before trying to calling `include`");
        };
        query_component.access = QueryAccess::Include;
        query_component.id = C::id().val();
        self
    }

    /// Excludes a specific component from the query, use:
    /// ```rust
    ///
    ///     #[derive(Component)]
    ///     struct A();
    ///     #[derive(Component)]
    ///     struct B();
    ///     fn foo(){
    ///         world
    ///             .query()
    ///             .exclude::<B>()
    ///     }
    /// ```
    pub fn exclude<C: Component>(mut self) -> Self {
        self.query.components.push(QueryComponent::default());
        let Some(query_component) = self.query.components.last_mut() else {
            panic!("Must add a component before trying to calling `exclude`");
        };
        query_component.access = QueryAccess::Exclude;
        query_component.id = C::id().val();
        self
    }

    /// Builds the query, use:
    /// ```rust
    //
    ///     fn foo(){
    ///         world
    ///             .query()
    ///             .build()
    ///             .run(|view: EntityView<'_>| {
    ///                 ...
    ///              });
    ///     }
    /// ```
    pub fn build(self) -> Query {
        self.query
    }
}
