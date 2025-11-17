use crate::app::engine::renderer::vertex::Vertex;

#[derive(Clone)]
pub struct Mesh<V: Vertex> {
    pub vertices: Vec<V>,
    pub indices: Vec<i32>,
}
