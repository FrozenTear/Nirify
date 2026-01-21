//! Dynamic switch events UI callbacks
//!
//! Handles lid close/open and tablet mode switch event configuration
//! using model-driven dynamic UI with generic callbacks.

use crate::config::models::SwitchEventsSettings;
use crate::config::{Settings, SettingsCategory};
use crate::{MainWindow, SwitchEventSettingModel};
use log::{debug, error};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::SaveManager;

// ============================================================================
// Helper functions for creating setting models
// ============================================================================

/// Create a toggle setting model
#[allow(dead_code)]
fn make_toggle(
    id: &str,
    label: &str,
    desc: &str,
    value: bool,
    visible: bool,
) -> SwitchEventSettingModel {
    SwitchEventSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 0,
        bool_value: value,
        visible,
        ..Default::default()
    }
}

/// Create a text input setting model
#[allow(dead_code)]
fn make_text(
    id: &str,
    label: &str,
    desc: &str,
    value: &str,
    placeholder: &str,
    visible: bool,
) -> SwitchEventSettingModel {
    SwitchEventSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 3,
        text_value: value.into(),
        placeholder: placeholder.into(),
        visible,
        ..Default::default()
    }
}

/// Create an integer slider setting model
#[allow(dead_code)]
fn make_slider_int(
    id: &str,
    label: &str,
    desc: &str,
    value: i32,
    min: f32,
    max: f32,
    suffix: &str,
    visible: bool,
) -> SwitchEventSettingModel {
    SwitchEventSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 1,
        int_value: value,
        min_value: min,
        max_value: max,
        suffix: suffix.into(),
        use_float: false,
        visible,
        ..Default::default()
    }
}

/// Create a combo box setting model
#[allow(dead_code)]
fn make_combo(
    id: &str,
    label: &str,
    desc: &str,
    index: i32,
    options: &[&str],
    visible: bool,
) -> SwitchEventSettingModel {
    let opts: Vec<SharedString> = options.iter().map(|s| (*s).into()).collect();
    SwitchEventSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 2,
        combo_index: index,
        combo_options: ModelRc::new(VecModel::from(opts)),
        visible,
        ..Default::default()
    }
}

// ============================================================================
// Command list helpers
// ============================================================================

/// Build command list model for UI display
fn build_command_list(commands: &[String]) -> ModelRc<SharedString> {
    let display_list: Vec<SharedString> = commands
        .iter()
        .map(|s| SharedString::from(s.as_str()))
        .collect();
    ModelRc::new(VecModel::from(display_list))
}

/// Populate general settings model (for any future non-list settings)
#[allow(dead_code)]
fn populate_general_settings() -> ModelRc<SwitchEventSettingModel> {
    // Currently switch events only has command lists, no other settings
    // This is a placeholder for potential future settings
    let settings: Vec<SwitchEventSettingModel> = vec![];
    ModelRc::new(VecModel::from(settings))
}

// ============================================================================
// Event ID helpers
// ============================================================================

/// Get commands for a specific event type
fn get_commands<'a>(settings: &'a SwitchEventsSettings, event_id: &str) -> &'a Vec<String> {
    match event_id {
        "lid_close" => &settings.lid_close.spawn,
        "lid_open" => &settings.lid_open.spawn,
        "tablet_mode_on" => &settings.tablet_mode_on.spawn,
        "tablet_mode_off" => &settings.tablet_mode_off.spawn,
        _ => &settings.lid_close.spawn, // fallback
    }
}

/// Get mutable commands for a specific event type
fn get_commands_mut<'a>(
    settings: &'a mut SwitchEventsSettings,
    event_id: &str,
) -> &'a mut Vec<String> {
    match event_id {
        "lid_close" => &mut settings.lid_close.spawn,
        "lid_open" => &mut settings.lid_open.spawn,
        "tablet_mode_on" => &mut settings.tablet_mode_on.spawn,
        "tablet_mode_off" => &mut settings.tablet_mode_off.spawn,
        _ => &mut settings.lid_close.spawn, // fallback
    }
}

