use crate::app::engine::{
    ecs::component::Component,
    renderer::{mesh::Mesh, vertex::ModelVertex},
};
use component_derive::DeriveComponent;

#[derive(Clone, DeriveComponent)]
pub struct RenderableComponent {
    pub mesh: Mesh<ModelVertex>,
}
