#![allow(unused)]
use std::any::{Any, TypeId};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use crate::app::engine::ecs::resource::Resource;

/// System Call Type
pub enum SystemCallType {
    Start,
    Update,
    FixedUpdate,
    Input,
    WindowChanged,
}

pub struct FunctionSystem<Input, F> {
    f: F,
    marker: PhantomData<fn() -> Input>,
}

/// Trait applied to every function that is converted into a system
pub trait System {
    fn run(&mut self, resources: &mut HashMap<TypeId, RefCell<Box<dyn Resource>>>);
}

/// Converts a function into a system
pub trait IntoSystem<Input> {
    type System: System;

    fn into_system(self) -> Self::System;
}

type StoredSystem = Box<dyn System>;

/// Stores systems and resources and allows them to run
#[derive(Default)]
pub struct Scheduler {
    pub start_systems: Vec<StoredSystem>,
    pub update_systems: Vec<StoredSystem>,
    pub fixed_update_systems: Vec<StoredSystem>,
    pub input_systems: Vec<StoredSystem>,
    pub window_changed_systems: Vec<StoredSystem>,
    pub resources: HashMap<TypeId, RefCell<Box<dyn Resource>>>,
}

impl Scheduler {
    /// Run each system
    pub fn run(&mut self, system_type: SystemCallType) {
        match system_type {
            SystemCallType::Start => {
                for system in self.start_systems.iter_mut() {
                    system.run(&mut self.resources);
                }
            }
            SystemCallType::Update => {
                for system in self.update_systems.iter_mut() {
                    system.run(&mut self.resources);
                }
            }
            SystemCallType::FixedUpdate => {
                for system in self.fixed_update_systems.iter_mut() {
                    system.run(&mut self.resources);
                }
            }
            SystemCallType::Input => {
                for system in self.input_systems.iter_mut() {
                    system.run(&mut self.resources);
                }
            }
            SystemCallType::WindowChanged => {
                for system in self.window_changed_systems.iter_mut() {
                    system.run(&mut self.resources);
                }
            }
        }
    }

    /// Add a new system
    pub fn add_system<I, S: System + 'static>(
        &mut self,
        system_type: SystemCallType,
        system: impl IntoSystem<I, System = S>,
    ) {
        match system_type {
            SystemCallType::Start => self.start_systems.push(Box::new(system.into_system())),
            SystemCallType::Update => self.update_systems.push(Box::new(system.into_system())),
            SystemCallType::FixedUpdate => self
                .fixed_update_systems
                .push(Box::new(system.into_system())),
            SystemCallType::Input => self.input_systems.push(Box::new(system.into_system())),
            SystemCallType::WindowChanged => self
                .window_changed_systems
                .push(Box::new(system.into_system())),
        }
    }

    /// Add a new resource
    pub fn add_resource<R: 'static + Resource>(&mut self, res: R) {
        self.resources
            .insert(TypeId::of::<R>(), RefCell::new(Box::new(res)));
    }
}

pub trait SystemParam {
    type Item<'new>;

    fn retrieve<'r>(resources: &'r HashMap<TypeId, RefCell<Box<dyn Resource>>>) -> Self::Item<'r>;
}

macro_rules! impl_system {
    ($($params:ident),*) => {
        #[allow(unused_variables)]
        #[allow(non_snake_case)]
        #[allow(clippy::too_many_arguments)]
        impl<F, $($params : SystemParam + 'static),*> System for FunctionSystem<($($params ,)*), F>
            where
                for<'a, 'b> &'a mut F:
                    FnMut($($params),*) +
                    FnMut($(<$params as SystemParam>::Item<'b>),*)
        {
            fn run(&mut self, resources: &mut HashMap<TypeId, RefCell<Box<dyn Resource>>>) {
                fn call_inner<$($params),*>(
                    mut f: impl FnMut($($params),*),
                    $(
                        $params: $params
                    ),*
                ) {
                    f($($params),*)
                }

                $(
                    let $params = $params::retrieve(resources);
                )*

                call_inner(&mut self.f, $($params),*)
            }
        }
    };
}

macro_rules! impl_into_system {
    (
        $($params:ident),*
    ) => {
        impl<F, $($params: SystemParam + 'static),*> IntoSystem<($($params,)*)> for F
            where
                for<'a, 'b> &'a mut F:
                    FnMut( $($params),* ) +
                    FnMut( $(<$params as SystemParam>::Item<'b>),* )
        {
            type System = FunctionSystem<($($params,)*), Self>;

            fn into_system(self) -> Self::System {
                FunctionSystem {
                    f: self,
                    marker: Default::default(),
                }
            }
        }
    }
}

macro_rules! call_n_times {
    ($target:ident, 1) => {
        $target!();
    };
    ($target:ident, 2) => {
        $target!(T1);
        call_n_times!($target, 1);
    };

    ($target:ident, 3) => {
        $target!(T1, T2);
        call_n_times!($target, 2);
    };

    ($target:ident, 4) => {
        $target!(T1, T2, T3);
        call_n_times!($target, 3);
    };

    ($target:ident, 5) => {
        $target!(T1, T2, T3, T4);
        call_n_times!($target, 4);
    };

    ($target:ident, 6) => {
        $target!(T1, T2, T3, T4, T5);
        call_n_times!($target, 5);
    };

    ($target:ident, 7) => {
        $target!(T1, T2, T3, T4, T5, T6);
        call_n_times!($target, 6);
    };

    ($target:ident, 8) => {
        $target!(T1, T2, T3, T4, T5, T6, T7);
        call_n_times!($target, 7);
    };

    ($target:ident, 9) => {
        $target!(T1, T2, T3, T4, T5, T6, T7, T8);
        call_n_times!($target, 8);
    };

    ($target:ident, 10) => {
        $target!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
        call_n_times!($target, 9);
    };
}

call_n_times!(impl_system, 10);
call_n_times!(impl_into_system, 10);
