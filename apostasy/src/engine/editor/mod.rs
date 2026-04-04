use crate::{
    self as apostasy,
    engine::{
        assets::server::AssetServer,
        editor::{
            asset_editor::asset_editor,
            engine_settings_ui::render_engine_settings_ui,
            file_manager::{FileNode, render_file_tree_ui},
            hierarchy::render_hierarchy,
            inspector::render_inspector,
            terrain_editor::{TerrainEditorSettings, render_terrain_edtor},
        },
        nodes::{Node, components::transform::Transform, scene::SceneInstance},
        rendering::{
            models::model::ModelRenderer, pipeline_settings::PipelineSettings,
            profiler::ProfilerState,
        },
        windowing::input_manager::KeyAction,
    },
    utils::screen_to_world::screen_to_world_plane,
};
use std::{
    fs::{read_to_string, write},
    path::Path,
    sync::{Arc, RwLock},
};

use crate::engine::editor::console_commands::render_console_ui;
use crate::engine::nodes::world::World;
use apostasy_macros::editor_ui;
use egui::{Color32, Context, Popup, PopupCloseBehavior, TopBottomPanel, Ui, Window};
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};
use serde::{Deserialize, Serialize};
use winit::event::MouseButton;

pub mod asset_editor;
pub mod console_commands;
pub mod engine_settings_ui;
pub mod file_manager;
pub mod hierarchy;
pub mod input_manager_ui;
pub mod inspectable;
pub mod inspector;
pub mod renderer_settings;
pub mod scene_manager_ui;
pub mod style;
pub mod terrain_editor;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
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

impl EditorTab {
    pub const fn panel_tabs() -> [Self; 5] {
        [
            Self::Hierarchy,
            Self::Inspector,
            Self::Files,
            Self::Console,
            Self::AssetEditor,
        ]
    }

