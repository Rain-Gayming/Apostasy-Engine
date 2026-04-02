use ash::vk;
use cgmath::{InnerSpace, Vector2, Vector3};
use std::collections::HashSet;

use crate::engine::editor::{inspectable::Inspectable as InspectableTrait, EditorStorage};
use crate::engine::nodes::components::camera::Camera;
use crate::engine::nodes::components::transform::Transform;
use crate::engine::nodes::world::World;
use crate::{self as apostasy, log};
use apostasy_macros::{Component, Inspectable, InspectValue, SerializableComponent, update};
use egui::{Slider, Ui};
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, InspectValue, SerializableComponent, Serialize, Deserialize)]
pub struct Terrain {
    /// How many sub divisions per chunk
    pub subdivisions: u8,
    /// How far between each vertex on a chunk
    pub world_scale: f32,
    pub lod_levels: u8,

    #[serde(default)]
    pub chunks: Vec<TerrainChunk>,
    #[serde(skip, default)]
    pub is_dirty: bool,

    #[serde(skip, default)]
    pub gpu_chunks: Vec<TerrainChunkGpu>,

    #[serde(skip, default)]
    pub selected_chunk: u32,
    #[serde(skip, default)]
    pub selected_vertex_x: u32,
    #[serde(skip, default)]
    pub selected_vertex_z: u32,
}

impl Default for Terrain {
    fn default() -> Self {
        Self {
            subdivisions: 16,
            world_scale: 1.0,
            lod_levels: 4,
            chunks: Vec::new(),
            is_dirty: false,

            gpu_chunks: Vec::new(),
            selected_chunk: 0,
            selected_vertex_x: 0,
            selected_vertex_z: 0,
        }
    }
}

impl Terrain {
    pub fn apply_brush(
        &mut self,
        chunk_index: usize,
        x: u32,
        z: u32,
        radius: u32,
        delta: f32,
    ) {
        let mut boundary_vertices: HashSet<(usize, usize)> = HashSet::new();

        if let Some(chunk) = self.chunks.get_mut(chunk_index) {
            let subdivisions = self.subdivisions.max(1) as i32;
            let center_x = x as i32;
            let center_z = z as i32;
            let radius = radius as i32;

            for offset_z in -radius..=radius {
                for offset_x in -radius..=radius {
                    let sample_x = center_x + offset_x;
                    let sample_z = center_z + offset_z;
                    if sample_x < 0
                        || sample_z < 0
                        || sample_x >= subdivisions
                        || sample_z >= subdivisions
                    {
                        continue;
                    }
                    let heightmap_index = (sample_z as usize) * (subdivisions as usize)
                        + sample_x as usize;
                    if let Some(value) = chunk.heightmap.get_mut(heightmap_index) {
                        *value += delta;
                        if sample_x == 0
                            || sample_z == 0
                            || sample_x == subdivisions - 1
                            || sample_z == subdivisions - 1
                        {
                            boundary_vertices.insert((sample_x as usize, sample_z as usize));
                        }
                    }
                }
            }
            chunk.mesh_handle = None;
            chunk.gpu_dirty = true;
            chunk.dirty = true;
        }

        let affected_chunks = self.blend_chunk_edges(chunk_index, &boundary_vertices);
        let subdivisions = self.subdivisions;
        let world_scale = self.world_scale;

        for idx in affected_chunks {
            if idx >= self.chunks.len() {
                continue;
            }
            let mesh = build_terrain_mesh(self, subdivisions, world_scale, idx);
            let chunk = &mut self.chunks[idx];
            chunk.mesh_handle = Some(mesh);
            chunk.gpu_dirty = true;
            chunk.dirty = false;
        }
    }

    // Sample the average height in a 3x3 neighborhood around the selected vertex.
    // This will cross into adjacent chunks if the vertex is on a border.
    pub fn average_height_with_neighbors(
        &self,
        chunk_index: usize,
        vertex_x: u32,
        vertex_z: u32,
    ) -> f32 {
        let size = self.subdivisions.max(1) as u32;
        let mut total = 0.0;
        let mut sample_count = 0;

        for offset_z in -1..=1 {
            for offset_x in -1..=1 {
                let sample_x = vertex_x as i32 + offset_x;
                let sample_z = vertex_z as i32 + offset_z;
                // Collect sampled heights from the local chunk or neighboring chunks.
                total += sample_heightmap_with_neighbors(
                    self,
                    chunk_index,
                    size,
                    sample_x,
                    sample_z,
                );
                sample_count += 1;
            }
        }

        if sample_count == 0 {
            return 0.0;
        }

        total / sample_count as f32
    }

