use crate::engine::nodes::World;
use crate::log;
use crate::log_warn;
use crate::{self as apostasy, engine::editor::EditorStorage, get_log_buffer};
use apostasy_macros::console_command;
use egui::{Color32, RichText, ScrollArea};

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
pub fn help(_world: &mut World, _editor_storage: &mut EditorStorage, inputs: Vec<String>) {
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

/// Called by `EditorTabViewer` when the Console tab is active.
/// Returns a pending command (name + args) to execute after the borrow ends.
pub fn render_console_ui(ui: &mut egui::Ui, world: &mut World, editor_storage: &mut EditorStorage) {
    // Drain new log lines each frame.
    let new_logs: Vec<String> = get_log_buffer().lock().drain(..).collect();
    editor_storage.console_log.extend(new_logs);

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

        let raw = editor_storage.console_command.clone();
        let mut parts = raw.splitn(2, ' ');
        let cmd_name = parts.next().unwrap_or("").to_string();
        let arg_str = parts.next().unwrap_or("");
        let inputs: Vec<String> = if arg_str.is_empty() {
            vec![]
        } else {
            arg_str.split(' ').map(|s| s.to_string()).collect()
        };

        editor_storage.console_command.clear();
        command_text_edit.request_focus();

        // Execute immediately — we have both world and editor_storage here.
        let mut found = false;
        for cmd in inventory::iter::<ConsoleCommand> {
            if cmd.name.to_lowercase() == cmd_name.to_lowercase() {
                (cmd.func)(world, editor_storage, inputs);
                found = true;
                break;
            }
        }
        if !found {
            log_warn!("Command not found: {}", cmd_name);
        }
    }

    ui.allocate_space(ui.available_size());
}