/// Update UI for a specific event type after command list changes
fn update_event_ui(ui: &MainWindow, settings: &SwitchEventsSettings, event_id: &str) {
    let commands = get_commands(settings, event_id);
    let model = build_command_list(commands);

    match event_id {
        "lid_close" => {
            ui.set_switch_events_lid_close_commands(model);
            ui.set_switch_events_lid_close_selected(-1);
            ui.set_switch_events_lid_close_current(SharedString::new());
        }
        "lid_open" => {
            ui.set_switch_events_lid_open_commands(model);
            ui.set_switch_events_lid_open_selected(-1);
            ui.set_switch_events_lid_open_current(SharedString::new());
        }
        "tablet_mode_on" => {
            ui.set_switch_events_tablet_mode_on_commands(model);
            ui.set_switch_events_tablet_mode_on_selected(-1);
            ui.set_switch_events_tablet_mode_on_current(SharedString::new());
        }
        "tablet_mode_off" => {
            ui.set_switch_events_tablet_mode_off_commands(model);
            ui.set_switch_events_tablet_mode_off_selected(-1);
            ui.set_switch_events_tablet_mode_off_current(SharedString::new());
        }
        _ => {}
    }
}

/// Get selected index for a specific event type from UI
fn get_selected_index(ui: &MainWindow, event_id: &str) -> i32 {
    match event_id {
        "lid_close" => ui.get_switch_events_lid_close_selected(),
        "lid_open" => ui.get_switch_events_lid_open_selected(),
        "tablet_mode_on" => ui.get_switch_events_tablet_mode_on_selected(),
        "tablet_mode_off" => ui.get_switch_events_tablet_mode_off_selected(),
        _ => -1,
    }
}

/// Set selected index for a specific event type in UI
fn set_selected_index(ui: &MainWindow, event_id: &str, index: i32) {
    match event_id {
        "lid_close" => ui.set_switch_events_lid_close_selected(index),
        "lid_open" => ui.set_switch_events_lid_open_selected(index),
        "tablet_mode_on" => ui.set_switch_events_tablet_mode_on_selected(index),
        "tablet_mode_off" => ui.set_switch_events_tablet_mode_off_selected(index),
        _ => {}
    }
}

/// Set current text for a specific event type in UI
fn set_current_text(ui: &MainWindow, event_id: &str, text: &str) {
    match event_id {
        "lid_close" => ui.set_switch_events_lid_close_current(text.into()),
        "lid_open" => ui.set_switch_events_lid_open_current(text.into()),
        "tablet_mode_on" => ui.set_switch_events_tablet_mode_on_current(text.into()),
        "tablet_mode_off" => ui.set_switch_events_tablet_mode_off_current(text.into()),
        _ => {}
    }
}

/// Update command list display for a specific event type
fn update_command_list(ui: &MainWindow, settings: &SwitchEventsSettings, event_id: &str) {
    let commands = get_commands(settings, event_id);
    let model = build_command_list(commands);

    match event_id {
        "lid_close" => ui.set_switch_events_lid_close_commands(model),
        "lid_open" => ui.set_switch_events_lid_open_commands(model),
        "tablet_mode_on" => ui.set_switch_events_tablet_mode_on_commands(model),
        "tablet_mode_off" => ui.set_switch_events_tablet_mode_off_commands(model),
        _ => {}
    }
}

// ============================================================================
// Setup function
// ============================================================================