    // Smooth the selected chunk area by blending each vertex toward the local average.
    pub fn smooth_brush(
        &mut self,
        chunk_index: usize,
        vertex_x: u32,
        vertex_z: u32,
        brush_radius: u32,
        strength: f32,
    ) {
        if chunk_index >= self.chunks.len() {
            return;
        }

        // Clamp smoothing strength to a valid range and abort when it is zero.
        let strength = strength.clamp(0.0, 1.0);
        if strength <= 0.0 {
            return;
        }

        let subdivisions = self.subdivisions.max(1) as i32;
        let center_vertex_x = vertex_x as i32;
        let center_vertex_z = vertex_z as i32;
        let brush_radius_i32 = brush_radius as i32;

        // Preserve the original height values while we calculate the smoothed result.
        let old_heightmap = self.chunks[chunk_index].heightmap.clone();
        let mut new_heightmap = old_heightmap.clone();
        let mut boundary_vertices: HashSet<(usize, usize)> = HashSet::new();

        // Iterate over the brush area in chunk-local vertex coordinates.
        for offset_z in -brush_radius_i32..=brush_radius_i32 {
            for offset_x in -brush_radius_i32..=brush_radius_i32 {
                let sample_x = center_vertex_x + offset_x;
                let sample_z = center_vertex_z + offset_z;
                if sample_x < 0
                    || sample_z < 0
                    || sample_x >= subdivisions
                    || sample_z >= subdivisions
                {
                    continue;
                }

                let distance_squared = offset_x * offset_x + offset_z * offset_z;
                if distance_squared > brush_radius_i32 * brush_radius_i32 {
                    continue;
                }

                let heightmap_index = (sample_z as usize) * (subdivisions as usize)
                    + sample_x as usize;
                // Smooth this vertex by interpolating toward the average of its neighbors.
                let average = self.average_height_with_neighbors(
                    chunk_index,
                    sample_x as u32,
                    sample_z as u32,
                );
                new_heightmap[heightmap_index] =
                    old_heightmap[heightmap_index] + (average - old_heightmap[heightmap_index]) * strength;

                // Track vertices on the chunk boundary so adjacent chunks can be updated.
                if sample_x == 0
                    || sample_z == 0
                    || sample_x == subdivisions - 1
                    || sample_z == subdivisions - 1
                {
                    boundary_vertices.insert((sample_x as usize, sample_z as usize));
                }
            }
        }

        // Write the smoothed heightmap back into the chunk and invalidate the mesh.
        let chunk = &mut self.chunks[chunk_index];
        chunk.heightmap = new_heightmap;
        chunk.mesh_handle = None;
        chunk.gpu_dirty = true;
        chunk.dirty = true;

        if !boundary_vertices.is_empty() {
            // If smoothing touched a border, update neighboring edge heights too.
            let affected_chunks = self.blend_chunk_edges(chunk_index, &boundary_vertices);
            let subdivisions = self.subdivisions;
            let world_scale = self.world_scale;

            for idx in affected_chunks {
                if idx >= self.chunks.len() {
                    continue;
                }

                let mesh = build_terrain_mesh(self, subdivisions, world_scale, idx);
                let chunk = &mut self.chunks[idx];
                chunk.mesh_handle = Some(mesh);
                chunk.gpu_dirty = true;
                chunk.dirty = false;
            }
        } else {
            let mesh = build_terrain_mesh(self, self.subdivisions, self.world_scale, chunk_index);
            let chunk = &mut self.chunks[chunk_index];
            chunk.mesh_handle = Some(mesh);
            chunk.gpu_dirty = true;
            chunk.dirty = false;
        }
    }

