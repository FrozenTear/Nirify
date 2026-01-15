//! Environment variables UI callbacks
//!
//! Handles environment variable configuration including add, remove, select, and editing.

use crate::config::models::EnvironmentVariable;
use crate::config::{Settings, SettingsCategory};
use crate::constants::MAX_ENVIRONMENT_VARS;
use crate::MainWindow;
use log::{debug, error, warn};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::SaveManager;

/// Build variable list model for UI display
fn build_variable_list_model(variables: &[EnvironmentVariable]) -> ModelRc<SharedString> {
    let mut display_list = Vec::with_capacity(variables.len());
    for var in variables {
        display_list.push(SharedString::from(format!("{}={}", var.name, var.value)));
    }
    ModelRc::new(VecModel::from(display_list))
}

/// Sync current variable to UI
fn sync_current_variable(ui: &MainWindow, var: &EnvironmentVariable) {
    ui.set_current_env_name(SharedString::from(var.name.as_str()));
    ui.set_current_env_value(SharedString::from(var.value.as_str()));
}

/// Set up environment variable callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Shared state for tracking selected index (Rc<Cell> since Slint callbacks are single-threaded)
    let selected_idx = Rc::new(Cell::new(-1i32));

    // Add variable callback
    {
        let settings = settings.clone();
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_environment_add_variable(move || {
            match settings.lock() {
                Ok(mut s) => {
                    // Check limit before adding
                    if s.environment.variables.len() >= MAX_ENVIRONMENT_VARS {
                        warn!(
                            "Maximum environment variables limit ({}) reached",
                            MAX_ENVIRONMENT_VARS
                        );
                        return;
                    }

                    let new_id = s.environment.next_id;
                    s.environment.next_id += 1;

                    let var = EnvironmentVariable {
                        id: new_id,
                        name: "NEW_VAR".to_string(),
                        value: "value".to_string(),
                    };
                    s.environment.variables.push(var);

                    let new_idx = (s.environment.variables.len() - 1) as i32;
                    selected_idx.set(new_idx);

                    // Update UI
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_environment_variable_list(build_variable_list_model(
                            &s.environment.variables,
                        ));
                        ui.set_selected_environment_index(new_idx);

                        if let Some(var) = s.environment.variables.get(new_idx as usize) {
                            sync_current_variable(&ui, var);
                        }
                    }

                    debug!("Added new environment variable with id {}", new_id);
                    save_manager.mark_dirty(SettingsCategory::Environment);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Remove variable callback
    {
        let settings = settings.clone();
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_environment_remove_variable(move |index| {
            let idx = index as usize;
            match settings.lock() {
                Ok(mut s) => {
                    if idx < s.environment.variables.len() {
                        let name = s.environment.variables[idx].name.clone();
                        s.environment.variables.remove(idx);

                        // Update selected index
                        let new_sel = if s.environment.variables.is_empty() {
                            -1
                        } else if idx >= s.environment.variables.len() {
                            (s.environment.variables.len() - 1) as i32
                        } else {
                            idx as i32
                        };

                        selected_idx.set(new_sel);

                        // Update UI
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_environment_variable_list(build_variable_list_model(
                                &s.environment.variables,
                            ));
                            ui.set_selected_environment_index(new_sel);

                            if new_sel >= 0 {
                                if let Some(var) = s.environment.variables.get(new_sel as usize) {
                                    sync_current_variable(&ui, var);
                                }
                            }
                        }

                        debug!("Removed environment variable at index {}: {}", idx, name);
                        save_manager.mark_dirty(SettingsCategory::Environment);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Select variable callback
    {
        let settings = settings.clone();
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        ui.on_environment_select_variable(move |index| {
            selected_idx.set(index);

            match settings.lock() {
                Ok(s) => {
                    if let Some(var) = s.environment.variables.get(index as usize) {
                        if let Some(ui) = ui_weak.upgrade() {
                            sync_current_variable(&ui, var);
                        }
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Name changed callback
    {
        let settings = settings.clone();
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_environment_name_changed(move |value| {
            let idx = selected_idx.get();

            if idx < 0 {
                return;
            }

            match settings.lock() {
                Ok(mut s) => {
                    if let Some(var) = s.environment.variables.get_mut(idx as usize) {
                        var.name = value.to_string();

                        // Update list display
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_environment_variable_list(build_variable_list_model(
                                &s.environment.variables,
                            ));
                        }

                        debug!("Changed environment variable name at index {}", idx);
                        save_manager.mark_dirty(SettingsCategory::Environment);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Value changed callback
    {
        let settings = settings.clone();
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_environment_value_changed(move |value| {
            let idx = selected_idx.get();

            if idx < 0 {
                return;
            }

            match settings.lock() {
                Ok(mut s) => {
                    if let Some(var) = s.environment.variables.get_mut(idx as usize) {
                        var.value = value.to_string();

                        // Update list display
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_environment_variable_list(build_variable_list_model(
                                &s.environment.variables,
                            ));
                        }

                        debug!("Changed environment variable value at index {}", idx);
                        save_manager.mark_dirty(SettingsCategory::Environment);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Reorder variable callback (drag-and-drop)
    {
        let settings = settings.clone();
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_environment_reorder_variable(move |from, to| {
            let from_idx = from as usize;
            let to_idx = to as usize;

            match settings.lock() {
                Ok(mut s) => {
                    let len = s.environment.variables.len();
                    if from_idx >= len || to_idx > len || from_idx == to_idx {
                        return;
                    }

                    // Remove from original position and insert at new position
                    let item = s.environment.variables.remove(from_idx);
                    let insert_idx = if to_idx > from_idx {
                        to_idx - 1
                    } else {
                        to_idx
                    };
                    s.environment.variables.insert(insert_idx, item);

                    // Update selected index to follow moved item
                    selected_idx.set(insert_idx as i32);

                    // Update UI
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_environment_variable_list(build_variable_list_model(
                            &s.environment.variables,
                        ));
                        ui.set_selected_environment_index(insert_idx as i32);

                        if let Some(var) = s.environment.variables.get(insert_idx) {
                            sync_current_variable(&ui, var);
                        }
                    }

                    debug!(
                        "Reordered environment variable from {} to {} (inserted at {})",
                        from_idx, to_idx, insert_idx
                    );
                    save_manager.mark_dirty(SettingsCategory::Environment);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }
}
