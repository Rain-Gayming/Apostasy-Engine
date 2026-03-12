use std::path::{Path, PathBuf};

use cgmath::Vector3;
use egui::{Button, Color32, RichText, ScrollArea, Sense, Stroke, Ui, pos2};

use crate::{
    engine::{
        editor::EditorStorage,
        nodes::{
            Node,
            components::{light::Light, transform::Transform},
            scene::{Scene, serialize_scene},
            world::World,
        },
        rendering::models::model::ModelRenderer,
    },
    log,
};

#[derive(Clone)]
pub struct FileNode {
    pub name: String,
    pub path: PathBuf,
    pub children: Vec<FileNode>,
    pub is_dir: bool,
}

impl FileNode {
    pub fn from_path(path: &Path) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let mut children = Vec::new();
        let is_dir = path.is_dir();

        if is_dir {
            if let Ok(entries) = std::fs::read_dir(path) {
                let mut entries: Vec<_> = entries.flatten().collect();
                entries.sort_by(|a, b| {
                    let a_is_dir = a.path().is_dir();
                    let b_is_dir = b.path().is_dir();
                    b_is_dir
                        .cmp(&a_is_dir)
                        .then(a.file_name().cmp(&b.file_name()))
                });
                for entry in entries {
                    children.push(FileNode::from_path(&entry.path()));
                }
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

pub fn render_file_tree_ui(ui: &mut Ui, editor_storage: &mut EditorStorage, world: &mut World) {
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

            ui.text_edit_singleline(&mut editor_storage.file_tree_search);
            ui.separator();

            let tree = editor_storage.file_tree.clone().unwrap();

            if editor_storage.file_tree_search.is_empty() {
                render_file_tree(
                    ui,
                    &tree,
                    0,
                    editor_storage.file_tree_search.clone(),
                    editor_storage,
                    world,
                );
            } else {
                let files = get_all_files(&tree.path);
                let search = editor_storage.file_tree_search.to_lowercase();
                for file in files {
                    if file.name.to_lowercase().contains(&search) {
                        render_file_tree(
                            ui,
                            &file,
                            0,
                            editor_storage.file_tree_search.clone(),
                            editor_storage,
                            world,
                        );
                    }
                }
            }

            ui.allocate_space(ui.available_size());
        });
}

fn get_all_files(path: &Path) -> Vec<FileNode> {
    let mut files: Vec<FileNode> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                files.extend(get_all_files(&p));
            } else {
                files.push(FileNode::from_path(&p));
            }
        }
    }
    files
}

