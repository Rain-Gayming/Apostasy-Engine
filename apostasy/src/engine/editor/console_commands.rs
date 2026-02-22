use crate::engine::ecs::resources::input_manager::InputManager;
use crate::engine::ecs::resources::input_manager::is_keybind_active;
use crate::log;
use crate::log_warn;
use crate::{self as apostasy, engine::editor::EditorStorage, get_log_buffer};
use apostasy_macros::{console_command, ui};
use egui::{Color32, Context, RichText, ScrollArea, Window};

use crate::engine::ecs::World;

pub struct ConsoleCommand {
    pub name: &'static str,
    pub func: fn(&mut World, inputs: Vec<String>),
}

inventory::collect!(ConsoleCommand);

#[console_command]
pub fn spawn(world: &mut World, inputs: Vec<String>) {
    log!("Spawning entity");
    let entity = world.spawn();
    for input in inputs {
        log!("Adding component: {}", input);
        let added = world.add_default_component_by_name(entity.entity, input.as_str());
        if !added {
            log_warn!("Component ({}) not found", input);
        }
    }
}

#[console_command]
pub fn insert(world: &mut World, inputs: Vec<String>) {
    if inputs.len() < 2 {
        log_warn!("Not enough arguments, command needs 2 arguments (entity, component)");
        return;
    }

    let entity = inputs[0].clone();

    if entity.parse::<u32>().is_err() {
        log_warn!("Entity ({}) is not a number", entity);
        return;
    }

    let entity = entity.parse::<u32>().unwrap();

    let component = inputs[1].clone();

    let entities = world.get_all_entities();
    for index_entity in entities {
        if index_entity.0.index == entity as u32 {
            let added = world.add_default_component_by_name(index_entity, component.as_str());

            log!("Inserted component {} into entity {}", component, entity);

            if !added {
                log_warn!("Component ({}) not found", component);
            }

            return;
        }
    }
    log_warn!("Entity ({}) not found", entity);
}
#[ui]
pub fn console_ui(context: &mut Context, world: &mut World) {
    let mut command_to_execute: Option<String> = None;
    let mut command_inputs: Vec<String> = Vec::new();

    world.with_resources::<(EditorStorage, InputManager), _>(|(editor_storage, input_manager)| {
        if is_keybind_active(input_manager, "console_toggle") {
            editor_storage.is_console_open = !editor_storage.is_console_open;
        }
        if !editor_storage.is_console_open {
            return;
        }

        // Drain new logs once
        let new_logs: Vec<String> = get_log_buffer().lock().drain(..).collect();
        editor_storage.console_log.extend(new_logs);

        // Pre-filter into a reusable slice of references rather than allocating
        // a new Vec on every frame when filter is active
        let filtered: Vec<&String> = if editor_storage.console_filter.is_empty() {
            editor_storage.console_log.iter().collect()
        } else {
            editor_storage
                .console_log
                .iter()
                .filter(|line| line.contains(&editor_storage.console_filter))
                .collect()
        };

        let row_height = 14.0;
        let num_rows = filtered.len();

        Window::new("Console")
            .resizable(true)
            .default_size([400.0, 300.0])
            .show(context, |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut editor_storage.console_filter)
                        .hint_text("Console filter..."),
                );

                ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .id_salt("ConsoleScroll")
                    .show_rows(ui, row_height, num_rows, |ui, row_range| {
                        for i in row_range {
                            let line = filtered[i].as_str();
                            let color = if line.starts_with("[ERROR]") {
                                Color32::from_rgb(220, 80, 80)
                            } else if line.starts_with("[WARN]") {
                                Color32::from_rgb(220, 180, 80)
                            } else {
                                Color32::from_gray(200)
                            };
                            ui.label(RichText::new(line).size(11.0).color(color).monospace());
                        }
                    });

                let command_text_edit = ui.add(
                    egui::TextEdit::singleline(&mut editor_storage.console_command)
                        .hint_text("Command..."),
                );

                if command_text_edit.lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    && !editor_storage.console_command.is_empty()
                {
                    let split_command: Vec<&str> =
                        editor_storage.console_command.split(' ').collect();
                    command_to_execute = Some(split_command[0].to_string());
                    for input in split_command.iter().skip(1) {
                        command_inputs.push(input.to_string());
                    }
                    editor_storage.console_command = String::new();
                    command_text_edit.request_focus();
                }

                ui.allocate_space(ui.available_size());
            });
    });

    if let Some(command_name) = command_to_execute {
        for cmd in inventory::iter::<ConsoleCommand> {
            if cmd.name.to_lowercase() == command_name.to_lowercase() {
                (cmd.func)(world, command_inputs);
                break;
            }
        }
    }
}
