use crate::{
    self as apostasy,
    engine::{
        assets::server::AssetServer,
        editor::{
            asset_editor::asset_editor,
            file_manager::{FileNode, render_file_tree_ui},
            hierarchy::render_hierarchy,
            input_manager_ui::render_input_manager,
            inspector::render_inspector,
            scene_manager_ui::render_scene_manager,
        },
        nodes::{Node, components::transform::Transform, scene::SceneInstance},
        rendering::{models::model::ModelRenderer, profiler::ProfilerState},
        windowing::input_manager::KeyAction,
    },
    utils::screen_to_world::screen_to_world_plane,
};
use std::{
    path::Path,
    sync::{Arc, RwLock},
};

use crate::engine::editor::console_commands::render_console_ui;
use crate::engine::nodes::world::World;
use apostasy_macros::editor_ui;
use egui::{Color32, Context, TopBottomPanel, Ui};
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};
use serde::{Deserialize, Serialize};
use winit::event::MouseButton;

pub mod asset_editor;
pub mod console_commands;
pub mod file_manager;
pub mod hierarchy;
pub mod input_manager_ui;
pub mod inspectable;
pub mod inspector;
pub mod scene_manager_ui;
pub mod style;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum EditorTab {
    Hierarchy,
    Inspector,
    Files,
    Console,
    Viewport,
    AssetEditor,
}

impl std::fmt::Display for EditorTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditorTab::Hierarchy => write!(f, "Hierarchy"),
            EditorTab::Inspector => write!(f, "Inspector"),
            EditorTab::Files => write!(f, "Files"),
            EditorTab::Console => write!(f, "Console"),
            EditorTab::Viewport => write!(f, "Viewport"),
            EditorTab::AssetEditor => write!(f, "Asset Editor"),
        }
    }
}

pub struct EditorStorage {
    pub component_text_edit: String,
    pub is_editor_open: bool,

    pub was_dragging_last_frame: bool,
    // file tree
    pub files: Vec<FileNode>,
    pub file_tree_search: String,
    pub file_tree: Option<FileNode>,
    pub dragged_tree_node: Option<String>,
    pub selected_tree_node: Option<String>,
    pub file_dragging: bool,
    pub scene_to_open: Option<String>,

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
    pub node_to_remove: Option<u64>,

    // scene manager
    pub is_scene_manager_open: bool,
    pub scene_name: String,
    pub last_scene_name: String,
    pub scene_to_add: Option<String>,

    pub should_close: bool,

    pub dock_state: DockState<EditorTab>,
    pub profiler: ProfilerState,
    pub asset_server: Arc<RwLock<AssetServer>>,

    pub viewport_drag_preview_id: Option<u64>,
    pub viewport_drag_model: Option<String>,
}

pub enum DragTarget {
    Parent(u64),
    Root,
}

fn default_dock_state() -> DockState<EditorTab> {
    let mut state = DockState::new(vec![EditorTab::Viewport]);

    let surface = state.main_surface_mut();

    let [_viewport, _hierarchy] =
        surface.split_left(NodeIndex::root(), 0.2, vec![EditorTab::Hierarchy]);

    let [_, _inspector] = surface.split_right(NodeIndex::root(), 0.75, vec![EditorTab::Inspector]);

    let [_console, _file_tree] =
        surface.split_below(_hierarchy, 0.6, vec![EditorTab::Files, EditorTab::Console]);
    surface.split_below(_inspector, 0.6, vec![EditorTab::AssetEditor]);

    state
}

