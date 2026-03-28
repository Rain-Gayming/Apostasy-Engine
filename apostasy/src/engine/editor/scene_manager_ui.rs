use std::path::Path;

use egui::{Context, ScrollArea, Window};

use crate::{
    engine::{
        editor::{EditorStorage, file_manager::file_dragging_ui},
        nodes::{scene::Scene, world::World},
    },
    log,
};

pub fn render_scene_manager(
    context: &mut Context,
    world: &mut World,
    editor_storage: &mut EditorStorage,
) {
    if !editor_storage.is_scene_manager_open {
        return;
    }

    // compute default pixel size from stored ratio (if available)
    let default_size = if let Some(r) = editor_storage.scene_manager_window_size {
        if world.window_size.x > 0.0 && world.window_size.y > 0.0 {
            [r[0] * world.window_size.x, r[1] * world.window_size.y]
        } else {
            [400.0, 500.0]
        }
    } else {
        [400.0, 500.0]
    };

    Window::new("Scene Manager")
        .default_size(default_size)
        .show(context, |ui| {
            // store the current window content size as a ratio of the main window
            let size = ui.available_size();
            if world.window_size.x > 0.0 && world.window_size.y > 0.0 {
                editor_storage.scene_manager_window_size = Some([
                    size.x / world.window_size.x,
                    size.y / world.window_size.y,
                ]);
            }
            if ui.button("Close").clicked() {
                editor_storage.is_scene_manager_open = false;
            }
            ui.collapsing("Add Scene", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut editor_storage.scene_name);
                });
                ui.add_space(4.0);

                let scene_path = format!("res/{}.scene", editor_storage.scene_name.clone());
                let can_add =
                    !editor_storage.scene_name.is_empty() && !Path::new(&scene_path).exists();

                ui.add_enabled_ui(can_add, |ui| {
                    if ui.button("Add Scene").clicked() {
                        let mut scene = Scene::new(scene_path);
                        scene.name = editor_storage.scene_name.clone();
                        world.serialize_scene_not_loaded(&scene).unwrap();
                        world.scene_manager.scenes.push(scene);
                        editor_storage.scene_name.clear();
                    }
                });
            });

            ui.separator();
            ui.collapsing("Scenes", |ui| {
                ScrollArea::vertical()
                    .id_salt("scenes_scroll")
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Add Scene");

                            let scene_button_text =
                                if let Some(path) = editor_storage.scene_to_add.clone() {
                                    path.to_string()
                                } else {
                                    "Drag scene here".to_string()
                                };

                            let (is_file, path) = file_dragging_ui(
                                ui,
                                editor_storage,
                                scene_button_text,
                                ".scene".to_string(),
                                "Scene".to_string(),
                            );

                            if is_file {
                                editor_storage.scene_to_add = Some(path.clone());
                                editor_storage.file_dragging = false;
                            }

                            if ui.button("Add Scene").clicked() {
                                let scene = world
                                    .scene_manager
                                    .load_scene(&editor_storage.scene_to_add.clone().unwrap())
                                    .unwrap();

                                for manager_scene in world.scene_manager.scenes.iter() {
                                    if manager_scene.path == scene.path {
                                        log!("Scene already exists");
                                        return;
                                    }
                                }

                                world.scene_manager.scene_paths.push(scene.path.clone());
                                world.scene_manager.scenes.push(scene);
                                println!("{:?}", world.scene_manager.scene_paths);
                                editor_storage.scene_to_add = None;
                            }
                        });

                        let scene_names: Vec<String> = world
                            .scene_manager
                            .scenes
                            .iter()
                            .map(|s| s.path.clone())
                            .collect();

                        for name in scene_names {
                            ui.horizontal(|ui| {
                                let new_name = name.clone();
                                ui.label(new_name.clone());

                                let (is_primary, scene_exists) = world
                                    .scene_manager
                                    .scenes
                                    .iter()
                                    .find(|s| s.name == new_name)
                                    .map(|s| (s.is_primary, true))
                                    .unwrap_or((false, false));

                                if scene_exists {
                                    let mut primary = is_primary;
                                    if ui.checkbox(&mut primary, "Primary").clicked() {
                                        world
                                            .scene_manager
                                            .set_scene_primary(&new_name, !is_primary);
                                        if let Some(s) = world
                                            .scene_manager
                                            .scenes
                                            .iter()
                                            .find(|s| s.name == new_name)
                                        {
                                            world.serialize_scene_not_loaded(s).unwrap();
                                        }
                                    }
                                }

                                ui.add_space(4.0);
                                if ui.button("load").clicked() {
                                    let scene = world.scene_manager.load_scene(&name);
                                    world.scene = scene.unwrap();
                                }
                                if ui.button("❌").clicked() {
                                    world.scene_manager.remove_scene(&name);
                                }
                            });
                        }
                    });
            });
        });
}
