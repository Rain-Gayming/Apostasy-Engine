use crate::{
    self as apostasy,
    engine::{
        editor::inspectable::InspectValue,
        nodes::{
            ENGINE_SCENE_SAVE_PATH, Node,
            components::{
                camera::Camera, collider::Collider, physics::Physics, player::Player,
                transform::Transform, velocity::Velocity,
            },
            scene::Scene,
        },
        rendering::models::model::ModelRenderer,
        windowing::{
            cursor_manager::CursorManager,
            input_manager::{KeyAction, KeyBind, MouseBind},
        },
    },
    log_warn,
};
use std::path::{Path, PathBuf};

use crate::engine::editor::console_commands::render_console_ui;
use crate::{engine::nodes::World, log};
use apostasy_macros::editor_ui;
use egui::{
    Align2, Color32, Context, FontFamily, FontId, RichText, ScrollArea, Sense, Stroke,
    TopBottomPanel, Ui, Vec2, Window, pos2,
};
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};
use serde::{Deserialize, Serialize};
use winit::{event::MouseButton, keyboard::PhysicalKey};

pub mod console_commands;
pub mod inspectable;
pub mod style;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum EditorTab {
    Hierarchy,
    Inspector,
    Files,
    Console,
    Viewport,
}

impl std::fmt::Display for EditorTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditorTab::Hierarchy => write!(f, "Hierarchy"),
            EditorTab::Inspector => write!(f, "Inspector"),
            EditorTab::Files => write!(f, "Files"),
            EditorTab::Console => write!(f, "Console"),
            EditorTab::Viewport => write!(f, "Viewport"),
        }
    }
}

pub struct EditorStorage {
    pub component_text_edit: String,
    pub is_editor_open: bool,

    // file tree
    pub file_tree_search: String,
    pub file_tree: Option<FileNode>,

    // console
    pub is_console_open: bool,
    pub console_log: Vec<String>,
    pub console_filter: String,
    pub console_command: String,

    // keybind editor
    pub is_keybind_editor_open: bool,
    pub keybind_name: String,
    pub keybind_key_code: String,
    pub keybind_action: KeyAction,
    pub keybind_error: Option<String>,

    // mousebind editor
    pub mousebind_name: String,
    pub mousebind_button: MouseButton,
    pub mousebind_action: KeyAction,

    // hierarchy
    pub dragging_node: Option<u64>,
    pub drag_target: Option<DragTarget>,
    pub selected_node: Option<u64>,
    pub show_globals: bool,

    // scene manager
    pub is_scene_manager_open: bool,
    pub scene_name: String,

    pub should_close: bool,

    pub dock_state: DockState<EditorTab>,
}

pub enum DragTarget {
    Parent(u64),
    Root,
}

const ENGINE_EDITOR_SAVE_PATH: &str = "res/editor.yaml";

fn default_dock_state() -> DockState<EditorTab> {
    let mut state = DockState::new(vec![EditorTab::Viewport]);

    let surface = state.main_surface_mut();

    let [_viewport, hierarchy] =
        surface.split_left(NodeIndex::root(), 0.2, vec![EditorTab::Hierarchy]);

    let [_, _inspector] = surface.split_right(NodeIndex::root(), 0.75, vec![EditorTab::Inspector]);

    surface.split_below(hierarchy, 0.6, vec![EditorTab::Files, EditorTab::Console]);

    state
}

