use std::collections::HashMap;

use crate::objects::{
    Object,
    systems::{FixedUpdateSystem, LateUpdateSystem, StartSystem, UpdateSystem},
};

#[derive(Default)]
pub struct World {
    pub objects: HashMap<u64, Object>,
}

impl World {
    // ========== ========== Nodes ========== ==========
    /// Adds a new node to the world
    pub fn add_new_node(&mut self) -> &mut Self {
        let index = self.objects.len();
        self.objects.insert(index as u64, Object::default());

        self.assign_node_ids();

        self
    }

    pub(crate) fn assign_node_ids(&mut self) {
        let mut index = 0;

        for object in self.objects.iter_mut() {
            object.1.id = index;
            index += 1;
        }
    }

    pub fn debug_nodes(&self) {
        for object in self.objects.iter() {
            println!("{}: {}", object.1.name, object.1.id);
        }
    }

    // ========== ========== Systems ========== ==========

    /// Runs all start systems
    pub(crate) fn start(&mut self) {
        let mut systems = inventory::iter::<StartSystem>().collect::<Vec<_>>();
        systems.sort_by(|a, b| a.priority.cmp(&b.priority));
        systems.reverse();
        for system in systems.iter_mut() {
            (system.func)(self);
        }
    }

    /// Runs all update systems
    pub(crate) fn update(&mut self) {
        let mut systems = inventory::iter::<UpdateSystem>().collect::<Vec<_>>();
        systems.sort_by(|a, b| a.priority.cmp(&b.priority));
        systems.reverse();
        for system in systems.iter_mut() {
            (system.func)(self);
        }
    }

    /// Runs all fixed update systems
    pub(crate) fn fixed_update(&mut self, delta: f32) {
        let mut systems = inventory::iter::<FixedUpdateSystem>().collect::<Vec<_>>();
        systems.sort_by(|a, b| a.priority.cmp(&b.priority));
        systems.reverse();
        for system in systems.iter_mut() {
            (system.func)(self, delta);
        }
    }

    /// Runs all late update systems
    pub(crate) fn late_update(&mut self) {
        let mut systems = inventory::iter::<LateUpdateSystem>().collect::<Vec<_>>();
        systems.sort_by(|a, b| a.priority.cmp(&b.priority));
        systems.reverse();
        for system in systems.iter_mut() {
            (system.func)(self);
        }
    }
}
