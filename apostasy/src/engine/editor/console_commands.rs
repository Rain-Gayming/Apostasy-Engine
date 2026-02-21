use crate::engine::ecs::entity::Entity;
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
    let entity = inputs[0].parse::<usize>().unwrap();
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

    world.with_resource_mut(|editor_storage: &mut EditorStorage| {
        let new_logs: Vec<String> = get_log_buffer().lock().drain(..).collect();
        editor_storage.console_log.extend(new_logs);

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
                    .auto_shrink([false, false])
                    .id_salt("ConsoleScroll")
                    .show(ui, |ui| {
                        for line in &editor_storage.console_log {
                            // skip lines that don't match the filter
                            if !editor_storage.console_filter.is_empty()
                                && !line.contains(&editor_storage.console_filter)
                            {
                                continue;
                            }

                            let (color, text) = if line.starts_with("[ERROR]") {
                                (Color32::from_rgb(220, 80, 80), line.as_str())
                            } else if line.starts_with("[WARN]") {
                                (Color32::from_rgb(220, 180, 80), line.as_str())
                            } else {
                                (Color32::from_gray(200), line.as_str())
                            };
                            ui.label(RichText::new(text).size(11.0).color(color).monospace());
                        }

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

                            // split the command and add it to the inputs
                            for input in split_command.iter().skip(1) {
                                command_inputs.push(input.to_string());
                            }
                            // command_to_execute = Some(editor_storage.console_command.clone());
                            editor_storage.console_command = String::new();
                            command_text_edit.request_focus();
                        }

                        ui.allocate_space(ui.available_size());
                    });
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
