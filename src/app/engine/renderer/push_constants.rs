#[derive(Clone, Copy)]
pub struct PushConstants {
    pub view_matrix: [[f32; 4]; 4],
    pub projection_matrix: [[f32; 4]; 4],
    pub chunk_position: [i32; 3],
}
impl Default for PushConstants {
    fn default() -> Self {
        let view_matrix = [
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
        ];
        let projection_matrix = [
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
        ];
        let chunk_position = [0, 0, 0];

        PushConstants {
            view_matrix,
            projection_matrix,
            chunk_position,
        }
    }
}
