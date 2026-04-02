use egui::{CentralPanel, Context, Slider, Window};
use std::time::Instant;

use crate::engine::{editor::EditorStorage, nodes::world::World};

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum TerrainEditMode {
    #[default]
    Raise,
    Lower,
}

// Editor tool modes for terrain interaction.
// Edit: paint existing chunks, Delete: remove a chunk tile, PaintNew: spawn a new chunk,
// Smooth: soften the selected terrain area.
#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum TerrainEditTool {
    #[default]
    Edit,
    Delete,
    PaintNew,
    Smooth,
}

pub struct TerrainEditorSettings {
    pub edit_mode: TerrainEditMode,
    pub edit_tool: TerrainEditTool,
    pub edit_strength: f32,
    pub brush_radius: u32,
    pub last_paint: Instant,
}

impl Default for TerrainEditorSettings {
    fn default() -> Self {
        Self {
            edit_mode: TerrainEditMode::Raise,
            edit_tool: TerrainEditTool::Edit,
            edit_strength: 0.0,
            brush_radius: 1,
            last_paint: Instant::now(),
        }
    }
}

pub fn render_terrain_edtor(
    context: &mut Context,
    world: &mut World,
    editor_storage: &mut EditorStorage,
) {
    if !editor_storage.is_terrain_editor_open {
        return;
    }

    Window::new("Terrain Editor")
        .default_size((300.0, 120.0))
        .resizable(true)
        .show(context, |ui| {
            CentralPanel::default().show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut editor_storage.terrain_editor_settings.edit_mode,
                        TerrainEditMode::Raise,
                        "Raise",
                    );
                    ui.selectable_value(
                        &mut editor_storage.terrain_editor_settings.edit_mode,
                        TerrainEditMode::Lower,
                        "Lower",
                    );
                });

                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut editor_storage.terrain_editor_settings.edit_tool,
                        TerrainEditTool::Edit,
                        "Edit Existing",
                    );
                    ui.selectable_value(
                        &mut editor_storage.terrain_editor_settings.edit_tool,
                        TerrainEditTool::Delete,
                        "Delete Chunk",
                    );
                    ui.selectable_value(
                        &mut editor_storage.terrain_editor_settings.edit_tool,
                        TerrainEditTool::PaintNew,
                        "Paint New Chunk",
                    );
                    ui.selectable_value(
                        &mut editor_storage.terrain_editor_settings.edit_tool,
                        TerrainEditTool::Smooth,
                        "Smooth",
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Edit Strength: ");
                    ui.add(Slider::new(
                        &mut editor_storage.terrain_editor_settings.edit_strength,
                        0.0..=32.0,
                    ))
                });

                ui.horizontal(|ui| {
                    ui.label("Brush Radius: ");
                    ui.add(Slider::new(
                        &mut editor_storage.terrain_editor_settings.brush_radius,
                        0..=8,
                    ));
                });

            });

            ui.allocate_space(ui.available_size());
        });
}
