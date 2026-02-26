use crate::engine::nodes::World;
use crate::log;
use crate::log_warn;
use crate::{self as apostasy, engine::editor::EditorStorage, get_log_buffer};
use apostasy_macros::{console_command, editor_ui};
use egui::{Color32, Context, RichText, ScrollArea, Window};

pub struct ConsoleCommand {
    pub name: &'static str,
    pub func: fn(world: &mut World, editor_storage: &mut EditorStorage, inputs: Vec<String>),
}

inventory::collect!(ConsoleCommand);

#[console_command]
pub fn editor_mode(_world: &mut World, editor_storage: &mut EditorStorage, inputs: Vec<String>) {
    if inputs.len() == 1 {
        if inputs[0].to_lowercase() == "on" || inputs[0].to_lowercase() == "true" {
            editor_storage.is_editor_open = true;
        } else if inputs[0].to_lowercase() == "off" || inputs[0].to_lowercase() == "false" {
            editor_storage.is_editor_open = false;
        }
    } else {
        log_warn!("Usage: editor_mode [on/true|off/false]");
    }
}

#[editor_ui]
pub fn console_ui(context: &mut Context, world: &mut World, editor_storage: &mut EditorStorage) {
    let mut command_to_execute: Option<String> = None;
    let mut command_inputs: Vec<String> = Vec::new();

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
                let split_command: Vec<&str> = editor_storage.console_command.split(' ').collect();
                command_to_execute = Some(split_command[0].to_string());
                for input in split_command.iter().skip(1) {
                    command_inputs.push(input.to_string());
                }
                editor_storage.console_command = String::new();
                command_text_edit.request_focus();
            }

            ui.allocate_space(ui.available_size());
        });
    //
    if let Some(command_name) = command_to_execute {
        for cmd in inventory::iter::<ConsoleCommand> {
            if cmd.name.to_lowercase() == command_name.to_lowercase() {
                (cmd.func)(world, editor_storage, command_inputs);
                break;
            }
        }
    }
}