impl Default for EditorStorage {
    fn default() -> Self {
        let dock_state = Self::load_dock_state().unwrap_or_else(default_dock_state);

        Self {
            component_text_edit: String::new(),

            file_tree_search: String::new(),
            file_tree: Some(FileNode::from_path(Path::new("res/"))),

            is_editor_open: true,

            is_console_open: false,
            console_log: Vec::new(),
            console_filter: String::new(),
            console_command: String::new(),

            is_keybind_editor_open: false,
            keybind_name: String::new(),
            keybind_key_code: String::new(),
            keybind_action: KeyAction::Press,
            keybind_error: None,

            mousebind_name: String::new(),
            mousebind_button: MouseButton::Left,
            mousebind_action: KeyAction::Press,

            dragging_node: None,
            drag_target: None,
            selected_node: None,
            show_globals: false,

            is_scene_manager_open: false,
            scene_name: String::new(),
            should_close: false,

            dock_state,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct SerializedDockState {
    dock: DockState<EditorTab>,
}

impl EditorStorage {
    pub fn save_dock_state(&self) {
        let path = format!("{}/editor.yaml", ENGINE_EDITOR_SAVE_PATH);
        if !Path::new(ENGINE_EDITOR_SAVE_PATH).exists() {
            let _ = std::fs::create_dir_all(ENGINE_EDITOR_SAVE_PATH);
        }
        let serialized = SerializedDockState {
            dock: self.dock_state.clone(),
        };
        if let Ok(yaml) = serde_yaml::to_string(&serialized) {
            let _ = std::fs::write(path, yaml);
        }
    }

    fn load_dock_state() -> Option<DockState<EditorTab>> {
        let path = format!("{}/editor.yaml", ENGINE_EDITOR_SAVE_PATH);
        let contents = std::fs::read_to_string(&path).ok()?;
        let s: SerializedDockState = serde_yaml::from_str(&contents).ok()?;
        Some(s.dock)
    }
}

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
pub struct EditorTabViewer<'a> {
    pub world: &'a mut World,
    pub editor_storage: &'a mut EditorStorage,
    pub viewport_rect: Option<egui::Rect>,
}

impl<'a> TabViewer for EditorTabViewer<'a> {
    type Tab = EditorTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.to_string().into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            EditorTab::Hierarchy => render_hierarchy(ui, self.world, self.editor_storage),
            EditorTab::Inspector => render_inspector(ui, self.world, self.editor_storage),
            EditorTab::Files => render_file_tree_ui(ui, self.editor_storage),
            EditorTab::Console => render_console_ui(ui, self.world, self.editor_storage),
            EditorTab::Viewport => {
                // The central viewport: transparent so the 3-D render shows through.
                let rect = ui.max_rect();
                self.viewport_rect = Some(rect);

                ui.painter().rect_filled(rect, 0.0, Color32::TRANSPARENT);
                ui.allocate_space(ui.available_size());
            }
        }
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn tab_style_override(
        &self,
        tab: &Self::Tab,
        global_style: &egui_dock::TabStyle,
    ) -> Option<egui_dock::TabStyle> {
        if *tab == EditorTab::Viewport {
            let mut style = global_style.clone();
            style.tab_body.bg_fill = Color32::TRANSPARENT;
            Some(style)
        } else {
            None
        }
    }
}

#[editor_ui]
pub fn render_editor(context: &mut Context, world: &mut World, editor_storage: &mut EditorStorage) {
    render_top_bar(context, world, editor_storage);

    if !editor_storage.is_editor_open {
        return;
    }
    render_scene_manager(context, world, editor_storage);
    render_input_manager(context, world, editor_storage);
    let mut dock_state = std::mem::replace(&mut editor_storage.dock_state, default_dock_state());

    let mut viewer = EditorTabViewer {
        world,
        editor_storage,
        viewport_rect: None,
    };

    DockArea::new(&mut dock_state)
        .style({
            let mut style = Style::from_egui(context.style().as_ref());

            style.main_surface_border_stroke = egui::Stroke::NONE;
            style
        })
        .show(context, &mut viewer);
    viewer.editor_storage.dock_state = dock_state;

    let viewport_rect = viewer.viewport_rect;

    if let Some(rect) = viewport_rect {
        let pointer_in_rect = context
            .pointer_latest_pos()
            .map_or(false, |pos| rect.contains(pos));
        viewer.world.is_world_hovered = pointer_in_rect && !context.wants_pointer_input();
    }
}

fn render_top_bar(context: &mut Context, world: &mut World, editor_storage: &mut EditorStorage) {
    TopBottomPanel::top("TopBar")
        .default_height(20.0)
        .show(context, |ui| {
            ui.add_space(1.0);
            ui.horizontal(|ui| {
                if ui.button("InputManager").clicked() {
                    editor_storage.is_keybind_editor_open = !editor_storage.is_keybind_editor_open;
                }
                if ui.button("SceneManager").clicked() {
                    editor_storage.is_scene_manager_open = !editor_storage.is_scene_manager_open;
                }

                if ui.button("Play").clicked() {
                    editor_storage.is_editor_open = !editor_storage.is_editor_open;
                    world.scene_manager.get_primary_scene();
                    let scene = world
                        .scene_manager
                        .load_scene(&world.scene_manager.primary_scene.clone().unwrap());
                    world.scene = scene;
                }

                if ui.button("Play Current").clicked() {
                    editor_storage.is_editor_open = !editor_storage.is_editor_open;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Close").clicked() {
                        editor_storage.should_close = true;
                    }
                });
            });
            ui.add_space(1.0);
        });
}

