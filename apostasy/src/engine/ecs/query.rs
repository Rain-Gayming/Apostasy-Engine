use crate::engine::ecs::{
    World,
    component::{self, ComponentId},
    entity::Entity,
};

pub enum QueryAccess {
    Noop,
    Include,
    Exclude,
}

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

pub struct Query {
    world: World,
    components: Vec<QueryComponent>,
}

pub struct QueryBuilder {
    query: Query,
    size: usize,
}

impl QueryBuilder {
    pub fn new(world: World) -> Self {
        QueryBuilder {
            query: Query {
                world,
                components: Vec::new(),
            },
            size: 0,
        }
    }

    pub fn with(mut self) -> Self {
        self.query.components.push(QueryComponent::default());
        self
    }

    pub fn include(mut self, component: Entity) -> Self {
        let Some(query_component) = self.query.components.last_mut() else {
            panic!("Must create a term before trying to calling `include`");
        };
        query_component.access = QueryAccess::Include;
        query_component.id = component.raw();
        self
    }

    pub fn exclude(mut self, component: Entity) -> Self {
        let Some(query_component) = self.query.components.last_mut() else {
            panic!("Must create a term before trying to calling `exclude`");
        };
        query_component.access = QueryAccess::Exclude;
        query_component.id = component.raw();
        self
    }

    pub fn build(self) -> Query {
        self.query
    }
}

// Queries take *in* a set of components
// and return any entities with those components
//
// eg:
// ```
//  fn foo(query: Query<Health, Stamina>){
//      for entity in query.results{
//          ...
//      }
//  }
//
// ```