    pub fn is_visible_tab(&self) -> bool {
        *self != Self::Viewport
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
    pub scene_name: String,
    pub last_scene_name: String,
    pub scene_to_add: Option<String>,

    pub should_close: bool,

    pub dock_state: DockState<EditorTab>,
    pub layout_serialized: Option<String>,
    // sizes for floating windows
    pub scene_manager_window_size: Option<[f32; 2]>,
    pub input_manager_window_size: Option<[f32; 2]>,
    pub profiler: ProfilerState,
    pub asset_server: Arc<RwLock<AssetServer>>,

    pub viewport_drag_preview_id: Option<u64>,
    pub viewport_drag_model: Option<String>,

    pub is_engine_settings_open: bool,
    pub open_engine_settings_tab: EngineSettingsTab,

    pub is_terrain_editor_open: bool,
    pub terrain_editor_settings: TerrainEditorSettings,

    pub should_update_renderer: bool,
    pub pipeline_settings: PipelineSettings,

    pub is_panel_manager_open: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct EditorLayout {
    dock_state: DockState<EditorTab>,
    scene_manager_window_size: Option<[f32; 2]>,
    input_manager_window_size: Option<[f32; 2]>,
}

pub enum DragTarget {
    Parent(u64),
    Root,
}

pub enum EngineSettingsTab {
    Inputs,
    Scenes,

    Renderer,
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

fn is_tab_open(dock_state: &DockState<EditorTab>, tab: EditorTab) -> bool {
    dock_state.iter_surfaces().any(|surface| {
        surface
            .iter_all_tabs()
            .any(|(_, existing)| *existing == tab)
    })
}

fn add_tab_if_missing(dock_state: &mut DockState<EditorTab>, tab: EditorTab) {
    if !is_tab_open(dock_state, tab) {
        dock_state.push_to_first_leaf(tab);
    }
}

fn remove_tab(dock_state: &mut DockState<EditorTab>, tab: EditorTab) {
    for surface in dock_state.iter_surfaces_mut() {
        surface.retain_tabs(|existing| *existing != tab);
    }
}

fn reset_editor_layout(editor_storage: &mut EditorStorage) {
    editor_storage.dock_state = default_dock_state();
    editor_storage.layout_serialized = None;
}

impl EditorStorage {
    pub fn default(
        asset_server: Arc<RwLock<AssetServer>>,
        _world: Arc<RwLock<World>>,
        pipeline_settings: PipelineSettings,
    ) -> Self {
        // Attempt to load a previously saved editor layout (dock + window sizes).
        let (dock_state, scene_win_size, input_win_size, layout_serialized) =
            match read_to_string("res/.engine/editor_layout.yaml") {
                Ok(contents) => match serde_yaml::from_str::<EditorLayout>(&contents) {
                    Ok(layout) => (
                        layout.dock_state,
                        layout.scene_manager_window_size,
                        layout.input_manager_window_size,
                        Some(contents),
                    ),
                    Err(_) => (default_dock_state(), None, None, None),
                },
                Err(_) => (default_dock_state(), None, None, None),
            };

        let pipeline_settings: PipelineSettings =
            match read_to_string("res/.engine/pipeline_settings.yaml") {
                Ok(contents) => match serde_yaml::from_str::<PipelineSettings>(&contents) {
                    Ok(pipeline_settings) => pipeline_settings,
                    Err(_) => pipeline_settings,
                },
                Err(_) => pipeline_settings,
            };

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

            scene_name: String::new(),
            last_scene_name: String::new(),
            should_close: false,
            scene_to_add: None,

            dock_state,
            layout_serialized,
            scene_manager_window_size: scene_win_size,
            input_manager_window_size: input_win_size,
            profiler: ProfilerState::default(),
            asset_server,
            viewport_drag_preview_id: None,
            viewport_drag_model: None,

            is_engine_settings_open: false,
            open_engine_settings_tab: EngineSettingsTab::Inputs,

            is_terrain_editor_open: false,
            terrain_editor_settings: TerrainEditorSettings::default(),

            should_update_renderer: true,
            pipeline_settings,
            is_panel_manager_open: false,
        }
    }

    pub fn save_layout(&self) {
        let layout = EditorLayout {
            dock_state: self.dock_state.clone(),
            scene_manager_window_size: self.scene_manager_window_size,
            input_manager_window_size: self.input_manager_window_size,
        };
        if let Ok(s) = serde_yaml::to_string(&layout) {
            let _ = write("res/.engine/editor_layout.yaml", s);
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

    render_engine_settings_ui(context, world, editor_storage);
    render_terrain_edtor(context, world, editor_storage);
    render_editor_panel_manager(context, editor_storage);

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
        // Persist combined layout (dock state + floating window sizes) when it changes.
        let layout = EditorLayout {
            dock_state: viewer.editor_storage.dock_state.clone(),
            scene_manager_window_size: viewer.editor_storage.scene_manager_window_size,
            input_manager_window_size: viewer.editor_storage.input_manager_window_size,
        };
        let new_serialized = serde_yaml::to_string(&layout).ok();
        if new_serialized.is_some() && new_serialized != viewer.editor_storage.layout_serialized {
            if let Some(ref s) = new_serialized {
                let _ = write("res/.engine/editor_layout.yaml", s);
            }
            viewer.editor_storage.layout_serialized = new_serialized.clone();
        }

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

        // Commit or cancel preview drag on mouse release
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
                    } else if let Some(scene_instance) = world
                        .get_node_mut(preview_id)
                        .get_component_mut::<SceneInstance>()
                    {
                        name = std::path::Path::new(&scene_instance.source_path)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("SceneInstance")
                            .to_string();
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
                let response = ui.button("Engine");
                Popup::menu(&response)
                    .close_behavior(PopupCloseBehavior::CloseOnClick)
                    .show(|ui| {
                        if ui.button("Engine Settings").clicked() {
                            editor_storage.is_engine_settings_open =
                                !editor_storage.is_engine_settings_open;
                        }

                        ui.menu_button("View", |ui| {
                            if ui.button("Editor Panels").clicked() {
                                editor_storage.is_panel_manager_open =
                                    !editor_storage.is_panel_manager_open;
                            }
                            if ui.button("Reset Layout").clicked() {
                                reset_editor_layout(editor_storage);
                            }
                        });
                    });
                let response = ui.button("Tools");
                Popup::menu(&response)
                    .close_behavior(PopupCloseBehavior::CloseOnClick)
                    .show(|ui| {
                        if ui.button("Terrain Editor").clicked() {
                            editor_storage.is_terrain_editor_open =
                                !editor_storage.is_terrain_editor_open;
                        }
                    });

                if ui.button("Play").clicked() {
                    if editor_storage.is_editor_open {
                        let _ = world.serialize_scene();
                        world.start();
                    }

                    world.scene_manager.get_primary_scene();

                    let scene_name = world
                        .scene_manager
                        .primary_scene
                        .clone()
                        .unwrap_or_else(|| world.scene.path.clone());

                    world.scene = world.scene_manager.load_scene(&scene_name).unwrap();
                    world.check_node_ids();
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
fn render_editor_panel_manager(context: &mut Context, editor_storage: &mut EditorStorage) {
    if !editor_storage.is_panel_manager_open {
        return;
    }

    Window::new("Editor Panels")
        .default_size((280.0, 240.0))
        .resizable(true)
        .show(context, |ui| {
            ui.label("Toggle editor panels and restore missing tabs.");
            ui.separator();

            for tab in EditorTab::panel_tabs() {
                let mut is_open = is_tab_open(&editor_storage.dock_state, tab);
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut is_open, tab.to_string()).changed() {
                        if is_open {
                            add_tab_if_missing(&mut editor_storage.dock_state, tab);
                        } else {
                            remove_tab(&mut editor_storage.dock_state, tab);
                        }
                    }
                });
            }

            ui.separator();
            if ui.button("Restore Default Layout").clicked() {
                reset_editor_layout(editor_storage);
            }
        });
}