    fn blend_chunk_edges(
        &mut self,
        chunk_index: usize,
        boundary_vertices: &HashSet<(usize, usize)>,
    ) -> Vec<usize> {
        let subdivisions = self.subdivisions.max(1) as usize;
        let mut affected_chunks: HashSet<usize> = HashSet::new();

        if chunk_index >= self.chunks.len() {
            return Vec::new();
        }

        affected_chunks.insert(chunk_index);
        let origin = self.chunks[chunk_index].origin;

        for &(x, z) in boundary_vertices {
            let current_idx = z * subdivisions + x;

            if x == 0 {
                let neighbor_origin = Vector2::new(origin.x - 1, origin.y);
                if let Some(neighbor_index) = self
                    .chunks
                    .iter()
                    .position(|chunk| chunk.origin == neighbor_origin)
                {
                    let neighbor_idx = z * subdivisions + (subdivisions - 1);
                    let avg = (self.chunks[chunk_index].heightmap[current_idx]
                        + self.chunks[neighbor_index].heightmap[neighbor_idx])
                        * 0.5;
                    self.chunks[chunk_index].heightmap[current_idx] = avg;
                    self.chunks[neighbor_index].heightmap[neighbor_idx] = avg;
                    affected_chunks.insert(neighbor_index);
                }
            }

            if x == subdivisions - 1 {
                let neighbor_origin = Vector2::new(origin.x + 1, origin.y);
                if let Some(neighbor_index) = self
                    .chunks
                    .iter()
                    .position(|chunk| chunk.origin == neighbor_origin)
                {
                    let neighbor_idx = z * subdivisions;
                    let avg = (self.chunks[chunk_index].heightmap[current_idx]
                        + self.chunks[neighbor_index].heightmap[neighbor_idx])
                        * 0.5;
                    self.chunks[chunk_index].heightmap[current_idx] = avg;
                    self.chunks[neighbor_index].heightmap[neighbor_idx] = avg;
                    affected_chunks.insert(neighbor_index);
                }
            }

            if z == 0 {
                let neighbor_origin = Vector2::new(origin.x, origin.y - 1);
                if let Some(neighbor_index) = self
                    .chunks
                    .iter()
                    .position(|chunk| chunk.origin == neighbor_origin)
                {
                    let neighbor_idx = (subdivisions - 1) * subdivisions + x;
                    let avg = (self.chunks[chunk_index].heightmap[current_idx]
                        + self.chunks[neighbor_index].heightmap[neighbor_idx])
                        * 0.5;
                    self.chunks[chunk_index].heightmap[current_idx] = avg;
                    self.chunks[neighbor_index].heightmap[neighbor_idx] = avg;
                    affected_chunks.insert(neighbor_index);
                }
            }

            if z == subdivisions - 1 {
                let neighbor_origin = Vector2::new(origin.x, origin.y + 1);
                if let Some(neighbor_index) = self
                    .chunks
                    .iter()
                    .position(|chunk| chunk.origin == neighbor_origin)
                {
                    let neighbor_idx = x;
                    let avg = (self.chunks[chunk_index].heightmap[current_idx]
                        + self.chunks[neighbor_index].heightmap[neighbor_idx])
                        * 0.5;
                    self.chunks[chunk_index].heightmap[current_idx] = avg;
                    self.chunks[neighbor_index].heightmap[neighbor_idx] = avg;
                    affected_chunks.insert(neighbor_index);
                }
            }

            if x == 0 && z == 0 {
                let left_origin = Vector2::new(origin.x - 1, origin.y);
                let top_origin = Vector2::new(origin.x, origin.y - 1);
                let diag_origin = Vector2::new(origin.x - 1, origin.y - 1);
                let left_index = self.chunks.iter().position(|chunk| chunk.origin == left_origin);
                let top_index = self.chunks.iter().position(|chunk| chunk.origin == top_origin);
                let diag_index = self.chunks.iter().position(|chunk| chunk.origin == diag_origin);

                if let (Some(left_index), Some(top_index), Some(diag_index)) =
                    (left_index, top_index, diag_index)
                {
                    let left_idx = 0 * subdivisions + (subdivisions - 1);
                    let top_idx = (subdivisions - 1) * subdivisions + 0;
                    let diag_idx = (subdivisions - 1) * subdivisions + (subdivisions - 1);
                    let total = self.chunks[chunk_index].heightmap[current_idx]
                        + self.chunks[left_index].heightmap[left_idx]
                        + self.chunks[top_index].heightmap[top_idx]
                        + self.chunks[diag_index].heightmap[diag_idx];
                    let avg = total / 4.0;
                    self.chunks[chunk_index].heightmap[current_idx] = avg;
                    {
                        let chunk = &mut self.chunks[left_index];
                        chunk.heightmap[left_idx] = avg;
                    }
                    {
                        let chunk = &mut self.chunks[top_index];
                        chunk.heightmap[top_idx] = avg;
                    }
                    {
                        let chunk = &mut self.chunks[diag_index];
                        chunk.heightmap[diag_idx] = avg;
                    }
                    affected_chunks.insert(left_index);
                    affected_chunks.insert(top_index);
                    affected_chunks.insert(diag_index);
                }
            }

            if x == subdivisions - 1 && z == 0 {
                let right_origin = Vector2::new(origin.x + 1, origin.y);
                let top_origin = Vector2::new(origin.x, origin.y - 1);
                let diag_origin = Vector2::new(origin.x + 1, origin.y - 1);
                let right_index = self.chunks.iter().position(|chunk| chunk.origin == right_origin);
                let top_index = self.chunks.iter().position(|chunk| chunk.origin == top_origin);
                let diag_index = self.chunks.iter().position(|chunk| chunk.origin == diag_origin);

                if let (Some(right_index), Some(top_index), Some(diag_index)) =
                    (right_index, top_index, diag_index)
                {
                    let right_idx = 0 * subdivisions;
                    let top_idx = (subdivisions - 1) * subdivisions + (subdivisions - 1);
                    let diag_idx = (subdivisions - 1) * subdivisions;
                    let total = self.chunks[chunk_index].heightmap[current_idx]
                        + self.chunks[right_index].heightmap[right_idx]
                        + self.chunks[top_index].heightmap[top_idx]
                        + self.chunks[diag_index].heightmap[diag_idx];
                    let avg = total / 4.0;
                    self.chunks[chunk_index].heightmap[current_idx] = avg;
                    {
                        let chunk = &mut self.chunks[right_index];
                        chunk.heightmap[right_idx] = avg;
                    }
                    {
                        let chunk = &mut self.chunks[top_index];
                        chunk.heightmap[top_idx] = avg;
                    }
                    {
                        let chunk = &mut self.chunks[diag_index];
                        chunk.heightmap[diag_idx] = avg;
                    }
                    affected_chunks.insert(right_index);
                    affected_chunks.insert(top_index);
                    affected_chunks.insert(diag_index);
                }
            }

            if x == 0 && z == subdivisions - 1 {
                let left_origin = Vector2::new(origin.x - 1, origin.y);
                let bottom_origin = Vector2::new(origin.x, origin.y + 1);
                let diag_origin = Vector2::new(origin.x - 1, origin.y + 1);
                let left_index = self.chunks.iter().position(|chunk| chunk.origin == left_origin);
                let bottom_index = self.chunks.iter().position(|chunk| chunk.origin == bottom_origin);
                let diag_index = self.chunks.iter().position(|chunk| chunk.origin == diag_origin);

                if let (Some(left_index), Some(bottom_index), Some(diag_index)) =
                    (left_index, bottom_index, diag_index)
                {
                    let left_idx = (subdivisions - 1) * subdivisions + (subdivisions - 1);
                    let bottom_idx = 0;
                    let diag_idx = subdivisions - 1;
                    let total = self.chunks[chunk_index].heightmap[current_idx]
                        + self.chunks[left_index].heightmap[left_idx]
                        + self.chunks[bottom_index].heightmap[bottom_idx]
                        + self.chunks[diag_index].heightmap[diag_idx];
                    let avg = total / 4.0;
                    self.chunks[chunk_index].heightmap[current_idx] = avg;
                    {
                        let chunk = &mut self.chunks[left_index];
                        chunk.heightmap[left_idx] = avg;
                    }
                    {
                        let chunk = &mut self.chunks[bottom_index];
                        chunk.heightmap[bottom_idx] = avg;
                    }
                    {
                        let chunk = &mut self.chunks[diag_index];
                        chunk.heightmap[diag_idx] = avg;
                    }
                    affected_chunks.insert(left_index);
                    affected_chunks.insert(bottom_index);
                    affected_chunks.insert(diag_index);
                }
            }

            if x == subdivisions - 1 && z == subdivisions - 1 {
                let right_origin = Vector2::new(origin.x + 1, origin.y);
                let bottom_origin = Vector2::new(origin.x, origin.y + 1);
                let diag_origin = Vector2::new(origin.x + 1, origin.y + 1);
                let right_index = self.chunks.iter().position(|chunk| chunk.origin == right_origin);
                let bottom_index = self.chunks.iter().position(|chunk| chunk.origin == bottom_origin);
                let diag_index = self.chunks.iter().position(|chunk| chunk.origin == diag_origin);

                if let (Some(right_index), Some(bottom_index), Some(diag_index)) =
                    (right_index, bottom_index, diag_index)
                {
                    let right_idx = (subdivisions - 1) * subdivisions;
                    let bottom_idx = subdivisions - 1;
                    let diag_idx = 0;
                    let total = self.chunks[chunk_index].heightmap[current_idx]
                        + self.chunks[right_index].heightmap[right_idx]
                        + self.chunks[bottom_index].heightmap[bottom_idx]
                        + self.chunks[diag_index].heightmap[diag_idx];
                    let avg = total / 4.0;
                    self.chunks[chunk_index].heightmap[current_idx] = avg;
                    {
                        let chunk = &mut self.chunks[right_index];
                        chunk.heightmap[right_idx] = avg;
                    }
                    {
                        let chunk = &mut self.chunks[bottom_index];
                        chunk.heightmap[bottom_idx] = avg;
                    }
                    {
                        let chunk = &mut self.chunks[diag_index];
                        chunk.heightmap[diag_idx] = avg;
                    }
                    affected_chunks.insert(right_index);
                    affected_chunks.insert(bottom_index);
                    affected_chunks.insert(diag_index);
                }
            }
        }

        for idx in affected_chunks.iter() {
            if let Some(chunk) = self.chunks.get_mut(*idx) {
                chunk.mesh_handle = None;
                chunk.gpu_dirty = true;
                chunk.dirty = true;
            }
        }

        affected_chunks.into_iter().collect()
    }

