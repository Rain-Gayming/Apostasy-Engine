use egui::{CentralPanel, Context, Slider, Window};

use crate::engine::{editor::EditorStorage, nodes::world::World};

#[derive(Default, PartialEq, Eq)]
pub enum TerrainEditMode {
    #[default]
    Raise,
    Lower,
}

#[derive(Default)]
pub struct TerrainEditorSettings {
    pub edit_mode: TerrainEditMode,
    pub edit_strength: f32,
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
                    ui.label("Edit Strength: ");
                    ui.add(Slider::new(
                        &mut editor_storage.terrain_editor_settings.edit_strength,
                        0.0..=32.0,
                    ))
                });
            });

            ui.allocate_space(ui.available_size());
        });
}
