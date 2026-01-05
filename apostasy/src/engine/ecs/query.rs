use core::index;

use apostasy_macros::Component;

use crate::engine::ecs::archetype::{Archetype, ArchetypeId, Signature};
use crate::engine::ecs::component::ComponentId;
use crate::engine::ecs::entity::EntityView;
use crate::engine::ecs::{World, entity::Entity};

#[derive(PartialEq, Eq)]
pub enum QueryAccess {
    Noop,
    Include,
    Exclude,
}
use crate as apostasy;
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
    pub world: World,
    pub components: Vec<QueryComponent>,
}

#[derive(Component)]
struct QueryState {
    // TODO:
}
trait QueryClosure {
    fn run(self, query: &Query, state: &QueryState);
}

impl<F: FnMut(EntityView<'_>)> QueryClosure for F {
    fn run(self, query: &Query, state: &QueryState) {}
}

impl Query {
    pub fn run<Closure: QueryClosure>(&self, func: Closure) {
        let cache = QueryState {};
        func.run(self, &cache);
    }
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
            panic!("Must add a component before trying to calling `include`");
        };
        query_component.access = QueryAccess::Include;
        query_component.id = component.raw();
        self
    }

    pub fn exclude(mut self, component: Entity) -> Self {
        let Some(query_component) = self.query.components.last_mut() else {
            panic!("Must add a component before trying to calling `exclude`");
        };
        query_component.access = QueryAccess::Exclude;
        query_component.id = component.raw();
        self
    }

    pub fn build(self) -> Query {
        // let mut archetypes_included: Vec<ArchetypeId> = Vec::new();
        // let mut archetypes_excluded: Vec<ArchetypeId> = Vec::new();
        //
        // self.query.world.crust.mantle(|mantle| {
        //     for signature in mantle.core.signature_index.iter() {
        //         for component in self.query.components.iter() {
        //             if signature.0.contains(ComponentId(component.id)) {
        //                 match component.access {
        //                     QueryAccess::Noop => (),
        //                     QueryAccess::Include => {
        //                         if archetypes_included.contains(signature.1) {
        //                             continue;
        //                         }
        //                         archetypes_included.push(signature.1.clone());
        //                     }
        //                     QueryAccess::Exclude => {
        //                         if archetypes_excluded.contains(signature.1) {
        //                             continue;
        //                         }
        //                         archetypes_excluded.push(signature.1.clone());
        //                     }
        //                 }
        //             }
        //         }
        //     }
        // });
        //
        // let mut to_remove = Vec::new();
        // for included in archetypes_included.clone() {
        //     if archetypes_excluded.contains(&included) {
        //         to_remove.push(included);
        //     }
        // }
        // for remove in to_remove {
        //     archetypes_included.retain(|&x| x != remove);
        // }
        //
        // self.query.world.crust.mantle(|mantle| {
        //     for archetype_id in archetypes_included {
        //         let archetype = mantle.core.archetypes[archetype_id];
        //     }
        // });

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