impl EditorStorage {
    pub fn default(asset_server: Arc<RwLock<AssetServer>>, _world: Arc<RwLock<World>>) -> Self {
        Self {
            component_text_edit: String::new(),

            files: Vec::new(),
            file_tree_search: String::new(),
            file_tree: Some(FileNode::from_path(Path::new("res/"))),
            dragged_tree_node: None,
            selected_tree_node: None,
            file_dragging: false,
            was_dragging_last_frame: false,
            scene_to_open: None,

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
            node_to_remove: None,

            is_scene_manager_open: false,
            scene_name: String::new(),
            last_scene_name: String::new(),
            should_close: false,
            scene_to_add: None,

            dock_state: default_dock_state(),
            profiler: ProfilerState::default(),
            asset_server,
            viewport_drag_preview_id: None,
            viewport_drag_model: None,
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
            EditorTab::AssetEditor => asset_editor(ui, self.world, self.editor_storage),
            EditorTab::Files => render_file_tree_ui(ui, self.editor_storage, self.world),
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
    if let Some(scene_path) = &editor_storage.scene_to_open {
        let scene = world.scene_manager.load_scene(scene_path);
        world.scene = scene.unwrap();
        editor_storage.scene_to_open = None;
    }

    render_top_bar(context, world, editor_storage);

    if !editor_storage.is_editor_open {
        return;
    }
    render_scene_manager(context, world, editor_storage);
    render_input_manager(context, world, editor_storage);

    let mut dock_state = std::mem::replace(&mut editor_storage.dock_state, default_dock_state());

    let viewport_rect = {
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
        viewer.viewport_rect
    };

    if let Some(rect) = viewport_rect {
        let pointer_pos = context.pointer_latest_pos();
        let is_over_viewport = pointer_pos.map_or(false, |p| rect.contains(p));

        let is_dragging_glb = editor_storage
            .dragged_tree_node
            .as_deref()
            .map_or(false, |p| p.ends_with(".glb"));

        let is_dragging_scene = editor_storage
            .dragged_tree_node
            .as_deref()
            .map_or(false, |p| p.ends_with(".scene"));

        // Spawn preview node when glb enters viewport
        if is_over_viewport
            && editor_storage.viewport_drag_preview_id.is_none()
            && editor_storage.dragged_tree_node.is_some()
        {
            let path = editor_storage.dragged_tree_node.clone().unwrap();

            // create the preview node
            let mut node = Node::new();
            node.name = "__viewport_drag_preview__".to_string();
            node.exempt_from_id_check = true;
            node.id = u64::MAX;
            node.add_component(Transform::default());

            if is_dragging_glb {
                let model_path = path[4..].to_string(); // strip "res/"

                let mut model_renderer = ModelRenderer::default();
                model_renderer.loaded_model = model_path.clone();
                node.add_component(model_renderer);

                editor_storage.viewport_drag_model = Some(model_path);
            } else if is_dragging_scene {
                let scene_path = path.clone(); // strip "res/"

                let scene_preview = SceneInstance::new(scene_path);
                node.add_component(scene_preview);

                world.reload_scene_instances();
            }

            // add and store the preview node
            world.add_node(node);

            let id = Some(u64::MAX);
            editor_storage.viewport_drag_preview_id = id;
        }

        // Update preview position while dragging over viewport
        if let (Some(preview_id), Some(pos)) =
            (editor_storage.viewport_drag_preview_id, pointer_pos)
        {
            if is_over_viewport {
                let world_pos = screen_to_world_plane(pos, rect, world, context);
                if let Some(transform) = world
                    .get_node_mut(preview_id)
                    .get_component_mut::<Transform>()
                {
                    transform.position = world_pos;
                } else {
                    let node = world.get_node_mut(preview_id);
                    node.add_component(Transform::default());
                }
            }
        }

        // Commit or cancel on mouse release
        if context.input(|i| i.pointer.any_released()) {
            if let Some(preview_id) = editor_storage.viewport_drag_preview_id.take() {
                if is_over_viewport {
                    let mut name = "Name".to_string();
                    if let Some(_model) = world
                        .get_node_mut(preview_id)
                        .get_component_mut::<ModelRenderer>()
                    {
                        let model = editor_storage
                            .viewport_drag_model
                            .take()
                            .unwrap_or_default();
                        name = std::path::Path::new(&model)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Model")
                            .to_string();
                    } else if let Some(_scene_instance) = world
                        .get_node_mut(preview_id)
                        .get_component_mut::<SceneInstance>()
                    {
                        let node = world.get_node_mut(preview_id);
                        let scene = node.get_component_mut::<SceneInstance>();
                        if let Some(scene) = scene {
                            name = std::path::Path::new(&scene.source_path)
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("SceneInstance")
                                .to_string();
                        }
                    }

                    world.get_node_mut(preview_id).name = name;
                    world.get_node_mut(preview_id).exempt_from_id_check = false;
                    editor_storage.dragged_tree_node = None;
                    editor_storage.file_dragging = false;
                    editor_storage.viewport_drag_preview_id = None;
                    world.check_node_ids();
                } else {
                    world.remove_node(preview_id);
                    editor_storage.viewport_drag_model = None;
                    editor_storage.viewport_drag_preview_id = None;
                    println!("Removing node");
                }
            }
        }

        world.is_world_hovered = pointer_pos.map_or(false, |pos| rect.contains(pos));
    }

    if let Some(id) = editor_storage.node_to_remove {
        world.remove_node(id);
        editor_storage.node_to_remove = None;
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
                    if editor_storage.is_editor_open {
                        println!("Playing");
                        // ignore the result, errors are logged inside if needed
                        let _ = world.serialize_scene();
                        world.scene_manager.get_primary_scene();
                        let scene = world
                            .scene_manager
                            .load_scene(&world.scene_manager.primary_scene.clone().unwrap());
                        world.scene = scene.unwrap();
                        world.check_node_ids();
                    } else {
                        world.scene_manager.get_primary_scene();
                        let scene = world
                            .scene_manager
                            .load_scene(&world.scene_manager.primary_scene.clone().unwrap());
                        world.scene = scene.unwrap();
                        world.check_node_ids();
                    }
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
