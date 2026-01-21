//! Dynamic environment variables UI callbacks
//!
//! Handles environment variable configuration using model-driven dynamic UI
//! with add, remove, select, edit, and reorder capabilities.

use crate::config::models::EnvironmentVariable;
use crate::config::{Settings, SettingsCategory};
use crate::constants::MAX_ENVIRONMENT_VARS;
use crate::{EnvironmentVariableModel, MainWindow};
use log::{debug, error, warn};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::SaveManager;

// =============================================================================
// Helper functions for building UI models
// =============================================================================

/// Build variable list model for UI display from settings
fn build_variable_list_model(variables: &[EnvironmentVariable]) -> ModelRc<EnvironmentVariableModel> {
    let models: Vec<EnvironmentVariableModel> = variables
        .iter()
        .map(|var| EnvironmentVariableModel {
            id: var.id as i32,
            name: SharedString::from(var.name.as_str()),
            value: SharedString::from(var.value.as_str()),
        })
        .collect();
    ModelRc::new(VecModel::from(models))
}

/// Sync current variable fields to UI
fn sync_current_variable(ui: &MainWindow, var: &EnvironmentVariable) {
    ui.set_environment_dynamic_current_name(SharedString::from(var.name.as_str()));
    ui.set_environment_dynamic_current_value(SharedString::from(var.value.as_str()));
}

// =============================================================================
// Setup function
// =============================================================================