fn render_file_tree(
    ui: &mut Ui,
    node: &FileNode,
    depth: usize,
    search: String,
    editor_storage: &mut EditorStorage,
    world: &mut World,
) {
    let indent = depth as f32 * 12.0;
    let search_lc = search.to_lowercase();
    let name_lc = node.name.to_lowercase();
    if !editor_storage.file_dragging && editor_storage.was_dragging_last_frame {
        editor_storage.was_dragging_last_frame = false;
        editor_storage.dragged_tree_node = None;
    }

    if node.is_dir {
        if *node.path != *"res/.engine" {
            let id = ui.make_persistent_id(&node.path);
            egui::CollapsingHeader::new(&node.name)
                .id_salt(id)
                .default_open(depth == 0)
                .icon(|ui, openness, response| {
                    let rect = response.rect;
                    let color = Color32::from_gray(180);
                    let points = if openness > 0.5 {
                        vec![
                            pos2(rect.left(), rect.top()),
                            pos2(rect.right(), rect.top()),
                            pos2(rect.center().x, rect.bottom()),
                        ]
                    } else {
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
                        render_file_tree(
                            ui,
                            child,
                            depth + 1,
                            search.clone(),
                            editor_storage,
                            world,
                        );
                    }
                });
        }
    } else if search_lc.is_empty() || name_lc.contains(&search_lc) {
        ui.horizontal(|ui| {
            ui.add_space(indent);
            let ext = node.path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let icon = match ext {
                "png" | "jpg" | "jpeg" | "webp" => "🖼",
                "glsl" | "vert" | "frag" | "spv" | "wgsl" => "🔷",
                "rs" => "🦀",
                "toml" | "json" | "yaml" | "yml" => "📄",
                "ttf" | "otf" => "🔤",
                "wav" | "mp3" | "ogg" => "🔊",
                _ => "📃",
            };

            let formatted_name = format!("{} {}", icon, node.name);
            let response =
                ui.add(Button::new(formatted_name.clone()).sense(Sense::click_and_drag()));

            if response.double_clicked() {
                if formatted_name.ends_with(".scene") {
                    log!("Open: {:?}", node.path);
                    editor_storage.scene_to_open = Some(node.path.to_str().unwrap().to_string());
                }
                // if the file is a .glb file
                // open a preview scene
                if formatted_name.ends_with(".glb") {
                    // create a new scene
                    let mut scene = Scene::new("res/.engine/model_preview.scene".to_string());

                    // create a new node
                    let mut scene_node = Node::new();

                    // add a model renderer to the scene
                    // it automatically loads the clicked model
                    let mut model_renderer = ModelRenderer::default();
                    let model_path = node.path.display().to_string()[4..].to_string();
                    model_renderer.loaded_model = model_path;
                    scene_node.add_component(model_renderer);
                    scene_node.add_component(Transform::default());

                    let mut light = Node::new();
                    light.name = "light".to_string();
                    let mut transform = Transform::default();
                    transform.position = Vector3::new(-10.0, 15.0, 0.0);
                    light.add_component(transform);
                    light.add_component(Light::default());
                    light.id = 1;
                    scene_node.add_child(light);

                    scene.root_node.add_child(scene_node);

                    let path = scene.path.clone();
                    let res = serialize_scene(scene);

                    println!("res: {:?}", res);

                    // load the scene
                    editor_storage.scene_to_open = Some(path);

                    println!("Scene loaded");
                }

                if formatted_name.ends_with(".material") {
                    // create a new scene
                    let mut scene = Scene::new("res/.engine/model_preview.scene".to_string());

                    // create a new node
                    let mut scene_node = Node::new();

                    // add a model renderer to the scene
                    // it automatically loads the clicked model
                    let mut model_renderer = ModelRenderer::default();
                    let model_path = node.path.display().to_string()[4..].to_string();
                    model_renderer.loaded_model = ".engine/sphere.glb".to_string();
                    model_renderer.material_path = model_path;
                    scene_node.add_component(model_renderer);
                    scene_node.add_component(Transform::default());

                    let mut light = Node::new();
                    light.name = "light".to_string();
                    let mut transform = Transform::default();
                    transform.position = Vector3::new(-10.0, 15.0, 0.0);
                    light.add_component(transform);
                    light.add_component(Light::default());
                    light.id = 1;
                    scene_node.add_child(light);

                    scene.root_node.add_child(scene_node);

                    let path = scene.path.clone();
                    let res = serialize_scene(scene);

                    println!("res: {:?}", res);

                    // load the scene
                    editor_storage.scene_to_open = Some(path);
                }
            }

            if response.clicked() {
                editor_storage.selected_tree_node = Some(node.path.to_str().unwrap().to_string());
                println!("selected: {}", node.path.to_str().unwrap());
            }

            if response.drag_started() {
                editor_storage.dragged_tree_node = Some(node.path.to_str().unwrap().to_string());
                editor_storage.file_dragging = true;
            } else if response.drag_stopped() {
                editor_storage.file_dragging = false;
            }

            if response.dragged() {
                egui::Tooltip::always_open(
                    ui.ctx().clone(),
                    ui.layer_id(),
                    egui::Id::new("file_drag_tooltip"),
                    response.rect,
                )
                .at_pointer()
                .show(|ui| {
                    ui.label(&formatted_name);
                });
            }
        });
    }
}

/// Implimentaiton for a button that can accept a file of a certain type
/// returns true if the file was accepted and it's file path
pub fn file_dragging_ui(
    ui: &mut Ui,
    editor_storage: &mut EditorStorage,
    button_text: String,
    extension: String,
    file_type: String,
) -> (bool, String) {
    let tool_tip_text = format!("Drag any {} file here", file_type);
    let response = ui.add(
        Button::new(button_text)
            .sense(Sense::drag())
            .sense(Sense::hover())
            .sense(Sense::click())
            .min_size(egui::Vec2::new(100.0, 25.0)),
    );

    if response.contains_pointer() {
        if let Some(tree_node) = &editor_storage.dragged_tree_node {
            if tree_node.ends_with(extension.clone().as_str()) {
                egui::Tooltip::always_open(
                    ui.ctx().clone(),
                    ui.layer_id(),
                    egui::Id::new("file_drag_tooltip_2"),
                    response.rect,
                )
                .at_pointer()
                .show(|ui| {
                    ui.label(format!("Set {}", file_type));
                });
            } else {
                egui::Tooltip::always_open(
                    ui.ctx().clone(),
                    ui.layer_id(),
                    egui::Id::new("drag_hint"),
                    response.rect,
                )
                .at_pointer()
                .show(|ui| {
                    ui.label(tool_tip_text.clone());
                });
            }
        } else {
            egui::Tooltip::always_open(
                ui.ctx().clone(),
                ui.layer_id(),
                egui::Id::new("drag_hint"),
                response.rect,
            )
            .at_pointer()
            .show(|ui| {
                ui.label(format!("Drag any {} file here", file_type));
            });
        }
    }

    let pointer_pos = ui.ctx().pointer_latest_pos();
    let is_over = pointer_pos.map_or(false, |pos| response.rect.contains(pos));
    let pointer_released = ui.input(|i| i.pointer.any_released());

    if is_over && pointer_released {
        if let Some(tree_node) = &editor_storage.dragged_tree_node {
            if tree_node.ends_with(extension.clone().as_str()) {
                let file_path = tree_node[4..].to_string();
                // split off after "res/"
                println!("path: {}", file_path);

                return (true, file_path);
            }
        }
    }
    (false, "".to_string())
}
