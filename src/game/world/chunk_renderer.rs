use crate::{
    app::engine::renderer::{
        create_vertex_buffer_from_data,
        voxel_vertex::{VoxelVertex, CUBEMESH, CUBE_INDICES},
        Renderer,
    },
    game::{
        game_constants::{CHUNK_SIZE, CHUNK_SIZE_MINUS_ONE},
        world::chunk::Chunk,
    },
};

pub fn render_chunk(chunk: &Chunk, renderer: &mut Renderer, adjacent_chunks: [Option<Chunk>; 6]) {
    let mut vertex_data: Vec<VoxelVertex> = Vec::new();
    let mut index_data: Vec<u16> = Vec::new();

    let voxels = &chunk.voxels;

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                if !(z != 0
                    && voxels[x + y * CHUNK_SIZE + (z - 1) * CHUNK_SIZE * CHUNK_SIZE]
                        .voxel_type
                        .is_solid()
                    || z == 0
                        && adjacent_chunks[0].is_some()
                        && adjacent_chunks[0].as_ref().unwrap().voxels
                            [x + y * CHUNK_SIZE + CHUNK_SIZE_MINUS_ONE * CHUNK_SIZE * CHUNK_SIZE]
                            .voxel_type
                            .is_solid())
                {
                    generate_voxel_face(
                        &mut vertex_data,
                        &mut index_data,
                        0,
                        [x as u8, y as u8, z as u8],
                    );
                }
                if !(z != CHUNK_SIZE_MINUS_ONE
                    && voxels[x + y * CHUNK_SIZE + (z + 1) * CHUNK_SIZE * CHUNK_SIZE]
                        .voxel_type
                        .is_solid()
                    || z == CHUNK_SIZE_MINUS_ONE
                        && adjacent_chunks[1].is_some()
                        && adjacent_chunks[1].as_ref().unwrap().voxels
                            [x + y * CHUNK_SIZE + (CHUNK_SIZE * CHUNK_SIZE)]
                            .voxel_type
                            .is_solid())
                {
                    generate_voxel_face(
                        &mut vertex_data,
                        &mut index_data,
                        1,
                        [x as u8, y as u8, z as u8],
                    );
                }
                if !(x != CHUNK_SIZE_MINUS_ONE
                    && voxels[(x + 1) + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE]
                        .voxel_type
                        .is_solid()
                    || x == CHUNK_SIZE_MINUS_ONE
                        && adjacent_chunks[2].is_some()
                        && adjacent_chunks[2].as_ref().unwrap().voxels
                            [y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE]
                            .voxel_type
                            .is_solid())
                {
                    generate_voxel_face(
                        &mut vertex_data,
                        &mut index_data,
                        2,
                        [x as u8, y as u8, z as u8],
                    );
                }
                if !(x != 0
                    && voxels[(x - 1) + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE]
                        .voxel_type
                        .is_solid()
                    || x == 0
                        && adjacent_chunks[3].is_some()
                        && adjacent_chunks[3].as_ref().unwrap().voxels
                            [CHUNK_SIZE_MINUS_ONE + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE]
                            .voxel_type
                            .is_solid())
                {
                    generate_voxel_face(
                        &mut vertex_data,
                        &mut index_data,
                        3,
                        [x as u8, y as u8, z as u8],
                    );
                }
                if !(y != 0
                    && voxels[x + (y - 1) * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE]
                        .voxel_type
                        .is_solid()
                    || y == 0
                        && adjacent_chunks[4].is_some()
                        && adjacent_chunks[4].as_ref().unwrap().voxels
                            [x + CHUNK_SIZE_MINUS_ONE * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE]
                            .voxel_type
                            .is_solid())
                {
                    generate_voxel_face(
                        &mut vertex_data,
                        &mut index_data,
                        4,
                        [x as u8, y as u8, z as u8],
                    );
                }
                if !(y != CHUNK_SIZE_MINUS_ONE
                    && voxels[x + (y + 1) * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE]
                        .voxel_type
                        .is_solid()
                    || y == CHUNK_SIZE_MINUS_ONE
                        && adjacent_chunks[5].is_some()
                        && adjacent_chunks[5].as_ref().unwrap().voxels
                            [x + z * CHUNK_SIZE * CHUNK_SIZE]
                            .voxel_type
                            .is_solid())
                {
                    generate_voxel_face(
                        &mut vertex_data,
                        &mut index_data,
                        5,
                        [x as u8, y as u8, z as u8],
                    );
                }
            }
        }
    }

    if !vertex_data.is_empty() && !index_data.is_empty() {
        create_vertex_buffer_from_data(renderer, vertex_data, index_data, chunk.position);
    } else {
        println!("vertex and index buffers are empty");
    }
}

pub fn generate_voxel_face(
    vertex_data: &mut Vec<VoxelVertex>,
    index_data: &mut Vec<u16>,
    face: u8,
    position: [u8; 3],
) {
    for index in CUBE_INDICES.into_iter().take(6) {
        index_data.push(index + vertex_data.len() as u16);
    }
    for vertex in 0..4 {
        let base_position = CUBEMESH[face as usize * 4 + vertex];
        let position_x = base_position[0] + position[0];
        let position_y = base_position[1] + position[1];
        let position_z = base_position[2] + position[2];
        let position = [position_x, position_y, position_z];
        vertex_data.push(VoxelVertex { position });
    }
}
