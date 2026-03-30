use egui::{Button, Context, Window};

use crate::engine::{
    editor::{
        EditorStorage, EngineSettingsTab, input_manager_ui::render_input_manager,
        renderer_settings::render_renderer_settings, scene_manager_ui::render_scene_manager,
    },
    nodes::world::World,
};

pub fn render_engine_settings_ui(
    context: &mut Context,
    world: &mut World,
    editor_storage: &mut EditorStorage,
) {
    if !editor_storage.is_engine_settings_open {
        return;
    }

    Window::new("Engine Settings")
        .resizable(true)
        .show(context, |ui| {
            egui::SidePanel::left("Side Bar")
                .resizable(true)
                .default_width(150.0)
                .min_width(80.0)
                .max_width(200.0)
                .show_inside(ui, |ui| {
                    ui.separator();
                    ui.add_sized([ui.available_width(), 0.0], egui::Label::new("Game"));
                    let inputs_response = ui
                        .add_sized([ui.available_width(), 0.0], egui::Button::new("Inputs"))
                        .clicked();

                    if inputs_response {
                        editor_storage.open_engine_settings_tab = EngineSettingsTab::Inputs;
                    }

                    let scenes_response = ui
                        .add_sized([ui.available_width(), 0.0], egui::Button::new("Scenes"))
                        .clicked();

                    if scenes_response {
                        editor_storage.open_engine_settings_tab = EngineSettingsTab::Scenes;
                    }

                    ui.separator();
                    let response = ui
                        .add_sized([ui.available_width(), 0.0], egui::Button::new("Renderer"))
                        .clicked();

                    if response {
                        editor_storage.open_engine_settings_tab = EngineSettingsTab::Renderer;
                    }
                });

            match editor_storage.open_engine_settings_tab {
                EngineSettingsTab::Inputs => {
                    render_input_manager(ui, world, editor_storage);
                }
                EngineSettingsTab::Scenes => {
                    render_scene_manager(ui, world, editor_storage);
                }
                EngineSettingsTab::Renderer => {
                    render_renderer_settings(ui, editor_storage);
                }
            }

            ui.separator();
        });
}
