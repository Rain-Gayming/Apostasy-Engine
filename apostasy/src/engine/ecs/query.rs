use crate as apostasy;
use crate::engine::ecs::World;
use crate::engine::ecs::component::Component;
use crate::engine::ecs::entity::EntityView;
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

#[derive(Component)]
struct QueryState {}

#[allow(private_bounds)]
trait QueryClosure {
    fn run(self, query: &Query, state: &QueryState);
}

#[allow(unused_variables)]
impl<F: FnMut(EntityView<'_>)> QueryClosure for F {
    fn run(self, query: &Query, state: &QueryState) {
        query.world.crust.mantle(|mantle| {
            for entity_location in mantle.core.entity_index.lock().slots.iter() {
                let matches = query.components.iter().all(|qc| {
                    let data = entity_location.data.unwrap();
                    println!("looking for entity");
                    let entity_view = query
                        .world
                        .entity_from_location(entity_location.data.unwrap());

                    println!("looking for component");
                    let has_component = entity_view.get_id(qc.id).is_some();

                    match qc.access {
                        QueryAccess::Include => has_component,
                        QueryAccess::Exclude => !has_component,
                        QueryAccess::Noop => true,
                    }
                });
            }
        });
    }
}

impl Query {
    /// Runs the query
    #[allow(private_bounds)]
    pub fn run<Closure: QueryClosure>(&self, func: Closure) {
        let cache = QueryState {};
        func.run(self, &cache);
    }
}

impl Clone for Query {
    fn clone(&self) -> Self {
        Self {
            components: self.components.clone(),
            world: World {
                crust: self.world.crust.clone(),
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

    /// Adds a component to the query
    pub fn with(mut self) -> Self {
        self.query.components.push(QueryComponent::default());
        self
    }

    /// Includes a specific component from the query, use:
    /// ```rust
    ///     #[derive(Component)]
    ///     struct A();
    ///     fn foo(){
    ///         world
    ///             .query()
    ///             .with()
    ///             .include(A::id())
    ///     }
    /// ```
    pub fn include<C: Component>(mut self) -> Self {
        let Some(query_component) = self.query.components.last_mut() else {
            panic!("Must add a component before trying to calling `include`");
        };
        query_component.access = QueryAccess::Include;
        query_component.id = C::id().raw();
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
    ///             .with()
    ///             .exclude::<B>()
    ///     }
    /// ```
    pub fn exclude<C: Component>(mut self) -> Self {
        let Some(query_component) = self.query.components.last_mut() else {
            panic!("Must add a component before trying to calling `exclude`");
        };
        query_component.access = QueryAccess::Exclude;
        query_component.id = C::id().raw();
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
