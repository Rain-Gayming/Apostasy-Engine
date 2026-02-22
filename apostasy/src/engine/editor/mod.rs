use std::path::{Path, PathBuf};

use crate::{
    self as apostasy,
    engine::{
        ecs::{
            World,
            components::{
                name::Name,
                transform::{Transform, calculate_rotation},
            },
            entity::Entity,
        },
        rendering::models::model::{ModelLoader, ModelRenderer, does_model_exist},
    },
    log,
};
use apostasy_macros::{Resource, ui};
use egui::{
    Align2, CollapsingHeader, Color32, Context, FontFamily, FontId, RichText, ScrollArea, Sense,
    Stroke, Ui, Vec2, Window, pos2, vec2,
};

pub mod console_commands;

/// Storage for all information needed by the editor
#[derive(Resource)]
pub struct EditorStorage {
    pub selected_entity: Entity,
    pub component_text_edit: String,
    pub file_tree: Option<FileNode>,

    pub is_console_open: bool,
    pub console_log: Vec<String>,
    pub console_size: Vec2,
    pub console_filter: String,
    pub console_command: String,
}

impl Default for EditorStorage {
    fn default() -> Self {
        Self {
            selected_entity: Entity::from_raw(0),
            component_text_edit: String::new(),
            file_tree: Some(FileNode::from_path(Path::new("res/"))),

            is_console_open: false,
            console_log: Vec::new(),
            console_size: Vec2::new(100.0, 100.0),
            console_filter: String::new(),
            console_command: String::new(),
        }
    }
}

/// A node in the file tree
pub struct FileNode {
    pub name: String,
    pub path: PathBuf,
    pub children: Vec<FileNode>,
    pub is_dir: bool,
}
impl FileNode {
    pub fn from_path(path: &Path) -> Self {
        // get the name of the file or directory
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let mut children = Vec::new();
        let is_dir = path.is_dir();

        // if the path is a directory, read the entries and sort them
        if is_dir && let Ok(entries) = std::fs::read_dir(path) {
            let mut entries: Vec<_> = entries.flatten().collect();
            entries.sort_by(|a, b| {
                let a_is_dir = a.path().is_dir();
                let b_is_dir = b.path().is_dir();
                b_is_dir
                    .cmp(&a_is_dir)
                    .then(a.file_name().cmp(&b.file_name()))
            });
            // recursively create FileNodes for each entry
            for entry in entries {
                children.push(FileNode::from_path(&entry.path()));
            }
        }

        Self {
            name,
            path: path.to_path_buf(),
            children,
            is_dir,
        }
    }
}
fn render_file_tree(ui: &mut Ui, node: &FileNode, depth: usize) {
    let indent = depth as f32 * 12.0;

    if node.is_dir {
        let id = ui.make_persistent_id(&node.path);
        let default_open = depth == 0; // root open by default
        CollapsingHeader::new(&node.name)
            .id_salt(id)
            .default_open(default_open)
            .icon(|ui, openness, response| {
                // Simple triangle icon
                let rect = response.rect;
                let color = Color32::from_gray(180);
                let points = if openness > 0.5 {
                    // pointing down
                    vec![
                        pos2(rect.left(), rect.top()),
                        pos2(rect.right(), rect.top()),
                        pos2(rect.center().x, rect.bottom()),
                    ]
                } else {
                    // pointing right
                    vec![
                        pos2(rect.left(), rect.top()),
                        pos2(rect.right(), rect.center().y),
                        pos2(rect.left(), rect.bottom()),
                    ]
                };
                ui.painter()
                    .add(epaint::Shape::convex_polygon(points, color, Stroke::NONE));
            })
            .show(ui, |ui| {
                ui.add_space(2.0);
                for child in &node.children {
                    render_file_tree(ui, child, depth + 1);
                }
            });
    } else {
        // File entry
        ui.horizontal(|ui| {
            ui.add_space(indent);
            let ext = node.path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let icon = match ext {
                "png" | "jpg" | "jpeg" | "webp" => "🖼",
                "glsl" | "vert" | "frag" | "wgsl" => "🔷",
                "rs" => "🦀",
                "toml" | "json" | "yaml" | "yml" => "📄",
                "ttf" | "otf" => "🔤",
                "wav" | "mp3" | "ogg" => "🔊",
                _ => "📃",
            };
            let label = ui.selectable_label(false, format!("{} {}", icon, node.name));
            if label.double_clicked() {
                log!("Open: {:?}", node.path); // hook into your editor's open-file logic
            }
            label.on_hover_text(node.path.to_string_lossy());
        });
    }
}