    pub fn world_point_to_vertex(
        &self,
        transform: &Transform,
        world_point: Vector3<f32>
    ) -> Option<(usize, u32, u32)> {
        let subdivisions = self.subdivisions.max(1) as f32;
        let chunk_world_size = subdivisions * self.world_scale;
        let local_point = world_point - transform.global_position;

        let chunk_x = (local_point.x / chunk_world_size).floor() as i32;
        let chunk_z = (local_point.z / chunk_world_size).floor() as i32;

        let chunk_index = self.chunks.iter().position(|chunk| {
            chunk.origin.x == chunk_x && chunk.origin.y == chunk_z
        })?;

        let local_x = (local_point.x - chunk_x as f32 * chunk_world_size) / self.world_scale;
        let local_z = (local_point.z - chunk_z as f32 * chunk_world_size) / self.world_scale;

        let vertex_x = local_x.round().clamp(0.0, subdivisions - 1.0) as u32;
        let vertex_z = local_z.round().clamp(0.0, subdivisions - 1.0) as u32;

        Some((chunk_index, vertex_x, vertex_z))
    }

    pub fn chunk_index_for_origin(&self, origin: Vector2<i32>) -> Option<usize> {
        self.chunks.iter().position(|chunk| chunk.origin == origin)
    }

    pub fn add_chunk(&mut self, origin: Vector2<i32>) -> usize {
        if let Some(index) = self.chunk_index_for_origin(origin) {
            return index;
        }

        let subdivisions = self.subdivisions.max(1) as usize;
        let heightmap = vec![0.0; subdivisions * subdivisions];

        let index = self.chunks.len();
        self.chunks.push(TerrainChunk {
            origin,
            lod: 0,
            mesh_handle: None,
            heightmap,
            gpu_dirty: true,
            dirty: true,
        });
        self.selected_chunk = index as u32;
        self.selected_vertex_x = 0;
        self.selected_vertex_z = 0;
        index
    }

