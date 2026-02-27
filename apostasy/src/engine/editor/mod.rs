use crate::{
    self as apostasy,
    engine::{
        editor::inspectable::InspectValue,
        nodes::{
            Node,
            components::{
                camera::Camera, collider::Collider, physics::Physics, transform::Transform,
                velocity::Velocity,
            },
        },
        rendering::models::model::ModelRenderer,
        windowing::input_manager::{KeyAction, KeyBind, MouseBind},
    },
};
use std::path::{Path, PathBuf};

use crate::{engine::nodes::World, log};
use apostasy_macros::editor_ui;
use egui::{
    Align2, CollapsingHeader, Color32, Context, FontFamily, FontId, RichText, ScrollArea, Sense,
    Stroke, Ui, Vec2, Window, collapsing_header::CollapsingState, pos2,
};
use winit::{event::MouseButton, keyboard::PhysicalKey};

pub mod console_commands;
pub mod inspectable;

/// Storage for all information needed by the editor
pub struct EditorStorage {
    pub component_text_edit: String,

    pub file_tree_search: String,
    pub file_tree: Option<FileNode>,

    pub is_console_open: bool,
    pub console_log: Vec<String>,
    pub console_size: Vec2,
    pub console_filter: String,
    pub console_command: String,

    pub selected_node: String,

    pub is_editor_open: bool,

    // keybind editor
    pub keybind_name: String,
    pub keybind_key_code: String,
    pub keybind_action: KeyAction,
    pub keybind_error: Option<String>,

    // mousebind editor
    pub mousebind_name: String,
    pub mousebind_button: MouseButton,
    pub mousebind_action: KeyAction,