pub fn render_hierarchy(ui: &mut Ui, world: &mut World, editor_storage: &mut EditorStorage) {
    ui.horizontal(|ui| {
        ui.label(format!("Scene: {}", world.scene.name));
    });
    ui.horizontal(|ui| {
        if ui.button("New Entity").clicked() {
            world.add_new_node();
        }
        if ui.button("Save Scene").clicked() {
            world.serialize_scene().unwrap();
        }
    });
    ui.horizontal(|ui| {
        ui.checkbox(&mut editor_storage.show_globals, "Show Globals");
    });

    ScrollArea::vertical()
        .id_salt("hierarchy_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(4.0);

            let root_children: Vec<Node> = world.scene.root_node.children.clone();
            for node in &root_children {
                draw_node(ui, node, editor_storage, 0);
            }

            if editor_storage.show_globals {
                for node in &world.global_nodes {
                    draw_node(ui, node, editor_storage, 0);
                }
            }

            let empty_space = ui.allocate_response(ui.available_size(), Sense::hover());
            if empty_space.hovered() && editor_storage.dragging_node.is_some() {
                editor_storage.drag_target = Some(DragTarget::Root);
            }

            if ui.input(|i| i.pointer.any_released()) {
                if let Some(dragging) = editor_storage.dragging_node.take() {
                    let target = editor_storage.drag_target.take();
                    let root = &mut *world.scene.root_node;
                    match target {
                        Some(DragTarget::Parent(parent_id)) if parent_id != dragging => {
                            if let Some(node) = root.remove_node(dragging) {
                                root.insert_under(parent_id, node);
                            }
                        }
                        _ => {
                            if let Some(mut node) = root.remove_node(dragging) {
                                node.parent = None;
                                root.children.push(node);
                            }
                        }
                    }
                }
            }
            ui.allocate_space(ui.available_size());
        });
}

fn draw_node(ui: &mut egui::Ui, node: &Node, editor_storage: &mut EditorStorage, depth: usize) {
    let indent = depth as f32 * 10.0;
    let has_children = !node.children.is_empty();
    let selected = Some(node.id) == editor_storage.selected_node;
    let id = ui.make_persistent_id(format!("node_{}", node.name));

    if has_children {
        egui::CollapsingHeader::new(&node.name)
            .id_salt(id)
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
                    draw_node(ui, child, editor_storage, depth + 1);
                }
            });
    } else {
        ui.horizontal(|ui| {
            let extra = if node.parent.as_deref() == Some("root") {
                18.0
            } else {
                0.0
            };
            ui.add_space(indent + extra);
            draw_node_row(ui, node, selected, editor_storage);
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
        editor_storage.dragging_node = Some(node.id);
    }

    if editor_storage.dragging_node == Some(node.id) && response.dragged() {
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

    let pointer_pos = ui.ctx().pointer_latest_pos();
    let is_drag_target =
        editor_storage.dragging_node.is_some() && pointer_pos.is_some_and(|pos| rect.contains(pos));

    if is_drag_target {
        editor_storage.drag_target = Some(DragTarget::Parent(node.id));
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

    if is_drag_target {
        ui.painter().line_segment(
            [rect.left_bottom(), rect.right_bottom()],
            egui::Stroke::new(2.0, Color32::from_rgb(100, 200, 100)),
        );
    }

    ui.painter().rect_filled(rect, 0.0, color);
    ui.painter().text(
        rect.left_center() + Vec2::new(4.0, 0.0),
        Align2::LEFT_CENTER,
        &node.name,
        FontId::new(11.0, FontFamily::Proportional),
        Color32::WHITE,
    );

    if response.clicked() {
        editor_storage.selected_node = Some(node.id);
    }
}

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
                    let text_edit = ui.text_edit_singleline(&mut node.editing_name);
                    if text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        node.name = node.editing_name.clone();
                        editor_storage.selected_node = Some(id);
                    }
                });

                if let Some(parent) = &node.parent {
                    ui.label(format!("Parent Node: {}", parent));
                }
                ui.separator();

                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut editor_storage.component_text_edit);
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

                if let Some(c) = node.get_component_mut::<Transform>() {
                    c.inspect_value(ui);
                }
                if let Some(c) = node.get_component_mut::<Camera>() {
                    c.inspect_value(ui);
                }
                if let Some(c) = node.get_component_mut::<ModelRenderer>() {
                    c.inspect_value(ui);
                }
                if let Some(c) = node.get_component_mut::<Velocity>() {
                    c.inspect_value(ui);
                }
                if let Some(c) = node.get_component_mut::<Physics>() {
                    c.inspect_value(ui);
                }
                if let Some(c) = node.get_component_mut::<Collider>() {
                    c.inspect_value(ui);
                }
                if let Some(c) = node.get_component_mut::<CursorManager>() {
                    c.inspect_value(ui);
                }
                if let Some(c) = node.get_component_mut::<Player>() {
                    c.inspect_value(ui);
                }

                ui.allocate_space(ui.available_size());
            });
    }
}

