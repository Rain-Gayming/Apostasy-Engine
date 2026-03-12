use egui::{ScrollArea, Ui};
use gltf::material::AlphaMode;

use crate::engine::{
    assets::handle::Handle,
    editor::{EditorStorage, file_manager::file_dragging_ui},
    nodes::world::World,
    rendering::models::material::MaterialAsset,
};

pub fn asset_editor(ui: &mut Ui, _world: &mut World, editor_storage: &mut EditorStorage) {
    ui.separator();
    if let Some(path) = &editor_storage.selected_tree_node.clone() {
        ScrollArea::vertical()
            .id_salt("asset_editor_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if path.ends_with(".material") {
                    ui.label("MATERIAL");

                    let asset_server = editor_storage.asset_server.clone();
                    let asset_server = asset_server.write().unwrap();
                    let material_handle: Handle<MaterialAsset> =
                        asset_server.load(path[4..].to_string()).unwrap();
                    let mut material = asset_server.get_mut(material_handle).unwrap();

                    ui.horizontal(|ui| {
                        ui.label("base color:");
                        let mut r = material.base_color[0].clone() as f64;
                        let mut g = material.base_color[1].clone() as f64;
                        let mut b = material.base_color[2].clone() as f64;
                        let mut a = material.base_color[3].clone() as f64;
                        ui.add(egui::DragValue::new(&mut r).speed(0.01));
                        ui.add(egui::DragValue::new(&mut g).speed(0.01));
                        ui.add(egui::DragValue::new(&mut b).speed(0.01));
                        ui.add(egui::DragValue::new(&mut a).speed(0.01));
                        let new = [r as f32, g as f32, b as f32, a as f32];
                        if new != material.base_color {
                            material.base_color = new;
                            material.save(path.clone());
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("metallic:");
                        let mut before = material.metallic;
                        ui.add(egui::DragValue::new(&mut before).speed(0.01));
                        if material.metallic != before {
                            material.metallic = before;
                            material.save(path.clone());
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("roughness:");
                        let mut before = material.roughness.clone();
                        ui.add(egui::DragValue::new(&mut before).speed(0.01));
                        if material.roughness != before {
                            material.roughness = before;
                            material.save(path.clone());
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("emissive:");
                        let mut r = material.emissive[0].clone() as f64;
                        let mut g = material.emissive[1].clone() as f64;
                        let mut b = material.emissive[2].clone() as f64;
                        ui.add(egui::DragValue::new(&mut r).speed(0.01));
                        ui.add(egui::DragValue::new(&mut g).speed(0.01));
                        ui.add(egui::DragValue::new(&mut b).speed(0.01));
                        let new = [r as f32, g as f32, b as f32];
                        if new != material.emissive {
                            material.emissive = new;
                            material.save(path.clone());
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("alpha mode:");
                        let mut mode = match material.alpha_mode {
                            AlphaMode::Opaque => 0usize,
                            AlphaMode::Mask => 1usize,
                            AlphaMode::Blend => 2usize,
                        };
                        egui::ComboBox::from_label("")
                            .selected_text(match mode {
                                0 => "OPAQUE",
                                1 => "MASK",
                                _ => "BLEND",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut mode, 0, "OPAQUE");
                                ui.selectable_value(&mut mode, 1, "MASK");
                                ui.selectable_value(&mut mode, 2, "BLEND");
                            });
                        let new_mode = match mode {
                            0 => AlphaMode::Opaque,
                            1 => AlphaMode::Mask,
                            _ => AlphaMode::Blend,
                        };
                        if new_mode != material.alpha_mode {
                            material.alpha_mode = new_mode;
                            material.save(path.clone());
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("alpha cutoff:");
                        let mut before = material.alpha_cutoff.clone();
                        ui.add(egui::DragValue::new(&mut before).speed(0.01));
                        if material.alpha_cutoff != before {
                            material.alpha_cutoff = before;
                            material.save(path.clone());
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("double sided:");
                        let mut before = material.double_sided.clone();
                        ui.checkbox(&mut before, "");
                        if material.double_sided != before {
                            material.double_sided = before;
                            material.save(path.clone());
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("textures resolved:");
                        let mut before = material.textures_resolved.clone();
                        ui.checkbox(&mut before, "");
                        if material.textures_resolved != before {
                            material.textures_resolved = before;
                            material.save(path.clone());
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("albedo texture:");

                        let albedo_path = if material.albedo_texture_name.is_empty() {
                            "No texture".to_string()
                        } else {
                            material
                                .albedo_texture_name
                                .split(".")
                                .next()
                                .unwrap()
                                .to_string()
                        };

                        let (is_file, path) = file_dragging_ui(
                            ui,
                            editor_storage,
                            albedo_path,
                            ".material".to_string(),
                            "Material".to_string(),
                        );

                        if is_file {
                            material.albedo_texture_name = path.clone();
                            material.albedo_handle = None;
                            material.textures_resolved = false;
                            material.save(path.clone());

                            println!("A: {:?}", material.albedo_texture_name);

                            editor_storage.file_dragging = false;
                        }
                    });
                }
            });
    }
}
