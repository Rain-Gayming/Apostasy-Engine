use crate::engine::editor::WindowPosition;
use crate::engine::nodes::World;
use crate::log;
use crate::log_warn;
use crate::{self as apostasy, engine::editor::EditorStorage, get_log_buffer};
use apostasy_macros::{console_command, editor_ui};
use egui::SidePanel;
use egui::TopBottomPanel;
use egui::Vec2;
use egui::{Color32, Context, RichText, ScrollArea, Window};

pub struct ConsoleCommand {
    pub name: &'static str,
    pub func: fn(world: &mut World, editor_storage: &mut EditorStorage, inputs: Vec<String>),
    pub inputs: &'static str,
}

inventory::collect!(ConsoleCommand);

#[console_command(inputs = "[on/true|off/false]")]
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

#[console_command]
pub fn help(_world: &mut World, editor_storage: &mut EditorStorage, inputs: Vec<String>) {
    if inputs.is_empty() {
        log!("Commands:");
        for cmd in inventory::iter::<ConsoleCommand> {
            log!("{} {}", cmd.name, cmd.inputs);
        }
    } else {
        for cmd in inventory::iter::<ConsoleCommand> {
            if cmd.name.to_lowercase() == inputs[0].to_lowercase() {
                log!("{} {}", cmd.name, cmd.inputs);
                break;
            }
        }
    }
}

#[editor_ui(priority = 1)]
pub fn console_ui(context: &mut Context, world: &mut World, editor_storage: &mut EditorStorage) {
    let mut command_to_execute: Option<String> = None;
    let mut command_inputs: Vec<String> = Vec::new();

    // Drain new logs once
    let new_logs: Vec<String> = get_log_buffer().lock().drain(..).collect();
    editor_storage.console_log.extend(new_logs);

    match editor_storage.console_position {
        WindowPosition::Floating => {
            Window::new("Console")
                .default_size([100.0, 100.0])
                .show(context, |ui| {
                    render_console_ui(
                        ui,
                        editor_storage,
                        &mut command_to_execute,
                        &mut command_inputs,
                    );
                });
        }
        WindowPosition::Left => {
            SidePanel::left("Console").show(context, |ui| {
                render_console_ui(
                    ui,
                    editor_storage,
                    &mut command_to_execute,
                    &mut command_inputs,
                );
            });
        }
        WindowPosition::Right => {
            SidePanel::right("Console").show(context, |ui| {
                render_console_ui(
                    ui,
                    editor_storage,
                    &mut command_to_execute,
                    &mut command_inputs,
                );
            });
        }
        WindowPosition::Top => {
            TopBottomPanel::top("Console")
                .default_height(200.0)
                .resizable(true)
                .show(context, |ui| {
                    render_console_ui(
                        ui,
                        editor_storage,
                        &mut command_to_execute,
                        &mut command_inputs,
                    );
                });
        }
        WindowPosition::Bottom => {
            let panel = TopBottomPanel::bottom("Console")
                .resizable(true)
                .min_height(64.0)
                .show(context, |ui| {
                    render_console_ui(
                        ui,
                        editor_storage,
                        &mut command_to_execute,
                        &mut command_inputs,
                    );
                });

            editor_storage.console_size.y = panel.response.rect.height();
        }
    }

    if let Some(command_name) = command_to_execute {
        for cmd in inventory::iter::<ConsoleCommand> {
            if cmd.name.to_lowercase() == command_name.to_lowercase() {
                (cmd.func)(world, editor_storage, command_inputs);
                return;
            }
        }
        log_warn!("Command not found: {}", command_name);
    }
}

pub fn render_console_ui(
    ui: &mut egui::Ui,
    editor_storage: &mut EditorStorage,
    mut command_to_execute: &mut Option<String>,
    mut command_inputs: &mut Vec<String>,
) {
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
        egui::TextEdit::singleline(&mut editor_storage.console_command).hint_text("Command..."),
    );

    if command_text_edit.lost_focus()
        && ui.input(|i| i.key_pressed(egui::Key::Enter))
        && !editor_storage.console_command.is_empty()
    {
        log!("> {}", editor_storage.console_command);
        let split_command: Vec<&str> = editor_storage.console_command.split(' ').collect();
        *command_to_execute = Some(split_command[0].to_string());
        for input in split_command.iter().skip(1) {
            command_inputs.push(input.to_string());
        }
        editor_storage.console_command = String::new();
        command_text_edit.request_focus();
    }

    ui.allocate_space(ui.available_size());
}