pub fn render_file_tree_ui(ui: &mut egui::Ui, editor_storage: &mut EditorStorage) {
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

            if let Some(tree) = &editor_storage.file_tree {
                if editor_storage.file_tree_search.is_empty() {
                    render_file_tree(ui, tree, 0, editor_storage.file_tree_search.clone());
                } else {
                    let files = get_all_files(&tree.path);
                    let search = editor_storage.file_tree_search.to_lowercase();
                    for file in files {
                        if file.name.to_lowercase().contains(&search) {
                            render_file_tree(ui, &file, 0, editor_storage.file_tree_search.clone());
                        }
                    }
                }
            } else {
                ui.label(RichText::new("res/ not found").color(Color32::from_rgb(200, 80, 80)));
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

fn render_file_tree(ui: &mut Ui, node: &FileNode, depth: usize, search: String) {
    let indent = depth as f32 * 12.0;
    let search_lc = search.to_lowercase();
    let name_lc = node.name.to_lowercase();

    if node.is_dir {
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
                    render_file_tree(ui, child, depth + 1, search.clone());
                }
            });
    } else if search_lc.is_empty() || name_lc.contains(&search_lc) {
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
                log!("Open: {:?}", node.path);
            }
            label.on_hover_text(node.path.to_string_lossy());
        });
    }
}

fn render_scene_manager(
    context: &mut Context,
    world: &mut World,
    editor_storage: &mut EditorStorage,
) {
    if !editor_storage.is_scene_manager_open {
        return;
    }

    Window::new("Scene Manager")
        .default_size([400.0, 500.0])
        .show(context, |ui| {
            ui.collapsing("Add Scene", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut editor_storage.scene_name);
                });
                ui.add_space(4.0);

                let scene_path = format!(
                    "{}/{}.yaml",
                    ENGINE_SCENE_SAVE_PATH, editor_storage.scene_name
                );
                let can_add =
                    !editor_storage.scene_name.is_empty() && !Path::new(&scene_path).exists();

                ui.add_enabled_ui(can_add, |ui| {
                    if ui.button("Add Scene").clicked() {
                        let mut scene = Scene::new();
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
                        let scene_names: Vec<String> = world
                            .scene_manager
                            .scenes
                            .iter()
                            .map(|s| s.name.clone())
                            .collect();

                        for name in scene_names {
                            ui.horizontal(|ui| {
                                let mut new_name = name.clone();
                                ui.text_edit_singleline(&mut new_name);
                                ui.add_space(4.0);

                                if new_name != name {
                                    let new_path =
                                        format!("{}/{}.yaml", ENGINE_SCENE_SAVE_PATH, new_name);
                                    if !Path::new(&new_path).exists() {
                                        let old_path =
                                            format!("{}/{}.yaml", ENGINE_SCENE_SAVE_PATH, name);
                                        std::fs::rename(&old_path, &new_path).unwrap();

                                        if let Some(s) = world
                                            .scene_manager
                                            .scenes
                                            .iter_mut()
                                            .find(|s| s.name == name)
                                        {
                                            s.name = new_name.clone();
                                        }
                                        if world.scene.name == name {
                                            world.scene.name = new_name.clone();
                                        }
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
                                    world.scene = scene;
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

fn render_input_manager(
    context: &mut Context,
    world: &mut World,
    editor_storage: &mut EditorStorage,
) {
    if !editor_storage.is_keybind_editor_open {
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
                ui.add_space(4.0);

                let can_add = !editor_storage.keybind_name.is_empty()
                    && !editor_storage.keybind_key_code.is_empty();

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

                ui.add_enabled_ui(!editor_storage.mousebind_name.is_empty(), |ui| {
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
                    ScrollArea::vertical()
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
                    ScrollArea::vertical()
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

// ── Key code helpers (unchanged) ───────────────────────────────────────────────

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