    pub dragging_node: Option<String>,
    pub drag_target: Option<DragTarget>,
}
pub enum DragTarget {
    Parent(String), // drop onto root to un-parent
    Root,           // drop onto root to un-parent
}
impl Default for EditorStorage {
    fn default() -> Self {
        Self {
            component_text_edit: String::new(),

            file_tree_search: String::new(),
            file_tree: Some(FileNode::from_path(Path::new("res/"))),

            is_console_open: false,
            console_log: Vec::new(),
            console_size: Vec2::new(100.0, 100.0),
            console_filter: String::new(),
            console_command: String::new(),
            selected_node: "".to_string(),
            is_editor_open: true,

            // keybind editor
            keybind_name: String::new(),
            keybind_key_code: String::new(),
            keybind_action: KeyAction::Press,
            keybind_error: None,

            // mousebind editor
            mousebind_name: String::new(),
            mousebind_button: MouseButton::Left,
            mousebind_action: KeyAction::Press,

            dragging_node: None,
            drag_target: None,
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

#[editor_ui]
pub fn hierarchy_ui(context: &mut Context, world: &mut World, editor_storage: &mut EditorStorage) {
    if !editor_storage.is_editor_open {
        return;
    }

    Window::new("Hierarchy")
        .default_size([100.0, 300.0])
        .show(context, |ui| {
            if ui.button("New Entity").clicked() {
                world.add_new_node();
            }
            if ui.button("Save Scene").clicked() {
                world.serialize_scene().unwrap();
            }
            if ui.button("Load Scene").clicked() {
                world.deserialize_scene("Scene".to_string()).unwrap();
            }

            ScrollArea::vertical()
                .id_salt("entities_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(4.0);

                    let root_children: Vec<Node> = world.scene.root_node.children.clone();
                    for node in &root_children {
                        draw_node(ui, node, editor_storage, 0);
                    }

                    // Drop onto empty space = move to root
                    let empty_space = ui.allocate_response(ui.available_size(), Sense::hover());
                    if empty_space.hovered() && editor_storage.dragging_node.is_some() {
                        editor_storage.drag_target = Some(DragTarget::Root);
                    }
                });

            // Commit the drag on mouse release
            if ui.input(|i| i.pointer.any_released())
                && let Some(dragging) = editor_storage.dragging_node.take()
            {
                let target = editor_storage.drag_target.take();
                let root = &mut *world.scene.root_node;

                match target {
                    Some(DragTarget::Parent(parent_name)) if parent_name != dragging => {
                        if let Some(node) = root.remove_node_by_name(&dragging) {
                            root.insert_under(&parent_name, node);
                        }
                    }
                    Some(DragTarget::Root) | None => {
                        if let Some(mut node) = root.remove_node_by_name(&dragging) {
                            node.parent = None;
                            root.children.push(node);
                        }
                    }
                    _ => {}
                }
            }
        });
}

fn draw_node(ui: &mut egui::Ui, node: &Node, editor_storage: &mut EditorStorage, depth: usize) {
    let indent = depth as f32 * 10.0;
    let has_children = !node.children.is_empty();
    let selected = editor_storage.selected_node == node.name;

    if has_children {
        let id = ui.make_persistent_id(format!("node_{}", node.name));

        CollapsingState::load_with_default_open(ui.ctx(), id, false)
            .show_header(ui, |ui: &mut egui::Ui| {
                ui.add_space(indent);
                draw_node_row(ui, node, selected, editor_storage);
            })
            .body(|ui| {
                for child in &node.children {
                    draw_node(ui, child, editor_storage, depth + 1);
                }
            });
    } else {
        ui.horizontal(|ui| {
            if let Some(parent) = &node.parent {
                let indent = indent
                    + if parent == &"root".to_string() {
                        18.0
                    } else {
                        0.0
                    };
                ui.add_space(indent);
                draw_node_row(ui, node, selected, editor_storage);
            } else {
                ui.add_space(indent + 18.0);
                draw_node_row(ui, node, selected, editor_storage);
            }
        });
    }
}

fn draw_node_row(
    ui: &mut egui::Ui,
    node: &Node,
    selected: bool,
    editor_storage: &mut EditorStorage,
) {
    let desired_size = Vec2::new(ui.available_width() - 5.0, 20.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

    if response.drag_started() {
        editor_storage.dragging_node = Some(node.name.clone());
    }

    // render a tooltip with the node name when dragging
    if editor_storage.dragging_node.as_deref() == Some(&node.name) && response.dragged() {
        egui::Tooltip::always_open(
            ui.ctx().clone(),
            ui.layer_id(),
            egui::Id::new("drag_tooltip"),
            response.rect,
        )
        .at_pointer()
        .show(|ui| {
            ui.label(&node.name);
        });
    }

    // get the pointer position
    let pointer_pos = ui.ctx().pointer_latest_pos();

    // highlight as drop target when something is being dragged over it
    let is_drag_target =
        editor_storage.dragging_node.is_some() && pointer_pos.is_some_and(|pos| rect.contains(pos));

    // detects and stores the current hovered node when dragging
    if is_drag_target {
        editor_storage.drag_target = Some(DragTarget::Parent(node.name.clone()));
    }

    let color = if selected {
        Color32::from_rgb(0, 120, 215)
    } else if is_drag_target {
        Color32::from_rgb(40, 100, 40)
    } else if response.hovered() {
        Color32::from_gray(70)
    } else {
        Color32::TRANSPARENT
    };

    // draw a line above the row to show insert position
    if is_drag_target {
        ui.painter().line_segment(
            [rect.left_bottom(), rect.right_bottom()],
            egui::Stroke::new(2.0, Color32::from_rgb(100, 200, 100)),
        );
    }

    // draw the color needed and the name
    ui.painter().rect_filled(rect, 0.0, color);
    ui.painter().text(
        rect.left_center() + Vec2::new(4.0, 0.0),
        Align2::LEFT_CENTER,
        &node.name,
        FontId::new(11.0, FontFamily::Proportional),
        Color32::WHITE,
    );

    if response.clicked() {
        editor_storage.selected_node = node.name.clone();
    }
}
#[editor_ui]
pub fn inspector_ui(context: &mut Context, world: &mut World, editor_storage: &mut EditorStorage) {
    if !editor_storage.is_editor_open {
        return;
    }

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
                if let Some(parent) = &node.parent {
                    ui.label(format!("Parent Node: {}", parent));
                }
                let text_edit = ui.text_edit_singleline(&mut node.editing_name);
                if text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    node.name = node.editing_name.clone();
                    editor_storage.selected_node = node.name.clone();
                }
                ui.separator();

                if let Some(transform) = node.get_component_mut::<Transform>() {
                    transform.inspect_value(ui);
                }
                if let Some(camera) = node.get_component_mut::<Camera>() {
                    camera.inspect_value(ui);
                }
                if let Some(model) = node.get_component_mut::<ModelRenderer>() {
                    model.inspect_value(ui);
                }
                if let Some(velocity) = node.get_component_mut::<Velocity>() {
                    velocity.inspect_value(ui);
                }
                if let Some(physics) = node.get_component_mut::<Physics>() {
                    physics.inspect_value(ui);
                }
                if let Some(collider) = node.get_component_mut::<Collider>() {
                    collider.inspect_value(ui);
                }

                ui.allocate_space(ui.available_size());
            }
        });
}