/// Set up dynamic environment variable callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Shared state for tracking selected index (Rc<Cell> since Slint callbacks are single-threaded)
    let selected_idx = Rc::new(Cell::new(-1i32));

    // Add variable callback
    {
        let settings = Arc::clone(&settings);
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_environment_dynamic_add_variable(move || {
            let (variables_clone, new_idx, new_id) = match settings.lock() {
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

                    // Clone data for UI update
                    (s.environment.variables.clone(), new_idx, new_id)
                }
                Err(e) => {
                    error!("Settings lock error: {}", e);
                    return;
                }
            };

            // UI updates happen after lock is released
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_environment_dynamic_variable_list(build_variable_list_model(
                    &variables_clone,
                ));
                ui.set_environment_dynamic_selected_index(new_idx);

                if let Some(var) = variables_clone.get(new_idx as usize) {
                    sync_current_variable(&ui, var);
                }
            }

            debug!("Added new environment variable with id {}", new_id);
            save_manager.mark_dirty(SettingsCategory::Environment);
            save_manager.request_save();
        });
    }

    // Remove variable callback
    {
        let settings = Arc::clone(&settings);
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_environment_dynamic_remove_variable(move |index| {
            let idx = index as usize;
            let (variables_clone, new_sel, name) = match settings.lock() {
                Ok(mut s) => {
                    if idx >= s.environment.variables.len() {
                        return;
                    }

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

                    // Clone data for UI update
                    (s.environment.variables.clone(), new_sel, name)
                }
                Err(e) => {
                    error!("Settings lock error: {}", e);
                    return;
                }
            };

            // UI updates happen after lock is released
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_environment_dynamic_variable_list(build_variable_list_model(
                    &variables_clone,
                ));
                ui.set_environment_dynamic_selected_index(new_sel);

                if new_sel >= 0 {
                    if let Some(var) = variables_clone.get(new_sel as usize) {
                        sync_current_variable(&ui, var);
                    }
                }
            }

            debug!("Removed environment variable at index {}: {}", idx, name);
            save_manager.mark_dirty(SettingsCategory::Environment);
            save_manager.request_save();
        });
    }

    // Select variable callback
    {
        let settings = Arc::clone(&settings);
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        ui.on_environment_dynamic_select_variable(move |index| {
            selected_idx.set(index);

            let var_clone = match settings.lock() {
                Ok(s) => s.environment.variables.get(index as usize).cloned(),
                Err(e) => {
                    error!("Settings lock error: {}", e);
                    return;
                }
            };

            // UI updates happen after lock is released
            if let Some(var) = var_clone {
                if let Some(ui) = ui_weak.upgrade() {
                    sync_current_variable(&ui, &var);
                }
            }
        });
    }

    // Name changed callback
    {
        let settings = Arc::clone(&settings);
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_environment_dynamic_name_changed(move |value| {
            let idx = selected_idx.get();

            if idx < 0 {
                return;
            }

            let mut value_str = value.to_string();

            // Validate string length to prevent memory issues
            if value_str.len() > crate::constants::MAX_STRING_LENGTH {
                warn!("Environment variable name exceeds maximum length, truncating");
                value_str.truncate(crate::constants::MAX_STRING_LENGTH);
            }

            let variables_clone = match settings.lock() {
                Ok(mut s) => {
                    if let Some(var) = s.environment.variables.get_mut(idx as usize) {
                        var.name = value_str;
                        // Clone data for UI update
                        Some(s.environment.variables.clone())
                    } else {
                        None
                    }
                }
                Err(e) => {
                    error!("Settings lock error: {}", e);
                    return;
                }
            };

            // UI updates happen after lock is released
            if let Some(variables) = variables_clone {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_environment_dynamic_variable_list(build_variable_list_model(
                        &variables,
                    ));
                }

                debug!("Changed environment variable name at index {}", idx);
                save_manager.mark_dirty(SettingsCategory::Environment);
                save_manager.request_save();
            }
        });
    }

    // Value changed callback
    {
        let settings = Arc::clone(&settings);
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_environment_dynamic_value_changed(move |value| {
            let idx = selected_idx.get();

            if idx < 0 {
                return;
            }

            let mut value_str = value.to_string();

            // Validate string length to prevent memory issues
            if value_str.len() > crate::constants::MAX_STRING_LENGTH {
                warn!("Environment variable value exceeds maximum length, truncating");
                value_str.truncate(crate::constants::MAX_STRING_LENGTH);
            }

            let variables_clone = match settings.lock() {
                Ok(mut s) => {
                    if let Some(var) = s.environment.variables.get_mut(idx as usize) {
                        var.value = value_str;
                        // Clone data for UI update
                        Some(s.environment.variables.clone())
                    } else {
                        None
                    }
                }
                Err(e) => {
                    error!("Settings lock error: {}", e);
                    return;
                }
            };

            // UI updates happen after lock is released
            if let Some(variables) = variables_clone {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_environment_dynamic_variable_list(build_variable_list_model(
                        &variables,
                    ));
                }

                debug!("Changed environment variable value at index {}", idx);
                save_manager.mark_dirty(SettingsCategory::Environment);
                save_manager.request_save();
            }
        });
    }

    // Reorder variable callback (drag-and-drop)
    {
        let settings = Arc::clone(&settings);
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_environment_dynamic_reorder_variable(move |from, to| {
            let from_idx = from as usize;
            let to_idx = to as usize;

            let (variables_clone, insert_idx) = match settings.lock() {
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

                    // Clone data for UI update
                    (s.environment.variables.clone(), insert_idx)
                }
                Err(e) => {
                    error!("Settings lock error: {}", e);
                    return;
                }
            };

            // UI updates happen after lock is released
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_environment_dynamic_variable_list(build_variable_list_model(
                    &variables_clone,
                ));
                ui.set_environment_dynamic_selected_index(insert_idx as i32);

                if let Some(var) = variables_clone.get(insert_idx) {
                    sync_current_variable(&ui, var);
                }
            }

            debug!(
                "Reordered environment variable from {} to {} (inserted at {})",
                from_idx, to_idx, insert_idx
            );
            save_manager.mark_dirty(SettingsCategory::Environment);
            save_manager.request_save();
        });
    }
}

// =============================================================================
// Public sync function
// =============================================================================

/// Sync the environment UI from settings
/// Call this on app startup to populate the UI from loaded settings
pub fn sync_environment_ui(ui: &MainWindow, settings: &Settings) {
    // Build and set the variable list model
    ui.set_environment_dynamic_variable_list(build_variable_list_model(
        &settings.environment.variables,
    ));

    // Reset selection state
    ui.set_environment_dynamic_selected_index(-1);
    ui.set_environment_dynamic_current_name(SharedString::new());
    ui.set_environment_dynamic_current_value(SharedString::new());
}