/// Set up dynamic switch events callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Command Add callback - dispatched by event_id
    {
        let settings = Arc::clone(&settings);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_switch_events_command_add(move |event_id, command_text| {
            let event_id_str = event_id.as_str();
            let cmd = command_text.to_string();

            if cmd.trim().is_empty() {
                return;
            }

            match settings.lock() {
                Ok(mut s) => {
                    let commands = get_commands_mut(&mut s.switch_events, &event_id_str);
                    commands.push(cmd.clone());

                    if let Some(ui) = ui_weak.upgrade() {
                        update_event_ui(&ui, &s.switch_events, &event_id_str);
                    }

                    debug!("Added {} command: {}", event_id_str, cmd);
                    save_manager.mark_dirty(SettingsCategory::SwitchEvents);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Command Remove callback - dispatched by event_id
    {
        let settings = Arc::clone(&settings);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_switch_events_command_remove(move |event_id, index| {
            let event_id_str = event_id.as_str();
            let idx = index as usize;

            match settings.lock() {
                Ok(mut s) => {
                    let commands = get_commands_mut(&mut s.switch_events, &event_id_str);
                    if idx < commands.len() {
                        commands.remove(idx);

                        if let Some(ui) = ui_weak.upgrade() {
                            update_event_ui(&ui, &s.switch_events, &event_id_str);
                        }

                        debug!("Removed {} command at index {}", event_id_str, idx);
                        save_manager.mark_dirty(SettingsCategory::SwitchEvents);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Command Select callback - dispatched by event_id
    {
        let settings = Arc::clone(&settings);
        let ui_weak = ui.as_weak();
        ui.on_switch_events_command_select(move |event_id, index| {
            let event_id_str = event_id.as_str();

            match settings.lock() {
                Ok(s) => {
                    let commands = get_commands(&s.switch_events, &event_id_str);
                    if let Some(cmd) = commands.get(index as usize) {
                        if let Some(ui) = ui_weak.upgrade() {
                            set_selected_index(&ui, &event_id_str, index);
                            set_current_text(&ui, &event_id_str, cmd);
                        }
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Command Text Changed callback - updates selected command in-place
    {
        let settings = Arc::clone(&settings);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_switch_events_command_text_changed(move |event_id, new_text| {
            let event_id_str = event_id.as_str();
            let text = new_text.to_string();

            if let Some(ui) = ui_weak.upgrade() {
                let idx = get_selected_index(&ui, &event_id_str);

                // Always update the current text field
                set_current_text(&ui, &event_id_str, &text);

                // If an item is selected, update it in the model
                if idx >= 0 {
                    match settings.lock() {
                        Ok(mut s) => {
                            let commands = get_commands_mut(&mut s.switch_events, &event_id_str);
                            if let Some(cmd) = commands.get_mut(idx as usize) {
                                *cmd = text;
                                update_command_list(&ui, &s.switch_events, &event_id_str);
                                save_manager.mark_dirty(SettingsCategory::SwitchEvents);
                                save_manager.request_save();
                            }
                        }
                        Err(e) => error!("Settings lock error: {}", e),
                    }
                }
            }
        });
    }
}

// ============================================================================
// Public sync functions
// ============================================================================

/// Sync all switch events UI from settings
pub fn sync_switch_events_ui(ui: &MainWindow, settings: &SwitchEventsSettings) {
    // Lid close
    ui.set_switch_events_lid_close_commands(build_command_list(&settings.lid_close.spawn));
    ui.set_switch_events_lid_close_selected(-1);
    ui.set_switch_events_lid_close_current(SharedString::new());

    // Lid open
    ui.set_switch_events_lid_open_commands(build_command_list(&settings.lid_open.spawn));
    ui.set_switch_events_lid_open_selected(-1);
    ui.set_switch_events_lid_open_current(SharedString::new());

    // Tablet mode on
    ui.set_switch_events_tablet_mode_on_commands(build_command_list(
        &settings.tablet_mode_on.spawn,
    ));
    ui.set_switch_events_tablet_mode_on_selected(-1);
    ui.set_switch_events_tablet_mode_on_current(SharedString::new());

    // Tablet mode off
    ui.set_switch_events_tablet_mode_off_commands(build_command_list(
        &settings.tablet_mode_off.spawn,
    ));
    ui.set_switch_events_tablet_mode_off_selected(-1);
    ui.set_switch_events_tablet_mode_off_current(SharedString::new());

    // General settings (currently empty)
    ui.set_switch_events_general_settings(populate_general_settings());
}