#[editor_ui]
pub fn file_tree_ui(context: &mut Context, _world: &mut World, editor_storage: &mut EditorStorage) {
    if !editor_storage.is_editor_open {
        return;
    }

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

                    let text_edit = ui.text_edit_singleline(&mut editor_storage.file_tree_search);

                    ui.separator();
                    if let Some(tree) = &editor_storage.file_tree {
                        if editor_storage.file_tree_search.is_empty() {
                            render_file_tree(ui, tree, 0, editor_storage.file_tree_search.clone());
                        } else {
                            let files = get_all_files(&tree.path);
                            for file in files {
                                let name = file.name.to_lowercase();
                                if name.contains(&name) {
                                    render_file_tree(
                                        ui,
                                        &file,
                                        0,
                                        editor_storage.file_tree_search.clone(),
                                    );
                                }
                            }
                        }
                    } else {
                        ui.label(
                            RichText::new("res/ not found").color(Color32::from_rgb(200, 80, 80)),
                        );
                    }

                    ui.allocate_space(ui.available_size());
                });
        });
}

fn get_all_files(path: &Path) -> Vec<FileNode> {
    let mut files: Vec<FileNode> = Vec::new();
    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            files.extend(get_all_files(&path));
        } else {
            files.push(FileNode::from_path(&path));
        }
    }
    files
}

