use crate::{
    self as apostasy,
    engine::{
        ecs::{World, entity::Entity},
        editor,
    },
};
use apostasy_macros::{Resource, ui};
use egui::{
    CentralPanel, Color32, Context, CursorIcon, Frame, Id, Rect, Sense, UiBuilder, pos2, vec2,
};

#[derive(Resource)]
pub struct EditorStorage {
    pub selected_entity: Entity,
    pub component_text_edit: String,
}

impl Default for EditorStorage {
    fn default() -> Self {
        Self {
            selected_entity: Entity::from_raw(0),
            component_text_edit: String::new(),
        }
    }
}

#[ui]
pub fn editor_ui(context: &mut Context, world: &mut World) {
    world.with_resource_mut(|editor_storage: &mut EditorStorage| {
        // --- Persistent layout ratios ---
        let left_ratio_id = Id::new("left_panel_ratio"); // left panel width ratio
        let split_ratio_id = Id::new("hs_split_ratio"); // hierarchy/inspector split
        let console_ratio_id = Id::new("console_ratio"); // viewport/console split
        let files_ratio_id = Id::new("files_ratio"); // files/inspector split

        let mut left_ratio =
            context.data_mut(|d| *d.get_temp_mut_or_insert_with(left_ratio_id, || 0.2_f32));
        let mut split_ratio =
            context.data_mut(|d| *d.get_temp_mut_or_insert_with(split_ratio_id, || 0.4_f32));
        let mut console_ratio =
            context.data_mut(|d| *d.get_temp_mut_or_insert_with(console_ratio_id, || 0.7_f32));
        let mut files_ratio =
            context.data_mut(|d| *d.get_temp_mut_or_insert_with(files_ratio_id, || 0.06_f32));

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

                let below_top =
                    Rect::from_min_max(pos2(full.min.x, full.min.y + TOP_BAR_H), full.max);

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

                let mut files_ui = ui.new_child(UiBuilder::new().max_rect(files_rect));
                files_ui
                    .painter()
                    .rect_filled(files_rect, 0.0, Color32::from_gray(110));
                files_ui.label("Files");

                // Files divider
                let files_div_resp = ui.allocate_rect(files_div_rect, Sense::drag());
                let files_div_color = if files_div_resp.hovered() || files_div_resp.dragged() {
                    context.set_cursor_icon(CursorIcon::ResizeHorizontal);
                    Color32::from_gray(180)
                } else {
                    Color32::from_gray(60)
                };
                ui.painter()
                    .rect_filled(files_div_rect, 0.0, files_div_color);
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

                // get all entity locations
                let entity_locations = world
                    .crust
                    .mantle(|mantle| mantle.core.entity_index.lock().clone());

                // iterate over all entities and add buttons for them
                for entity in world.get_all_entities() {
                    if hierarchy_ui
                        .button(format!("{:?}", entity.0.index))
                        .clicked()
                    {
                        let entity_location = entity_locations.get(entity).unwrap();
                        println!("{}", world.get_component_info(entity_location.to_owned()));
                        editor_storage.selected_entity = entity;
                    }
                }

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

                insp_ui.text_edit_singleline(&mut editor_storage.component_text_edit);

                if insp_ui.button("Add Component").clicked() {
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

                // ── Vertical divider (left panel | right) ────────────────────
                let vertical_div_resp = ui.allocate_rect(vertical_div_rect, Sense::drag());
                let vertical_div_color =
                    if vertical_div_resp.hovered() || vertical_div_resp.dragged() {
                        context.set_cursor_icon(CursorIcon::ResizeHorizontal);
                        Color32::from_gray(180)
                    } else {
                        Color32::from_gray(60)
                    };
                ui.painter()
                    .rect_filled(vertical_div_rect, 0.0, vertical_div_color);
                if vertical_div_resp.dragged() {
                    left_ratio = (left_ratio
                        + vertical_div_resp.drag_delta().x / main_rect.width())
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
                    console_ratio = (console_ratio
                        + cdiv_resp.drag_delta().y / right_rect.height())
                    .clamp(0.2, 0.9);
                }

                // Console
                let mut console_ui = ui.new_child(UiBuilder::new().max_rect(console_rect));
                console_ui
                    .painter()
                    .rect_filled(console_rect, 0.0, Color32::from_rgb(150, 50, 40));
                console_ui.label("Console");
            });

        // Persist ratios
        context.data_mut(|d| {
            d.insert_temp(left_ratio_id, left_ratio);
            d.insert_temp(split_ratio_id, split_ratio);
            d.insert_temp(console_ratio_id, console_ratio);
            d.insert_temp(files_ratio_id, files_ratio);
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
