use egui::{Context, ScrollArea, Window};
use winit::{event::MouseButton, keyboard::PhysicalKey};

use crate::engine::{
    editor::EditorStorage,
    nodes::world::World,
    windowing::input_manager::{KeyAction, KeyBind, MouseBind},
};

pub fn render_input_manager(
    context: &mut Context,
    world: &mut World,
    editor_storage: &mut EditorStorage,
) {
    if !editor_storage.is_keybind_editor_open {
        return;
    }

    let default_size = if let Some(r) = editor_storage.input_manager_window_size {
        if world.window_size.x > 0.0 && world.window_size.y > 0.0 {
            [r[0] * world.window_size.x, r[1] * world.window_size.y]
        } else {
            [400.0, 500.0]
        }
    } else {
        [400.0, 500.0]
    };

    Window::new("Input Manager")
        .default_size(default_size)
        .show(context, |ui| {
            // store the current window content size as a ratio of the main window
            let size = ui.available_size();
            if world.window_size.x > 0.0 && world.window_size.y > 0.0 {
                editor_storage.input_manager_window_size = Some([
                    size.x / world.window_size.x,
                    size.y / world.window_size.y,
                ]);
            }
            ui.horizontal(|ui| {
                if ui.button("Save Input Manager").clicked() {
                    world.input_manager.serialize_input_manager().unwrap();
                }
                if ui.button("Load Input Manager").clicked() {
                    world.input_manager.deserialize_input_manager().unwrap();
                }
                if ui.button("Close").clicked() {
                    editor_storage.is_keybind_editor_open = false;
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