fn render_file_tree(ui: &mut Ui, node: &FileNode, depth: usize, search: String) {
    let indent = depth as f32 * 12.0;

    let search = search.to_lowercase();
    let name = node.name.clone().to_lowercase();

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
                    render_file_tree(ui, child, depth + 1, search.clone());
                }
            });
    } else {
        if search.is_empty() || name.contains(&search) {
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
}

#[editor_ui]
pub fn input_manager_ui(
    context: &mut Context,
    world: &mut World,
    editor_storage: &mut EditorStorage,
) {
    if !editor_storage.is_editor_open {
        return;
    }

    Window::new("Input Manager")
        .default_size([400.0, 500.0])
        .show(context, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Save Input Manager").clicked() {
                    world.input_manager.serialize_input_manager().unwrap();
                }
                if ui.button("Load Input Manager").clicked() {
                    world.input_manager.deserialize_input_manager().unwrap();
                }
            });

            ui.separator();

            ui.collapsing("Add KeyBind", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut editor_storage.keybind_name);
                });

                ui.horizontal(|ui| {
                    ui.label("Key Code:");
                    egui::ComboBox::from_id_salt("keybind_key_code")
                        .selected_text(&editor_storage.keybind_key_code)
                        .show_ui(ui, |ui| {
                            for key in ALL_KEY_CODES {
                                ui.selectable_value(
                                    &mut editor_storage.keybind_key_code,
                                    key.to_string(),
                                    *key,
                                );
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Action:");
                    ui.selectable_value(
                        &mut editor_storage.keybind_action,
                        KeyAction::Press,
                        "Press",
                    );
                    ui.selectable_value(
                        &mut editor_storage.keybind_action,
                        KeyAction::Release,
                        "Release",
                    );
                    ui.selectable_value(
                        &mut editor_storage.keybind_action,
                        KeyAction::Hold,
                        "Hold",
                    );
                });

                let can_add = !editor_storage.keybind_name.is_empty()
                    && !editor_storage.keybind_key_code.is_empty();

                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.add_enabled_ui(can_add, |ui| {
                        if ui.button("Add KeyBind").clicked() {
                            if let Some(key_code) = parse_key_code(&editor_storage.keybind_key_code)
                            {
                                let bind = KeyBind::new(
                                    PhysicalKey::Code(key_code),
                                    editor_storage.keybind_action.clone(),
                                    editor_storage.keybind_name.clone(),
                                );
                                world
                                    .input_manager
                                    .keybinds
                                    .insert(editor_storage.keybind_name.clone(), bind);
                                editor_storage.keybind_name.clear();
                                editor_storage.keybind_key_code.clear();
                                editor_storage.keybind_action = KeyAction::Press;
                                editor_storage.keybind_error = None;
                            } else {
                                editor_storage.keybind_error = Some(format!(
                                    "Invalid key code: {}",
                                    editor_storage.keybind_key_code
                                ));
                            }
                        }
                    });

                    if let Some(err) = &editor_storage.keybind_error {
                        ui.colored_label(egui::Color32::RED, err);
                    }
                });
            });

            ui.separator();

            ui.collapsing("Add MouseBind", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut editor_storage.mousebind_name);
                });

                ui.horizontal(|ui| {
                    ui.label("Button:");
                    ui.selectable_value(
                        &mut editor_storage.mousebind_button,
                        MouseButton::Left,
                        "Left",
                    );
                    ui.selectable_value(
                        &mut editor_storage.mousebind_button,
                        MouseButton::Right,
                        "Right",
                    );
                    ui.selectable_value(
                        &mut editor_storage.mousebind_button,
                        MouseButton::Middle,
                        "Middle",
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Action:");
                    ui.selectable_value(
                        &mut editor_storage.mousebind_action,
                        KeyAction::Press,
                        "Press",
                    );
                    ui.selectable_value(
                        &mut editor_storage.mousebind_action,
                        KeyAction::Release,
                        "Release",
                    );
                    ui.selectable_value(
                        &mut editor_storage.mousebind_action,
                        KeyAction::Hold,
                        "Hold",
                    );
                });

                ui.add_space(4.0);

                let can_add = !editor_storage.mousebind_name.is_empty();

                ui.add_enabled_ui(can_add, |ui| {
                    if ui.button("Add MouseBind").clicked() {
                        let bind = MouseBind::new(
                            editor_storage.mousebind_button,
                            editor_storage.mousebind_action.clone(),
                            editor_storage.mousebind_name.clone(),
                        );
                        world
                            .input_manager
                            .mouse_keybinds
                            .insert(editor_storage.mousebind_name.clone(), bind);
                        editor_storage.mousebind_name.clear();
                        editor_storage.mousebind_action = KeyAction::Press;
                    }
                });
            });

            ui.separator();

            ui.collapsing(
                format!("KeyBinds ({})", world.input_manager.keybinds.len()),
                |ui| {
                    let mut to_remove: Option<String> = None;

                    egui::ScrollArea::vertical()
                        .id_salt("keybinds_scroll")
                        .max_height(150.0)
                        .show(ui, |ui| {
                            for (name, bind) in &world.input_manager.keybinds {
                                ui.horizontal(|ui| {
                                    ui.label(format!(
                                        "{}: {:?} — {:?}",
                                        name, bind.key, bind.action
                                    ));
                                    if ui.small_button("❌").clicked() {
                                        to_remove = Some(name.clone());
                                    }
                                });
                            }
                        });

                    if let Some(name) = to_remove {
                        world.input_manager.keybinds.remove(&name);
                    }
                },
            );

            ui.collapsing(
                format!("MouseBinds ({})", world.input_manager.mouse_keybinds.len()),
                |ui| {
                    let mut to_remove: Option<String> = None;

                    egui::ScrollArea::vertical()
                        .id_salt("mousebinds_scroll")
                        .max_height(150.0)
                        .show(ui, |ui| {
                            for (name, bind) in &world.input_manager.mouse_keybinds {
                                ui.horizontal(|ui| {
                                    ui.label(format!(
                                        "{}: {:?} — {:?}",
                                        name, bind.key, bind.action
                                    ));
                                    if ui.small_button("❌").clicked() {
                                        to_remove = Some(name.clone());
                                    }
                                });
                            }
                        });

                    if let Some(name) = to_remove {
                        world.input_manager.mouse_keybinds.remove(&name);
                    }
                },
            );
        });
}

