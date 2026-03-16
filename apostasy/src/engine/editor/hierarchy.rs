use egui::{Align2, Color32, FontFamily, FontId, ScrollArea, Sense, Stroke, Ui, pos2};

use crate::engine::{
    editor::{DragTarget, EditorStorage},
    nodes::{Node, scene::SceneInstance, world::World},
};

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
            world.reload_scene_instances();
            world.scene_manager.serialize_scene_manager().unwrap();
        }
    });
    ui.horizontal(|ui| {
        ui.checkbox(&mut editor_storage.show_globals, "Show Globals");
    });

    if let Some(id) = editor_storage.selected_node {
        if ui.input(|i| i.key_pressed(egui::Key::Delete)) {
            world.remove_node(id);
            editor_storage.selected_node = None;
        }
    }

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

            if ui.input(|i| i.pointer.any_released())
                && let Some(dragging) = editor_storage.dragging_node.take()
            {
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
            ui.allocate_space(ui.available_size());
        });
}
fn draw_node(ui: &mut egui::Ui, node: &Node, editor_storage: &mut EditorStorage, depth: usize) {
    let selected = Some(node.id) == editor_storage.selected_node;
    let id = ui.make_persistent_id(node.id);

    let is_collapsed_instance = node
        .get_component::<SceneInstance>()
        .is_some_and(|i| !i.unpacked);

    let has_visible_children = !node.children.is_empty() && !is_collapsed_instance;

    if has_visible_children {
        let mut state =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true);

        let _header_resp = ui.horizontal(|ui| {
            if ui.button("X").clicked() {
                editor_storage.node_to_remove = Some(node.id);
            }
            ui.add_space(depth as f32 * 10.0);

            let (toggle_rect, toggle_resp) =
                ui.allocate_exact_size(egui::Vec2::splat(16.0), Sense::click());
            if toggle_resp.clicked() {
                state.toggle(ui);
            }
            let openness = state.openness(ui.ctx());
            let color = Color32::from_gray(180);
            let points = if openness > 0.5 {
                vec![
                    pos2(toggle_rect.left(), toggle_rect.top()),
                    pos2(toggle_rect.right(), toggle_rect.top()),
                    pos2(toggle_rect.center().x, toggle_rect.bottom()),
                ]
            } else {
                vec![
                    pos2(toggle_rect.left(), toggle_rect.top()),
                    pos2(toggle_rect.right(), toggle_rect.center().y),
                    pos2(toggle_rect.left(), toggle_rect.bottom()),
                ]
            };
            ui.painter()
                .add(epaint::Shape::convex_polygon(points, color, Stroke::NONE));

            draw_node_row(ui, node, selected, editor_storage);
        });

        state.store(ui.ctx());
        if state.is_open() {
            ui.indent(id, |ui| {
                for child in &node.children {
                    draw_node(ui, child, editor_storage, depth + 1);
                }
            });
        }
    } else {
        ui.horizontal(|ui| {
            if ui.button("X").clicked() {
                editor_storage.node_to_remove = Some(node.id);
            }
            ui.add_space(depth as f32 * 10.0 + 16.0);
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
    let is_instance = node
        .get_component::<SceneInstance>()
        .is_some_and(|i| !i.unpacked);

    let desired_size = egui::Vec2::new(ui.available_width() - 5.0, 20.0);
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
    } else if is_instance && response.hovered() {
        Color32::from_rgb(50, 90, 130)
    } else if is_instance {
        Color32::from_rgb(30, 60, 90)
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

    let display_name = if is_instance {
        format!("⬡ {}", node.name)
    } else {
        node.name.clone()
    };

    ui.painter().text(
        rect.left_center() + egui::Vec2::new(4.0, 0.0),
        Align2::LEFT_CENTER,
        &display_name,
        FontId::new(11.0, FontFamily::Proportional),
        Color32::WHITE,
    );

    if response.clicked() {
        editor_storage.selected_node = Some(node.id);
    }
}
