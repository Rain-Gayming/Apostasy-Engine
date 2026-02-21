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
        rendering::models::model::ModelRenderer,
    },
    get_log_buffer, log,
};
use apostasy_macros::{Resource, ui};
use egui::{
    Align2, CentralPanel, CollapsingHeader, Color32, Context, CursorIcon, FontFamily, FontId,
    Frame, Id, Rect, RichText, ScrollArea, Sense, Stroke, Ui, UiBuilder, pos2, vec2,
};

/// Storage for all information needed by the editor
#[derive(Resource)]
pub struct EditorStorage {
    pub selected_entity: Entity,
    pub component_text_edit: String,
    pub file_tree: Option<FileNode>,
    pub console_log: Vec<String>,
}

impl Default for EditorStorage {
    fn default() -> Self {
        Self {
            selected_entity: Entity::from_raw(0),
            component_text_edit: String::new(),
            file_tree: Some(FileNode::from_path(Path::new("res/"))),
            console_log: Vec::new(),
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
pub fn editor_ui(context: &mut Context, world: &mut World) {
    // --- Persistent layout ratios ---
    let left_ratio_id = Id::new("left_panel_ratio"); // left panel width ratio
    let split_ratio_id = Id::new("hs_split_ratio"); // hierarchy/inspector split
    let console_ratio_id = Id::new("console_ratio"); // viewport/console split
    let files_ratio_id = Id::new("files_ratio"); // files/inspector split

    // split for the left panel (files, hierarchy, inspector) vs the right panel (viewport, console)
    let mut left_ratio =
        context.data_mut(|d| *d.get_temp_mut_or_insert_with(left_ratio_id, || 0.2_f32));
    // split for the heirarchy and inspector
    let mut split_ratio =
        context.data_mut(|d| *d.get_temp_mut_or_insert_with(split_ratio_id, || 0.5_f32));
    // split for the viewport and console
    let mut console_ratio =
        context.data_mut(|d| *d.get_temp_mut_or_insert_with(console_ratio_id, || 0.7_f32));
    // split for the files
    let mut files_ratio =
        context.data_mut(|d| *d.get_temp_mut_or_insert_with(files_ratio_id, || 0.15_f32));

    const DIV: f32 = 4.0;
    const TOP_BAR_H: f32 = 24.0;

    CentralPanel::default()
        .frame(Frame::new().fill(Color32::TRANSPARENT))
        .show(context, |ui| {
            let full = ui.max_rect();

            // ── Top bar ───────────────────────────────────────────────────
            let top_bar_rect = Rect::from_min_size(full.min, vec2(full.width(), TOP_BAR_H));
            let mut top_ui = ui.new_child(UiBuilder::new().max_rect(top_bar_rect));
            top_ui
                .painter()
                .rect_filled(top_bar_rect, 0.0, Color32::from_gray(80));
            top_ui.label("Top Bar");

            let below_top = Rect::from_min_max(pos2(full.min.x, full.min.y + TOP_BAR_H), full.max);

            // ── Files sidebar ─────────────────────────────────────────────
            let files_w = below_top.width() * files_ratio;
            let files_rect = Rect::from_min_max(
                below_top.min,
                pos2(below_top.min.x + files_w, below_top.max.y),
            );
            let files_div_rect = Rect::from_min_max(
                pos2(files_rect.max.x, below_top.min.y),
                pos2(files_rect.max.x + DIV, below_top.max.y),
            );

            // Files panel with scroll
            let mut files_ui = ui.new_child(UiBuilder::new().max_rect(files_rect));
            files_ui
                .painter()
                .rect_filled(files_rect, 0.0, Color32::from_gray(45));
            files_ui.style_mut().visuals.override_text_color = Some(Color32::from_gray(210));

            // create the file tree
            world.with_resource_mut(|editor_storage: &mut EditorStorage| {
                ScrollArea::vertical()
                    .id_salt("files_scroll")
                    .show(&mut files_ui, |ui| {
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
                    });
            });

            // Files divider
            let files_div_resp = ui.allocate_rect(files_div_rect, Sense::drag());
            let files_div_color = if files_div_resp.hovered() || files_div_resp.dragged() {
                context.set_cursor_icon(CursorIcon::ResizeHorizontal);
                Color32::from_gray(180)
            } else {
                Color32::from_gray(60)
            };

            // File background
            ui.painter()
                .rect_filled(files_div_rect, 0.0, files_div_color);

            // Updating the file menu size
            if files_div_resp.dragged() {
                files_ratio = (files_ratio + files_div_resp.drag_delta().x / below_top.width())
                    .clamp(0.05, 0.3);
            }

            // ── Main area (right of Files) ────────────────────────────────
            let main_rect =
                Rect::from_min_max(pos2(files_div_rect.max.x, below_top.min.y), below_top.max);

            // Vertical divider: left panel | right area
            let left_w = main_rect.width() * left_ratio;
            let left_panel_rect = Rect::from_min_max(
                main_rect.min,
                pos2(main_rect.min.x + left_w, main_rect.max.y),
            );
            let vertical_div_rect = Rect::from_min_max(
                pos2(left_panel_rect.max.x, main_rect.min.y),
                pos2(left_panel_rect.max.x + DIV, main_rect.max.y),
            );
            let right_rect = Rect::from_min_max(
                pos2(vertical_div_rect.max.x, main_rect.min.y),
                main_rect.max,
            );

            // ── Left panel: Hierarchy + Inspector ────────────────────────
            let hierarchy_h = left_panel_rect.height() * split_ratio;
            let hierarchy_rect = Rect::from_min_max(
                left_panel_rect.min,
                pos2(left_panel_rect.max.x, left_panel_rect.min.y + hierarchy_h),
            );
            let hierarchy_div_rect = Rect::from_min_max(
                pos2(left_panel_rect.min.x, hierarchy_rect.max.y),
                pos2(left_panel_rect.max.x, hierarchy_rect.max.y + DIV),
            );
            let insp_rect = Rect::from_min_max(
                pos2(left_panel_rect.min.x, hierarchy_div_rect.max.y),
                left_panel_rect.max,
            );

            let mut hierarchy_ui = ui.new_child(UiBuilder::new().max_rect(hierarchy_rect));
            hierarchy_ui
                .painter()
                .rect_filled(hierarchy_rect, 0.0, Color32::from_rgb(0, 0, 0));
            hierarchy_ui.label("Hierarchy");
            if hierarchy_ui.button("New Entity").clicked() {
                world.spawn();
            }

            world.with_resource_mut(|editor_storage: &mut EditorStorage| {
                ScrollArea::vertical()
                    .id_salt("entities_scroll")
                    .show(&mut hierarchy_ui, |ui| {
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
                    });
            });

            let hierarchy_resp = ui.allocate_rect(hierarchy_div_rect, Sense::drag());
            let hierarchy_color = if hierarchy_resp.hovered() || hierarchy_resp.dragged() {
                context.set_cursor_icon(CursorIcon::ResizeVertical);
                Color32::from_gray(180)
            } else {
                Color32::from_gray(60)
            };
            ui.painter()
                .rect_filled(hierarchy_div_rect, 0.0, hierarchy_color);
            if hierarchy_resp.dragged() {
                split_ratio = (split_ratio
                    + hierarchy_resp.drag_delta().y / left_panel_rect.height())
                .clamp(0.1, 0.9);
            }

            let mut insp_ui = ui.new_child(UiBuilder::new().max_rect(insp_rect));
            insp_ui
                .painter()
                .rect_filled(insp_rect, 0.0, Color32::from_rgb(160, 40, 220));
            insp_ui.label("Inspector");

            world.with_resource_mut(|editor_storage: &mut EditorStorage| {
                if editor_storage.selected_entity != Entity::from_raw(0) {
                    let text_edit =
                        insp_ui.text_edit_singleline(&mut editor_storage.component_text_edit);

                    if text_edit.lost_focus() && insp_ui.input(|i| i.key_pressed(egui::Key::Enter))
                        || insp_ui.button("Add Component").clicked()
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

                    insp_ui.separator();

                    insp_ui.label("Components");

                    let entity = world.entity(editor_storage.selected_entity);

                    if let Some(mut name) = entity.get_mut::<Name>() {
                        insp_ui.label(format!("Name: {}", name.0));
                        insp_ui.text_edit_singleline(&mut name.0);
                        insp_ui.separator();
                    }

                    if let Some(mut transform) = entity.get_mut::<Transform>() {
                        insp_ui.label("TRANSFORM");
                        insp_ui.label(format!("Position: {:?}", transform.position));
                        insp_ui.add_space(4.0);
                        insp_ui.add(egui::DragValue::new(&mut transform.position.x).speed(1));
                        insp_ui.add(egui::DragValue::new(&mut transform.position.y).speed(1));
                        insp_ui.add(egui::DragValue::new(&mut transform.position.z).speed(1));
                        insp_ui.add_space(4.0);
                        insp_ui.label(format!("Rotation: {:?}", transform.rotation));
                        insp_ui.add(egui::DragValue::new(&mut transform.yaw).speed(1));
                        insp_ui.add(egui::DragValue::new(&mut transform.pitch).speed(1));
                        insp_ui.add_space(4.0);
                        calculate_rotation(&mut transform);
                        insp_ui.separator();
                    }
                    if let Some(mut model_renderer) = entity.get_mut::<ModelRenderer>() {
                        insp_ui.label("MODEL RENDERER");
                        insp_ui.label(format!("Model: {}", model_renderer.0));
                        let text_edit = insp_ui.text_edit_singleline(&mut model_renderer.1);
                        if text_edit.lost_focus()
                            && insp_ui.input(|i| i.key_pressed(egui::Key::Enter))
                            || insp_ui.button("Load Model").clicked()
                        {
                            model_renderer.0 = model_renderer.1.clone();
                        }
                        insp_ui.separator();
                    }
                }
            });
            // ── Vertical divider (left panel | right) ────────────────────
            let vertical_div_resp = ui.allocate_rect(vertical_div_rect, Sense::drag());
            let vertical_div_color = if vertical_div_resp.hovered() || vertical_div_resp.dragged() {
                context.set_cursor_icon(CursorIcon::ResizeHorizontal);
                Color32::from_gray(180)
            } else {
                Color32::from_gray(60)
            };
            ui.painter()
                .rect_filled(vertical_div_rect, 0.0, vertical_div_color);
            if vertical_div_resp.dragged() {
                left_ratio = (left_ratio + vertical_div_resp.drag_delta().x / main_rect.width())
                    .clamp(0.1, 0.5);
            }

            // ── Right area: Viewport (top) + Console (bottom) ─────────────
            let view_h = right_rect.height() * console_ratio;
            let view_rect = Rect::from_min_max(
                right_rect.min,
                pos2(right_rect.max.x, right_rect.min.y + view_h),
            );
            let cdiv_rect = Rect::from_min_max(
                pos2(right_rect.min.x, view_rect.max.y),
                pos2(right_rect.max.x, view_rect.max.y + DIV),
            );
            let console_rect =
                Rect::from_min_max(pos2(right_rect.min.x, cdiv_rect.max.y), right_rect.max);

            // Viewport
            let _view_ui = ui.new_child(UiBuilder::new().max_rect(view_rect));

            let cdiv_resp = ui.allocate_rect(cdiv_rect, Sense::drag());
            let cdiv_color = if cdiv_resp.hovered() || cdiv_resp.dragged() {
                context.set_cursor_icon(CursorIcon::ResizeVertical);
                Color32::from_gray(180)
            } else {
                Color32::from_gray(60)
            };
            ui.painter().rect_filled(cdiv_rect, 0.0, cdiv_color);
            if cdiv_resp.dragged() {
                console_ratio = (console_ratio + cdiv_resp.drag_delta().y / right_rect.height())
                    .clamp(0.2, 0.9);
            }

            // Console
            world.with_resource_mut(|editor_storage: &mut EditorStorage| {
                // get all logs
                let new_logs: Vec<String> = get_log_buffer().lock().drain(..).collect();
                editor_storage.console_log.extend(new_logs);

                let mut console_ui = ui.new_child(UiBuilder::new().max_rect(console_rect));
                console_ui
                    .painter()
                    .rect_filled(console_rect, 0.0, Color32::from_rgb(150, 50, 40));
                console_ui.label("Console");
                ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .id_salt("ConsoleScroll")
                    .show(&mut console_ui, |ui| {
                        for line in &editor_storage.console_log {
                            // Color code by prefix
                            let (color, text) = if line.starts_with("[ERROR]") {
                                (Color32::from_rgb(220, 80, 80), line.as_str())
                            } else if line.starts_with("[WARN]") {
                                (Color32::from_rgb(220, 180, 80), line.as_str())
                            } else {
                                (Color32::from_gray(200), line.as_str())
                            };
                            ui.label(RichText::new(text).size(11.0).color(color).monospace());
                        }
                    });
            });
        });

    // Persist ratios
    context.data_mut(|d| {
        d.insert_temp(left_ratio_id, left_ratio);
        d.insert_temp(split_ratio_id, split_ratio);
        d.insert_temp(console_ratio_id, console_ratio);
        d.insert_temp(files_ratio_id, files_ratio);
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