pub fn parse_key_code(s: &str) -> Option<winit::keyboard::KeyCode> {
    match s {
        "KeyA" => Some(winit::keyboard::KeyCode::KeyA),
        "KeyB" => Some(winit::keyboard::KeyCode::KeyB),
        "KeyC" => Some(winit::keyboard::KeyCode::KeyC),
        "KeyD" => Some(winit::keyboard::KeyCode::KeyD),
        "KeyE" => Some(winit::keyboard::KeyCode::KeyE),
        "KeyF" => Some(winit::keyboard::KeyCode::KeyF),
        "KeyG" => Some(winit::keyboard::KeyCode::KeyG),
        "KeyH" => Some(winit::keyboard::KeyCode::KeyH),
        "KeyI" => Some(winit::keyboard::KeyCode::KeyI),
        "KeyJ" => Some(winit::keyboard::KeyCode::KeyJ),
        "KeyK" => Some(winit::keyboard::KeyCode::KeyK),
        "KeyL" => Some(winit::keyboard::KeyCode::KeyL),
        "KeyM" => Some(winit::keyboard::KeyCode::KeyM),
        "KeyN" => Some(winit::keyboard::KeyCode::KeyN),
        "KeyO" => Some(winit::keyboard::KeyCode::KeyO),
        "KeyP" => Some(winit::keyboard::KeyCode::KeyP),
        "KeyQ" => Some(winit::keyboard::KeyCode::KeyQ),
        "KeyR" => Some(winit::keyboard::KeyCode::KeyR),
        "KeyS" => Some(winit::keyboard::KeyCode::KeyS),
        "KeyT" => Some(winit::keyboard::KeyCode::KeyT),
        "KeyU" => Some(winit::keyboard::KeyCode::KeyU),
        "KeyV" => Some(winit::keyboard::KeyCode::KeyV),
        "KeyW" => Some(winit::keyboard::KeyCode::KeyW),
        "KeyX" => Some(winit::keyboard::KeyCode::KeyX),
        "KeyY" => Some(winit::keyboard::KeyCode::KeyY),
        "KeyZ" => Some(winit::keyboard::KeyCode::KeyZ),
        "Digit0" => Some(winit::keyboard::KeyCode::Digit0),
        "Digit1" => Some(winit::keyboard::KeyCode::Digit1),
        "Digit2" => Some(winit::keyboard::KeyCode::Digit2),
        "Digit3" => Some(winit::keyboard::KeyCode::Digit3),
        "Digit4" => Some(winit::keyboard::KeyCode::Digit4),
        "Digit5" => Some(winit::keyboard::KeyCode::Digit5),
        "Digit6" => Some(winit::keyboard::KeyCode::Digit6),
        "Digit7" => Some(winit::keyboard::KeyCode::Digit7),
        "Digit8" => Some(winit::keyboard::KeyCode::Digit8),
        "Digit9" => Some(winit::keyboard::KeyCode::Digit9),
        "Space" => Some(winit::keyboard::KeyCode::Space),
        "Enter" => Some(winit::keyboard::KeyCode::Enter),
        "Escape" => Some(winit::keyboard::KeyCode::Escape),
        "Backspace" => Some(winit::keyboard::KeyCode::Backspace),
        "Tab" => Some(winit::keyboard::KeyCode::Tab),
        "ShiftLeft" => Some(winit::keyboard::KeyCode::ShiftLeft),
        "ShiftRight" => Some(winit::keyboard::KeyCode::ShiftRight),
        "ControlLeft" => Some(winit::keyboard::KeyCode::ControlLeft),
        "ControlRight" => Some(winit::keyboard::KeyCode::ControlRight),
        "AltLeft" => Some(winit::keyboard::KeyCode::AltLeft),
        "AltRight" => Some(winit::keyboard::KeyCode::AltRight),
        "ArrowUp" => Some(winit::keyboard::KeyCode::ArrowUp),
        "ArrowDown" => Some(winit::keyboard::KeyCode::ArrowDown),
        "ArrowLeft" => Some(winit::keyboard::KeyCode::ArrowLeft),
        "ArrowRight" => Some(winit::keyboard::KeyCode::ArrowRight),
        "F1" => Some(winit::keyboard::KeyCode::F1),
        "F2" => Some(winit::keyboard::KeyCode::F2),
        "F3" => Some(winit::keyboard::KeyCode::F3),
        "F4" => Some(winit::keyboard::KeyCode::F4),
        "F5" => Some(winit::keyboard::KeyCode::F5),
        "F6" => Some(winit::keyboard::KeyCode::F6),
        "F7" => Some(winit::keyboard::KeyCode::F7),
        "F8" => Some(winit::keyboard::KeyCode::F8),
        "F9" => Some(winit::keyboard::KeyCode::F9),
        "F10" => Some(winit::keyboard::KeyCode::F10),
        "F11" => Some(winit::keyboard::KeyCode::F11),
        "F12" => Some(winit::keyboard::KeyCode::F12),
        "Delete" => Some(winit::keyboard::KeyCode::Delete),
        "Insert" => Some(winit::keyboard::KeyCode::Insert),
        "Home" => Some(winit::keyboard::KeyCode::Home),
        "End" => Some(winit::keyboard::KeyCode::End),
        "PageUp" => Some(winit::keyboard::KeyCode::PageUp),
        "PageDown" => Some(winit::keyboard::KeyCode::PageDown),
        "CapsLock" => Some(winit::keyboard::KeyCode::CapsLock),
        "Numpad0" => Some(winit::keyboard::KeyCode::Numpad0),
        "Numpad1" => Some(winit::keyboard::KeyCode::Numpad1),
        "Numpad2" => Some(winit::keyboard::KeyCode::Numpad2),
        "Numpad3" => Some(winit::keyboard::KeyCode::Numpad3),
        "Numpad4" => Some(winit::keyboard::KeyCode::Numpad4),
        "Numpad5" => Some(winit::keyboard::KeyCode::Numpad5),
        "Numpad6" => Some(winit::keyboard::KeyCode::Numpad6),
        "Numpad7" => Some(winit::keyboard::KeyCode::Numpad7),
        "Numpad8" => Some(winit::keyboard::KeyCode::Numpad8),
        "Numpad9" => Some(winit::keyboard::KeyCode::Numpad9),
        _ => None,
    }
}