#[ui]
pub fn hierarchy_ui(context: &mut Context, world: &mut World) {
    Window::new("Hierarchy")
        .default_size([100.0, 300.0])
        .show(context, |ui| {
            if ui.button("New Entity").clicked() {
                world.spawn();
            }

            world.with_resource_mut(|editor_storage: &mut EditorStorage| {
                ScrollArea::vertical()
                    .id_salt("entities_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add_space(4.0);
                        // iterate over all entities and add buttons for them

                        for entity in world.get_all_entities() {
                            let world_entity = world.entity(entity);
                            #[allow(unused_assignments)]
                            let name;
                            // if the entity has a name component, use that
                            // otherwise use the entity id
                            if let Some(name_component) = world_entity.get_ref::<Name>() {
                                name = name_component.0.clone();
                            } else {
                                name = format!("{:?}", entity.0.index);
                            }
                            ui.horizontal(|ui| {
                                ui.add_space(4.0);

                                let desired_size = vec2(ui.available_width() - 5.0, 20.0);
                                let (rect, response) =
                                    ui.allocate_exact_size(desired_size, Sense::click());

                                let selected = editor_storage.selected_entity == entity;

                                // hover/click/ignored colors
                                let color = if selected {
                                    Color32::from_rgb(0, 120, 215)
                                } else if response.hovered() {
                                    Color32::from_gray(70)
                                } else {
                                    Color32::TRANSPARENT
                                };

                                // draw a background
                                ui.painter().rect_filled(rect, 0.0, color);
                                // draw the name
                                ui.painter().text(
                                    rect.left_center() + vec2(4.0, 0.0),
                                    Align2::LEFT_CENTER,
                                    name,
                                    FontId::new(11.0, FontFamily::Proportional),
                                    Color32::WHITE,
                                );
                                if response.clicked() {
                                    editor_storage.selected_entity = entity;
                                }
                            });
                        }

                        ui.allocate_space(ui.available_size());
                    });
            });
        });
}

#[ui]
pub fn inspector_ui(context: &mut Context, world: &mut World) {
    Window::new("Inspector")
        .default_size([100.0, 100.0])
        .show(context, |ui| {
            world.with_resources::<(EditorStorage, ModelLoader), _>(
                |(editor_storage, model_loader)| {
                    if editor_storage.selected_entity != Entity::from_raw(0) {
                        let text_edit =
                            ui.text_edit_singleline(&mut editor_storage.component_text_edit);

                        if text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
                            || ui.button("Add Component").clicked()
                        {
                            if world
                                .get_component_info_by_name(&editor_storage.component_text_edit)
                                .is_some()
                            {
                                world.add_default_component_by_name(
                                    editor_storage.selected_entity,
                                    &editor_storage.component_text_edit,
                                );
                            } else {
                                editor_storage.component_text_edit = format!(
                                    "Component ({}) not found",
                                    editor_storage.component_text_edit
                                );
                            }
                        }

                        ui.separator();

                        ui.label("Components");

                        let entity = world.entity(editor_storage.selected_entity);

                        if let Some(mut name) = entity.get_mut::<Name>() {
                            ui.label(format!("Name: {}", name.0));
                            ui.text_edit_singleline(&mut name.0);
                            ui.separator();
                        }

                        if let Some(mut transform) = entity.get_mut::<Transform>() {
                            ui.label("TRANSFORM");
                            ui.label(format!("Position: {:?}", transform.position));
                            ui.add_space(4.0);
                            ui.add(egui::DragValue::new(&mut transform.position.x).speed(1));
                            ui.add(egui::DragValue::new(&mut transform.position.y).speed(1));
                            ui.add(egui::DragValue::new(&mut transform.position.z).speed(1));
                            ui.add_space(4.0);
                            ui.label(format!("Rotation: {:?}", transform.rotation));
                            ui.add(egui::DragValue::new(&mut transform.yaw).speed(1));
                            ui.add(egui::DragValue::new(&mut transform.pitch).speed(1));
                            ui.add_space(4.0);
                            calculate_rotation(&mut transform);
                            ui.separator();
                        }
                        if let Some(mut model_renderer) = entity.get_mut::<ModelRenderer>() {
                            ui.label("MODEL RENDERER");
                            ui.label(format!("Model: {}", model_renderer.0));
                            let text_edit = ui.text_edit_singleline(&mut model_renderer.1);
                            if text_edit.lost_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter))
                                || ui.button("Load Model").clicked()
                            {
                                let name = model_renderer.1.clone() + ".glb";
                                if does_model_exist(name.as_str(), model_loader) {
                                    model_renderer.0 = model_renderer.1.clone();
                                } else {
                                    let attempted_model = model_renderer.1.clone();
                                    model_renderer.1 =
                                        format!("Model: ({}) does not exist", attempted_model);
                                }
                            }
                            ui.separator();
                        }

                        ui.allocate_space(ui.available_size());
                    }
                },
            );
        });
}

#[ui]
pub fn file_tree_ui(context: &mut Context, world: &mut World) {
    Window::new("Files")
        .default_size([100.0, 300.0])
        .show(context, |ui| {
            ui.style_mut().visuals.override_text_color = Some(Color32::from_gray(210));
            world.with_resource_mut(|editor_storage: &mut EditorStorage| {
                ScrollArea::vertical()
                    .id_salt("files_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new("📁 res/")
                                    .size(11.0)
                                    .color(Color32::from_gray(150)),
                            );
                        });
                        ui.separator();
                        if let Some(tree) = &editor_storage.file_tree {
                            render_file_tree(ui, tree, 0);
                        } else {
                            ui.label(
                                RichText::new("res/ not found")
                                    .color(Color32::from_rgb(200, 80, 80)),
                            );
                        }

                        ui.allocate_space(ui.available_size());
                    });
            });
        });
}

pub fn get_all_entities(world: &World) -> Vec<Entity> {
    world.crust.mantle(|mantle| {
        let mut entities: Vec<Entity> = Vec::new();

        for archetype in mantle.core.archetypes.slots.iter() {
            if let Some(data) = &archetype.data {
                for entity in data.entities.iter() {
                    entities.push(*entity);
                }
            }
        }

        entities
    })
}
