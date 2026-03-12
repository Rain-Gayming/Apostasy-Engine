use egui::{ScrollArea, Ui};

use crate::engine::{editor::EditorStorage, nodes::world::World};

pub fn render_inspector(ui: &mut Ui, world: &mut World, editor_storage: &mut EditorStorage) {
    ui.separator();
    ui.label("Components");

    if let Some(id) = editor_storage.selected_node {
        ScrollArea::vertical()
            .id_salt("inspector_scroll")
            .show(ui, |ui| {
                let node = world.get_node_mut(id);

                ui.horizontal(|ui| {
                    ui.label("Name: ");
                    ui.text_edit_singleline(&mut node.name);
                });

                if let Some(parent) = &node.parent {
                    ui.label(format!("Parent Node: {}", parent));
                }
                ui.separator();

                ui.horizontal(|ui| {
                    let text_edit =
                        ui.text_edit_singleline(&mut editor_storage.component_text_edit);

                    if text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let res = node.add_component_by_name(&editor_storage.component_text_edit);
                        if res.is_err() {
                            editor_storage.component_text_edit = format!(
                                "Component ({}) not found",
                                editor_storage.component_text_edit
                            );
                        }
                    }

                    if ui.button("Add Component").clicked() {
                        let res = node.add_component_by_name(&editor_storage.component_text_edit);
                        if res.is_err() {
                            editor_storage.component_text_edit = format!(
                                "Component ({}) not found",
                                editor_storage.component_text_edit
                            );
                        }
                    }
                });

                ui.separator();

                let mut to_remove: Option<usize> = None;

                for (i, component) in node.components.iter_mut().enumerate() {
                    if component.inspect(ui, editor_storage) {
                        to_remove = Some(i);
                    }
                }

                if let Some(i) = to_remove {
                    node.components.remove(i);
                }

                ui.allocate_space(ui.available_size());
            });
    }
}