const ALL_KEY_CODES: &[&str] = &[
    "KeyA",
    "KeyB",
    "KeyC",
    "KeyD",
    "KeyE",
    "KeyF",
    "KeyG",
    "KeyH",
    "KeyI",
    "KeyJ",
    "KeyK",
    "KeyL",
    "KeyM",
    "KeyN",
    "KeyO",
    "KeyP",
    "KeyQ",
    "KeyR",
    "KeyS",
    "KeyT",
    "KeyU",
    "KeyV",
    "KeyW",
    "KeyX",
    "KeyY",
    "KeyZ",
    "Digit0",
    "Digit1",
    "Digit2",
    "Digit3",
    "Digit4",
    "Digit5",
    "Digit6",
    "Digit7",
    "Digit8",
    "Digit9",
    "F1",
    "F2",
    "F3",
    "F4",
    "F5",
    "F6",
    "F7",
    "F8",
    "F9",
    "F10",
    "F11",
    "F12",
    "Space",
    "Enter",
    "Escape",
    "Backspace",
    "Tab",
    "CapsLock",
    "ShiftLeft",
    "ShiftRight",
    "ControlLeft",
    "ControlRight",
    "AltLeft",
    "AltRight",
    "ArrowUp",
    "ArrowDown",
    "ArrowLeft",
    "ArrowRight",
    "Home",
    "End",
    "PageUp",
    "PageDown",
    "Insert",
    "Delete",
    "Numpad0",
    "Numpad1",
    "Numpad2",
    "Numpad3",
    "Numpad4",
    "Numpad5",
    "Numpad6",
    "Numpad7",
    "Numpad8",
    "Numpad9",
];