    // Remove a terrain chunk and keep the GPU chunk array aligned.
    pub fn delete_chunk(&mut self, index: usize) {
        if index >= self.chunks.len() {
            return;
        }
        // Remove the CPU-side chunk entry.

        self.chunks.remove(index);
        if index < self.gpu_chunks.len() {
            // Also remove the corresponding GPU chunk slot so indices stay aligned.
            self.gpu_chunks.remove(index);
        }

        let max_chunk = self.chunks.len().saturating_sub(1) as u32;
        self.selected_chunk = self.selected_chunk.min(max_chunk);
        self.selected_vertex_x = 0;
        self.selected_vertex_z = 0;
    }

    pub fn add_adjacent_chunks_from_vertex(
        &mut self,
        chunk_index: usize,
        x: u32,
        z: u32,
    ) -> Vec<usize> {
        if chunk_index >= self.chunks.len() {
            return Vec::new();
        }

        let origin = self.chunks[chunk_index].origin;
        let subdivisions = self.subdivisions.max(1) as u32;
        let mut added = Vec::new();

        if x == 0 {
            added.push(self.add_chunk(Vector2::new(origin.x - 1, origin.y)));
        }
        if x == subdivisions - 1 {
            added.push(self.add_chunk(Vector2::new(origin.x + 1, origin.y)));
        }
        if z == 0 {
            added.push(self.add_chunk(Vector2::new(origin.x, origin.y - 1)));
        }
        if z == subdivisions - 1 {
            added.push(self.add_chunk(Vector2::new(origin.x, origin.y + 1)));
        }

        added.sort_unstable();
        added.dedup();
        added
    }

