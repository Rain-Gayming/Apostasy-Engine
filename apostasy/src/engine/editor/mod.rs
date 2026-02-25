use crate::{
    self as apostasy,
    engine::{
        editor::inspectable::InspectValue,
        nodes::{
            camera::Camera,
            transform::{Transform, calculate_rotation},
        },
    },
};
use std::path::{Path, PathBuf};

use crate::{engine::nodes::World, log};
use apostasy_macros::editor_ui;
use egui::{
    Align2, CollapsingHeader, Color32, Context, FontFamily, FontId, RichText, ScrollArea, Sense,
    Stroke, Ui, Vec2, Window, pos2,
};

pub mod console_commands;
pub mod inspectable;

/// Storage for all information needed by the editor
pub struct EditorStorage {
    pub component_text_edit: String,
    pub file_tree: Option<FileNode>,

    pub is_console_open: bool,
    pub console_log: Vec<String>,
    pub console_size: Vec2,
    pub console_filter: String,
    pub console_command: String,

    pub selected_node: String,
}

impl Default for EditorStorage {
    fn default() -> Self {
        Self {
            component_text_edit: String::new(),
            file_tree: Some(FileNode::from_path(Path::new("res/"))),

            is_console_open: false,
            console_log: Vec::new(),
            console_size: Vec2::new(100.0, 100.0),
            console_filter: String::new(),
            console_command: String::new(),
            selected_node: "".to_string(),
        }
    }
}
//
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

#[editor_ui]
pub fn hierarchy_ui(context: &mut Context, world: &mut World, editor_storage: &mut EditorStorage) {
    Window::new("Hierarchy")
        .default_size([100.0, 300.0])
        .show(context, |ui| {
            if ui.button("New Entity").clicked() {
                world.add_new_node();
            }

            ScrollArea::vertical()
                .id_salt("entities_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(4.0);
                    let mut names = Vec::new();
                    for node in world.get_all_nodes_mut() {
                        let base_name = node.name.clone();

                        // check if name already exists
                        if names.contains(&base_name) {
                            let mut counter = 1;
                            let mut new_name = format!("{} ({})", base_name, counter);

                            // keep incrementing until it finds an unused name
                            while names.contains(&new_name) {
                                counter += 1;
                                new_name = format!("{} ({})", base_name, counter);
                            }

                            node.name = new_name.clone();
                            names.push(new_name);
                        } else {
                            names.push(base_name);
                        }

                        ui.horizontal(|ui| {
                            ui.add_space(4.0);

                            let desired_size = Vec2::new(ui.available_width() - 5.0, 20.0);
                            let (rect, response) =
                                ui.allocate_exact_size(desired_size, Sense::click());

                            let selected;
                            if !editor_storage.selected_node.is_empty() {
                                selected = editor_storage.selected_node == node.name;
                            } else {
                                selected = false;
                            }

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
                                rect.left_center() + Vec2::new(4.0, 0.0),
                                Align2::LEFT_CENTER,
                                node.name.clone(),
                                FontId::new(11.0, FontFamily::Proportional),
                                Color32::WHITE,
                            );
                            if response.clicked() {
                                editor_storage.selected_node = node.name.clone();
                            }
                        });
                    }

                    ui.allocate_space(ui.available_size());
                });
        });
}

#[editor_ui]
pub fn inspector_ui(context: &mut Context, world: &mut World, editor_storage: &mut EditorStorage) {
    Window::new("Inspector")
        .default_size([100.0, 100.0])
        .show(context, |ui| {
            if !editor_storage.selected_node.is_empty() {
                let text_edit = ui.text_edit_singleline(&mut editor_storage.component_text_edit);

                if text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    || ui.button("Add Component").clicked()
                {
                    // if world
                    //     .get_component_info_by_name(&editor_storage.component_text_edit)
                    //     .is_some()
                    // {
                    //     world.add_default_component_by_name(
                    //         editor_storage.selected_entity,
                    //         &editor_storage.component_text_edit,
                    //     );
                    // } else {
                    //     editor_storage.component_text_edit = format!(
                    //         "Component ({}) not found",
                    //         editor_storage.component_text_edit
                    //     );
                    // }
                }

                ui.separator();

                ui.label("Components");

                let node = world.get_node_with_name_mut(&editor_storage.selected_node);

                ui.label(format!("Name: {}", node.editing_name));
                let text_edit = ui.text_edit_singleline(&mut node.editing_name);
                if text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    node.name = node.editing_name.clone();
                    editor_storage.selected_node = node.name.clone();
                }
                ui.separator();

                if let Some(mut transform) = node.get_component_mut::<Transform>() {
                    transform.inspect_value(ui);
                    calculate_rotation(&mut transform);
                }
                if let Some(camera) = node.get_component_mut::<Camera>() {
                    camera.inspect_value(ui);
                }

                ui.allocate_space(ui.available_size());
            }
        });
}

#[editor_ui]
pub fn file_tree_ui(context: &mut Context, _world: &mut World, editor_storage: &mut EditorStorage) {
    Window::new("Files")
        .default_size([100.0, 300.0])
        .show(context, |ui| {
            ui.style_mut().visuals.override_text_color = Some(Color32::from_gray(210));
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
                            RichText::new("res/ not found").color(Color32::from_rgb(200, 80, 80)),
                        );
                    }

                    ui.allocate_space(ui.available_size());
                });
        });
}
