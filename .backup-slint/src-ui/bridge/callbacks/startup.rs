//! Dynamic startup commands UI callbacks
//!
//! Handles spawn-at-startup command configuration including add, remove, select,
//! edit, and reorder operations using model-driven dynamic UI.

use crate::config::models::StartupCommand;
use crate::config::{Settings, SettingsCategory};
use crate::constants::MAX_STARTUP_COMMANDS;
use crate::MainWindow;
use log::{debug, error, warn};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::SaveManager;

// ============================================================================
// Helper functions
// ============================================================================

/// Build command list model for UI display
fn build_command_list_model(commands: &[StartupCommand]) -> ModelRc<SharedString> {
    let display_list: Vec<SharedString> = commands
        .iter()
        .map(|cmd| SharedString::from(cmd.display()))
        .collect();
    ModelRc::new(VecModel::from(display_list))
}

/// Parse a command string into parts, respecting quotes
fn parse_command_string(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = '"';

    for c in input.chars() {
        match c {
            '"' | '\'' if !in_quotes => {
                in_quotes = true;
                quote_char = c;
            }
            c if c == quote_char && in_quotes => {
                in_quotes = false;
            }
            ' ' if !in_quotes => {
                if !current.is_empty() {
                    parts.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

// ============================================================================
// Setup function
// ============================================================================

/// Set up dynamic startup commands callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Shared state for tracking selected index (Rc<Cell> since Slint callbacks are single-threaded)
    let selected_idx = Rc::new(Cell::new(-1i32));

    // Add command callback
    {
        let settings = Arc::clone(&settings);
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_startup_dynamic_command_add(move || {
            match settings.lock() {
                Ok(mut s) => {
                    // Check limit before adding
                    if s.startup.commands.len() >= MAX_STARTUP_COMMANDS {
                        warn!(
                            "Maximum startup commands limit ({}) reached",
                            MAX_STARTUP_COMMANDS
                        );
                        return;
                    }

                    let new_id = s.startup.next_id;
                    s.startup.next_id += 1;

                    let cmd = StartupCommand {
                        id: new_id,
                        command: vec!["new-command".to_string()],
                    };
                    s.startup.commands.push(cmd);

                    let new_idx = (s.startup.commands.len() - 1) as i32;
                    selected_idx.set(new_idx);

                    // Update UI
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_startup_dynamic_command_list(build_command_list_model(
                            &s.startup.commands,
                        ));
                        ui.set_startup_dynamic_selected_index(new_idx);

                        if let Some(cmd) = s.startup.commands.get(new_idx as usize) {
                            ui.set_startup_dynamic_current_command(SharedString::from(
                                cmd.display(),
                            ));
                        }
                    }

                    debug!("Added new startup command with id {}", new_id);
                    save_manager.mark_dirty(SettingsCategory::Startup);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Remove command callback
    {
        let settings = Arc::clone(&settings);
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_startup_dynamic_command_remove(move |index| {
            let idx = index as usize;
            match settings.lock() {
                Ok(mut s) => {
                    if idx < s.startup.commands.len() {
                        let display = s.startup.commands[idx].display();
                        s.startup.commands.remove(idx);

                        // Update selected index
                        let new_sel = if s.startup.commands.is_empty() {
                            -1
                        } else if idx >= s.startup.commands.len() {
                            (s.startup.commands.len() - 1) as i32
                        } else {
                            idx as i32
                        };

                        selected_idx.set(new_sel);

                        // Update UI
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_startup_dynamic_command_list(build_command_list_model(
                                &s.startup.commands,
                            ));
                            ui.set_startup_dynamic_selected_index(new_sel);

                            if new_sel >= 0 {
                                if let Some(cmd) = s.startup.commands.get(new_sel as usize) {
                                    ui.set_startup_dynamic_current_command(SharedString::from(
                                        cmd.display(),
                                    ));
                                }
                            } else {
                                ui.set_startup_dynamic_current_command(SharedString::new());
                            }
                        }

                        debug!("Removed startup command at index {}: {}", idx, display);
                        save_manager.mark_dirty(SettingsCategory::Startup);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Select command callback
    {
        let settings = Arc::clone(&settings);
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        ui.on_startup_dynamic_command_select(move |index| {
            selected_idx.set(index);

            match settings.lock() {
                Ok(s) => {
                    if let Some(cmd) = s.startup.commands.get(index as usize) {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_startup_dynamic_selected_index(index);
                            ui.set_startup_dynamic_current_command(SharedString::from(
                                cmd.display(),
                            ));
                        }
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Command changed callback
    {
        let settings = Arc::clone(&settings);
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_startup_dynamic_command_changed(move |value| {
            let idx = selected_idx.get();

            if idx < 0 {
                return;
            }

            match settings.lock() {
                Ok(mut s) => {
                    if let Some(cmd) = s.startup.commands.get_mut(idx as usize) {
                        // Parse the command string into parts
                        // Simple parsing: split by whitespace, respecting quotes
                        let parts = parse_command_string(value.as_str());
                        if !parts.is_empty() {
                            cmd.command = parts;

                            // Update list display
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_startup_dynamic_command_list(build_command_list_model(
                                    &s.startup.commands,
                                ));
                            }

                            debug!("Changed startup command at index {}", idx);
                            save_manager.mark_dirty(SettingsCategory::Startup);
                            save_manager.request_save();
                        }
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Reorder command callback (drag-and-drop)
    {
        let settings = Arc::clone(&settings);
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_startup_dynamic_reorder_command(move |from, to| {
            let from_idx = from as usize;
            let to_idx = to as usize;

            match settings.lock() {
                Ok(mut s) => {
                    let len = s.startup.commands.len();
                    if from_idx >= len || to_idx > len || from_idx == to_idx {
                        return;
                    }

                    // Remove from original position and insert at new position
                    let item = s.startup.commands.remove(from_idx);
                    let insert_idx = if to_idx > from_idx {
                        to_idx - 1
                    } else {
                        to_idx
                    };
                    s.startup.commands.insert(insert_idx, item);

                    // Update selected index to follow moved item
                    selected_idx.set(insert_idx as i32);

                    // Update UI
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_startup_dynamic_command_list(build_command_list_model(
                            &s.startup.commands,
                        ));
                        ui.set_startup_dynamic_selected_index(insert_idx as i32);

                        if let Some(cmd) = s.startup.commands.get(insert_idx) {
                            ui.set_startup_dynamic_current_command(SharedString::from(
                                cmd.display(),
                            ));
                        }
                    }

                    debug!(
                        "Reordered startup command from {} to {} (inserted at {})",
                        from_idx, to_idx, insert_idx
                    );
                    save_manager.mark_dirty(SettingsCategory::Startup);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }
}

// ============================================================================
// Public sync function
// ============================================================================

/// Sync startup commands UI from settings
pub fn sync_startup_ui(ui: &MainWindow, settings: &Settings) {
    // Set command list
    ui.set_startup_dynamic_command_list(build_command_list_model(&settings.startup.commands));

    // Reset selection state
    ui.set_startup_dynamic_selected_index(-1);
    ui.set_startup_dynamic_current_command(SharedString::new());
}