    pub fn world_point_to_chunk_origin(
        &self,
        transform: &Transform,
        world_point: Vector3<f32>,
    ) -> (Vector2<i32>, u32, u32) {
        let subdivisions = self.subdivisions.max(1) as f32;
        let chunk_world_size = subdivisions * self.world_scale;
        let local_point = world_point - transform.global_position;

        let chunk_x = (local_point.x / chunk_world_size).floor() as i32;
        let chunk_z = (local_point.z / chunk_world_size).floor() as i32;

        let local_x = (local_point.x - chunk_x as f32 * chunk_world_size) / self.world_scale;
        let local_z = (local_point.z - chunk_z as f32 * chunk_world_size) / self.world_scale;

        let vertex_x = local_x.round().clamp(0.0, subdivisions - 1.0) as u32;
        let vertex_z = local_z.round().clamp(0.0, subdivisions - 1.0) as u32;

        (Vector2::new(chunk_x, chunk_z), vertex_x, vertex_z)
    }
}

impl InspectableTrait for Terrain {
    fn inspect(&mut self, ui: &mut Ui, _editor_storage: &mut EditorStorage) -> bool {
        let mut remove = false;

        ui.horizontal(|ui| {
            if ui.small_button("✕").clicked() {
                remove = true;
            }
            ui.label("Terrain");
        });

        egui::CollapsingHeader::new("Terrain")
            .default_open(true)
            .show(ui, |ui| {
                let mut changed = false;

                ui.horizontal(|ui| {
                    ui.label("Subdivisions:");
                    let mut subdivisions = self.subdivisions as u32;
                    if ui
                        .add(egui::DragValue::new(&mut subdivisions).range(1..=64))
                        .changed()
                    {
                        self.subdivisions = subdivisions as u8;
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("World scale:");
                    if ui
                        .add(egui::DragValue::new(&mut self.world_scale).speed(0.1))
                        .changed()
                    {
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("LOD levels:");
                    let mut lod_levels = self.lod_levels as u32;
                    if ui
                        .add(egui::DragValue::new(&mut lod_levels).range(1..=8))
                        .changed()
                    {
                        self.lod_levels = lod_levels as u8;
                        changed = true;
                    }
                });

                if ui.button("Regenerate terrain").clicked() {
                    self.is_dirty = true;
                }

                        if changed {
                    self.is_dirty = true;
                }

                ui.separator();

                if self.chunks.is_empty() {
                    ui.label("No terrain chunks available.");
                    return;
                }

                let chunk_count = self.chunks.len() as u32;
                let max_chunk = chunk_count.saturating_sub(1);
                self.selected_chunk = self.selected_chunk.min(max_chunk);

                ui.horizontal(|ui| {
                    ui.label("Chunk:");
                    ui.add(Slider::new(&mut self.selected_chunk, 0..=max_chunk));
                    ui.label(format!("{}/{}", self.selected_chunk + 1, chunk_count));
                });

                let subdivisions = self.subdivisions.max(1) as u32;
                let max_index = subdivisions.saturating_sub(1);
                self.selected_vertex_x = self.selected_vertex_x.min(max_index);
                self.selected_vertex_z = self.selected_vertex_z.min(max_index);

            });
        remove
    }
}

#[derive(Clone, InspectValue, Inspectable, Serialize, Deserialize)]
pub struct TerrainChunk {
    pub origin: Vector2<i32>,
    pub lod: u8,
    #[serde(skip, default)]
    #[inspect(skip)]
    pub mesh_handle: Option<TerrainMesh>,
    #[serde(default)]
    pub heightmap: Vec<f32>,
    #[serde(skip, default = "default_true")]
    #[inspect(skip)]
    pub gpu_dirty: bool,
    #[serde(skip, default)]
    #[inspect(skip)]
    pub dirty: bool,
}

fn default_true() -> bool {
    true
}

#[update]
pub fn terrain_update_system(world: &mut World) {
    let camera_pos = world
        .get_global_node_with_component::<Camera>()
        .and_then(|n| n.get_component::<Transform>())
        .map(|t| t.global_position)
        .unwrap();

    let terrain_ids: Vec<u64> = world
        .get_all_world_nodes()
        .iter()
        .filter(|n| n.has_component::<Terrain>())
        .map(|n| n.id)
        .collect();

    for id in terrain_ids {
        let node = world.get_node_mut(id);
        if let Some(_) = node.get_component::<Transform>() {
            let (terrain, transform) = node.get_components_mut::<(&mut Terrain, &mut Transform)>();

            if terrain.chunks.is_empty() {
                terrain.is_dirty = true;
            }

            if terrain.is_dirty {
                regenerate_chunks(terrain);
                log!("Regenerating terrain");
                terrain.is_dirty = false;
            }

            update_lod(terrain, transform, camera_pos);

            let subdivisions = terrain.subdivisions;
            let world_scale = terrain.world_scale;
            let required_heightmap_len = (subdivisions.max(1) as usize).pow(2);

            for chunk_index in 0..terrain.chunks.len() {
                {
                    let chunk = &mut terrain.chunks[chunk_index];
                    if chunk.heightmap.len() != required_heightmap_len {
                        chunk.heightmap.resize(required_heightmap_len, 0.0);
                        chunk.mesh_handle = None;
                        chunk.gpu_dirty = true;
                        chunk.dirty = true;
                    }
                    if chunk.dirty {
                        chunk.mesh_handle = None;
                        chunk.gpu_dirty = true;
                        chunk.dirty = false;
                    }
                }

                if terrain.chunks[chunk_index].mesh_handle.is_none() {
                    let mesh = build_terrain_mesh(&*terrain, subdivisions, world_scale, chunk_index);
                    terrain.chunks[chunk_index].mesh_handle = Some(mesh);
                }
            }
        }
    }
}

fn regenerate_chunks(terrain: &mut Terrain) {
    let subdivisions = terrain.subdivisions.max(1) as usize;
    let new_heightmap = vec![0.0; subdivisions * subdivisions];

    if terrain.chunks.is_empty() {
        let count = 4i32;
        for cz in 0..count {
            for cx in 0..count {
                terrain.chunks.push(TerrainChunk {
                    origin: Vector2::new(cx, cz),
                    lod: 0,
                    mesh_handle: None,
                    heightmap: new_heightmap.clone(),
                    gpu_dirty: true,
                    dirty: true,
                });
            }
        }
        return;
    }

    for chunk in terrain.chunks.iter_mut() {
        chunk.heightmap.clear();
        chunk.heightmap.resize(subdivisions * subdivisions, 0.0);
        chunk.mesh_handle = None;
        chunk.gpu_dirty = true;
        chunk.dirty = true;
    }
}

fn update_lod(terrain: &mut Terrain, transform: &Transform, camera_pos: cgmath::Vector3<f32>) {
    let chunk_world_size = terrain.subdivisions as f32 * terrain.world_scale;

    for chunk in terrain.chunks.iter_mut() {
        let chunk_world_x = chunk.origin.x as f32 * chunk_world_size + transform.global_position.x;
        let chunk_world_z = chunk.origin.y as f32 * chunk_world_size + transform.global_position.z;

        let dist = cgmath::Vector2::new(camera_pos.x - chunk_world_x, camera_pos.z - chunk_world_z)
            .magnitude();

        let new_lod = match dist {
            d if d < 64.0 => 0,
            d if d < 128.0 => 1,
            d if d < 256.0 => 2,
            _ => 3,
        };

        if new_lod != chunk.lod {
            chunk.lod = new_lod;
            chunk.mesh_handle = None;
        }
    }
}

#[derive(Clone, Default, InspectValue, Inspectable, Serialize, Deserialize)]
pub struct TerrainMesh {
    pub vertices: Vec<TerrainVertex>,
    pub indices: Vec<u32>,
}

#[repr(C)]
#[derive(Clone, Default, InspectValue, Inspectable, Serialize, Deserialize)]
pub struct TerrainVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

#[derive(Clone)]
pub struct TerrainChunkGpu {
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,
    pub index_count: u32,
}

impl Default for TerrainChunkGpu {
    fn default() -> Self {
        Self {
            vertex_buffer: vk::Buffer::null(),
            vertex_buffer_memory: vk::DeviceMemory::null(),
            index_buffer: vk::Buffer::null(),
            index_buffer_memory: vk::DeviceMemory::null(),
            index_count: 0,
        }
    }
}

fn sample_heightmap(height_map: &[f32], size: u32, x: u32, z: u32) -> f32 {
    let index = (z * size + x) as usize;
    height_map.get(index).copied().unwrap_or(0.0)
}

fn sample_heightmap_with_neighbors(
    terrain: &Terrain,
    chunk_index: usize,
    size: u32,
    x: i32,
    z: i32,
) -> f32 {
    if x >= 0 && x < size as i32 && z >= 0 && z < size as i32 {
        return sample_heightmap(
            &terrain.chunks[chunk_index].heightmap,
            size,
            x as u32,
            z as u32,
        );
    }

    let current_chunk = &terrain.chunks[chunk_index];
    let mut neighbor_origin = current_chunk.origin;
    let mut sample_x = x;
    let mut sample_z = z;

    if x < 0 {
        neighbor_origin.x -= 1;
        sample_x = size as i32 - 1;
    } else if x >= size as i32 {
        neighbor_origin.x += 1;
        sample_x = 0;
    }

    if z < 0 {
        neighbor_origin.y -= 1;
        sample_z = size as i32 - 1;
    } else if z >= size as i32 {
        neighbor_origin.y += 1;
        sample_z = 0;
    }

    if let Some(neighbor_index) = terrain
        .chunks
        .iter()
        .position(|chunk| chunk.origin == neighbor_origin)
    {
        return sample_heightmap(
            &terrain.chunks[neighbor_index].heightmap,
            size,
            sample_x as u32,
            sample_z as u32,
        );
    }

    let clamped_x = x.clamp(0, size as i32 - 1) as u32;
    let clamped_z = z.clamp(0, size as i32 - 1) as u32;
    sample_heightmap(&current_chunk.heightmap, size, clamped_x, clamped_z)
}

/// How many heightmap cells to skip per vertex in this level of detail
fn lod_step(lod: u8) -> u32 {
    1u32 << lod
}

fn calculate_normal(
    terrain: &Terrain,
    chunk_index: usize,
    size: u32,
    vertex_x: u32,
    vertex_z: u32,
    world_scale: f32,
    step: u32,
) -> [f32; 3] {
    let vertex_x = vertex_x as i32;
    let vertex_z = vertex_z as i32;
    let step_offset = step as i32;

    // Sample the surrounding heights at the LOD step distance.
    let height_left = sample_heightmap_with_neighbors(
        terrain,
        chunk_index,
        size,
        vertex_x - step_offset,
        vertex_z,
    );
    let height_right = sample_heightmap_with_neighbors(
        terrain,
        chunk_index,
        size,
        vertex_x + step_offset,
        vertex_z,
    );
    let height_up = sample_heightmap_with_neighbors(
        terrain,
        chunk_index,
        size,
        vertex_x,
        vertex_z - step_offset,
    );
    let height_down = sample_heightmap_with_neighbors(
        terrain,
        chunk_index,
        size,
        vertex_x,
        vertex_z + step_offset,
    );

    // Calculate slope in X and Z directions.
    let derivative_x = (height_right - height_left) / (2.0 * world_scale * step as f32);
    let derivative_z = (height_up - height_down) / (2.0 * world_scale * step as f32);

    // Build a normalized vertex normal from the slope.
    let normal_x = -derivative_x;
    let normal_y = 1.0_f32;
    let normal_z = -derivative_z;
    let normal_length = (normal_x * normal_x + normal_y * normal_y + normal_z * normal_z).sqrt();
    [normal_x / normal_length, normal_y / normal_length, normal_z / normal_length]
}

fn build_terrain_mesh(
    terrain: &Terrain,
    subdivisions: u8,
    world_scale: f32,
    chunk_index: usize,
) -> TerrainMesh {
    let chunk = &terrain.chunks[chunk_index];
    let step = lod_step(chunk.lod);
    let vertices_per_side = (subdivisions as u32 / step) + 1;
    let vertex_spacing = world_scale * step as f32; // world units between vertices at this lod

    let world_offset_x = chunk.origin.x as f32 * subdivisions as f32 * world_scale;
    let world_offset_z = chunk.origin.y as f32 * subdivisions as f32 * world_scale;

    let mut vertices: Vec<TerrainVertex> =
        Vec::with_capacity((vertices_per_side * vertices_per_side) as usize);
    let mut indices: Vec<u32> = Vec::with_capacity(((vertices_per_side - 1).pow(2) * 6) as usize);

    for row in 0..vertices_per_side {
        for col in 0..vertices_per_side {
            let height_x = (col * step).min(subdivisions as u32 - 1);
            let height_z = (row * step).min(subdivisions as u32 - 1);

            let height = sample_heightmap(&chunk.heightmap, subdivisions as u32, height_x, height_z);

            let x = world_offset_x + col as f32 * vertex_spacing;
            let z = world_offset_z + row as f32 * vertex_spacing;

            let normal = calculate_normal(
                terrain,
                chunk_index,
                subdivisions as u32,
                height_x,
                height_z,
                world_scale,
                step,
            );

            let u = col as f32 / (vertices_per_side - 1) as f32;
            let v = row as f32 / (vertices_per_side - 1) as f32;

            vertices.push(TerrainVertex {
                position: [x, height, z],
                normal,
                uv: [u, v],
            });
        }
    }

    for row in 0..(vertices_per_side - 1) {
        for col in 0..(vertices_per_side - 1) {
            let tl = row * vertices_per_side + col;
            let tr = tl + 1;
            let bl = tl + vertices_per_side;
            let br = bl + 1;
            indices.extend_from_slice(&[tl, bl, tr, tr, bl, br]);
        }
    }

    TerrainMesh { vertices, indices }
}
