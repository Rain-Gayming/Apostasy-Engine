use std::any::TypeId;

use cgmath::Vector3;
use cgmath::Zero;

use crate::app::engine::ecs::component::Component;

pub struct PositionComponent {
    pub position: Vector3<f32>,
}
impl Default for PositionComponent {
    fn default() -> Self {
        PositionComponent {
            position: Vector3::zero(),
        }
    }
}

impl Component for PositionComponent {
    fn type_id_dyn(&self) -> TypeId {
        TypeId::of::<PositionComponent>()
    }
}

impl PartialEq for PositionComponent {
    fn eq(&self, other: &Self) -> bool {
        if self.type_id_dyn() != other.type_id_dyn() {
            return false;
        }
        self.type_id_dyn() == other.type_id_dyn()
    }
}
