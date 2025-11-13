use std::any::TypeId;

use cgmath::{Vector3, Zero};

use crate::app::engine::ecs::component::Component;

pub struct VelocityComponent {
    pub velocity: Vector3<f32>,
}
impl Default for VelocityComponent {
    fn default() -> Self {
        VelocityComponent {
            velocity: Vector3::zero(),
        }
    }
}

impl Component for VelocityComponent {
    fn type_id_dyn(&self) -> TypeId {
        TypeId::of::<VelocityComponent>()
    }
}
impl PartialEq for VelocityComponent {
    fn eq(&self, other: &Self) -> bool {
        if self.type_id_dyn() != other.type_id_dyn() {
            return false;
        }
        self.type_id_dyn() == other.type_id_dyn()
    }
}
