//! Dynamic Trackpoint UI callbacks
//!
//! Handles trackpoint configuration using model-driven dynamic UI.
//! This provides the same functionality as trackpoint.rs but uses a generic
//! callback approach with setting IDs instead of individual callbacks.

use crate::config::models::TrackpointSettings;
use crate::config::{Settings, SettingsCategory};
use crate::constants::{ACCEL_SPEED_MAX, ACCEL_SPEED_MIN};
use crate::types::{AccelProfile, ScrollMethod};
use crate::{MainWindow, TrackpointSettingModel};
use log::{debug, error};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::SaveManager;

// ============================================================================
// HELPER FUNCTIONS FOR CREATING SETTING MODELS
// ============================================================================

fn make_toggle(
    id: &str,
    label: &str,
    desc: &str,
    value: bool,
    visible: bool,
) -> TrackpointSettingModel {
    TrackpointSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 0,
        bool_value: value,
        visible,
        ..Default::default()
    }
}

fn make_slider_int(
    id: &str,
    label: &str,
    desc: &str,
    value: i32,
    min: f32,
    max: f32,
    suffix: &str,
    visible: bool,
) -> TrackpointSettingModel {
    TrackpointSettingModel {
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

fn make_slider_float(
    id: &str,
    label: &str,
    desc: &str,
    value: f32,
    min: f32,
    max: f32,
    suffix: &str,
    visible: bool,
) -> TrackpointSettingModel {
    TrackpointSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 1,
        float_value: value,
        min_value: min,
        max_value: max,
        suffix: suffix.into(),
        use_float: true,
        visible,
        ..Default::default()
    }
}

fn make_combo(
    id: &str,
    label: &str,
    desc: &str,
    index: i32,
    options: &[&str],
    visible: bool,
) -> TrackpointSettingModel {
    let opts: Vec<SharedString> = options.iter().map(|s| (*s).into()).collect();
    TrackpointSettingModel {
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

#[allow(dead_code)]
fn make_text(
    id: &str,
    label: &str,
    desc: &str,
    value: &str,
    placeholder: &str,
    visible: bool,
) -> TrackpointSettingModel {
    TrackpointSettingModel {
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

// ============================================================================
// SECTION MODEL BUILDERS
// ============================================================================

/// Build general settings model (contains disable toggle)
fn populate_general_settings(trackpoint: &TrackpointSettings) -> ModelRc<TrackpointSettingModel> {
    let settings = vec![make_toggle(
        "off",
        "Disable trackpoint",
        "Turn off the trackpoint device entirely",
        trackpoint.off,
        true,
    )];
    ModelRc::new(VecModel::from(settings))
}

/// Build behavior settings model
fn populate_behavior_settings(trackpoint: &TrackpointSettings) -> ModelRc<TrackpointSettingModel> {
    let settings = vec![
        make_toggle(
            "natural_scroll",
            "Natural scrolling",
            "Scroll content in the direction of movement",
            trackpoint.natural_scroll,
            true,
        ),
        make_toggle(
            "left_handed",
            "Left-handed mode",
            "Swap left and right buttons",
            trackpoint.left_handed,
            true,
        ),
        make_toggle(
            "middle_emulation",
            "Middle click emulation",
            "Emulate middle click with left+right click",
            trackpoint.middle_emulation,
            true,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

/// Build acceleration settings model
fn populate_acceleration_settings(
    trackpoint: &TrackpointSettings,
) -> ModelRc<TrackpointSettingModel> {
    let accel_profile_index = trackpoint.accel_profile.to_index();
    // accel_speed is -1.0 to 1.0, display as percentage -100 to 100
    let accel_speed_display = (trackpoint.accel_speed * 100.0) as f32;

    let settings = vec![
        make_combo(
            "accel_profile",
            "Acceleration profile",
            "How pointer speed changes with movement",
            accel_profile_index,
            &["Adaptive", "Flat"],
            true,
        ),
        make_slider_float(
            "accel_speed",
            "Acceleration speed",
            "Pointer speed adjustment (-100 to 100)",
            accel_speed_display,
            (ACCEL_SPEED_MIN * 100.0) as f32,
            (ACCEL_SPEED_MAX * 100.0) as f32,
            "",
            true,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

/// Build scrolling settings model
fn populate_scrolling_settings(trackpoint: &TrackpointSettings) -> ModelRc<TrackpointSettingModel> {
    let scroll_method_index = trackpoint.scroll_method.to_index();
    let is_on_button_down = matches!(trackpoint.scroll_method, ScrollMethod::OnButtonDown);

    let settings = vec![
        make_combo(
            "scroll_method",
            "Scroll method",
            "How to scroll with the trackpoint",
            scroll_method_index,
            &["Two finger", "Edge", "On button down", "Disabled"],
            true,
        ),
        make_slider_int(
            "scroll_button",
            "Scroll button",
            "Button code for on-button-down scrolling",
            trackpoint.scroll_button.unwrap_or(0),
            0.0,
            999.0,
            "",
            is_on_button_down,
        ),
        make_toggle(
            "scroll_button_lock",
            "Scroll button lock",
            "Toggle scroll mode instead of holding button",
            trackpoint.scroll_button_lock,
            is_on_button_down,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

// ============================================================================
// SYNC ALL MODELS
// ============================================================================

/// Sync all UI models from trackpoint settings
pub fn sync_all_models(ui: &MainWindow, trackpoint: &TrackpointSettings) {
    ui.set_trackpoint_dynamic_device_off(trackpoint.off);
    ui.set_trackpoint_dynamic_general_settings(populate_general_settings(trackpoint));
    ui.set_trackpoint_dynamic_behavior_settings(populate_behavior_settings(trackpoint));
    ui.set_trackpoint_dynamic_acceleration_settings(populate_acceleration_settings(trackpoint));
    ui.set_trackpoint_dynamic_scrolling_settings(populate_scrolling_settings(trackpoint));
}

// ============================================================================
// CALLBACK SETUP
// ============================================================================

/// Set up dynamic trackpoint callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Generic toggle callback
    {
        let settings = Arc::clone(&settings);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_trackpoint_setting_toggle_changed(move |id, value| {
            let id_str = id.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    let trackpoint = &mut s.trackpoint;

                    match id_str.as_str() {
                        "off" => {
                            trackpoint.off = value;
                            // Update device_off property for section visibility
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_trackpoint_dynamic_device_off(value);
                            }
                        }
                        "natural_scroll" => {
                            trackpoint.natural_scroll = value;
                        }
                        "left_handed" => {
                            trackpoint.left_handed = value;
                        }
                        "middle_emulation" => {
                            trackpoint.middle_emulation = value;
                        }
                        "scroll_button_lock" => {
                            trackpoint.scroll_button_lock = value;
                        }
                        _ => {
                            debug!("Unknown trackpoint toggle setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Trackpoint toggle {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Trackpoint);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Generic slider int callback
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        ui.on_trackpoint_setting_slider_int_changed(move |id, value| {
            let id_str = id.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    let trackpoint = &mut s.trackpoint;

                    match id_str.as_str() {
                        "scroll_button" => {
                            trackpoint.scroll_button = if value == 0 { None } else { Some(value) };
                        }
                        _ => {
                            debug!("Unknown trackpoint slider int setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Trackpoint slider int {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Trackpoint);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Generic slider float callback
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        ui.on_trackpoint_setting_slider_float_changed(move |id, value| {
            let id_str = id.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    let trackpoint = &mut s.trackpoint;

                    match id_str.as_str() {
                        "accel_speed" => {
                            // Convert from percentage (-100 to 100) to actual value (-1.0 to 1.0)
                            let actual_value =
                                (value as f64 / 100.0).clamp(ACCEL_SPEED_MIN, ACCEL_SPEED_MAX);
                            trackpoint.accel_speed = actual_value;
                        }
                        _ => {
                            debug!("Unknown trackpoint slider float setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Trackpoint slider float {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Trackpoint);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Generic combo callback
    {
        let settings = Arc::clone(&settings);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_trackpoint_setting_combo_changed(move |id, index| {
            let id_str = id.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    let trackpoint = &mut s.trackpoint;
                    let mut needs_model_refresh = false;

                    match id_str.as_str() {
                        "accel_profile" => {
                            trackpoint.accel_profile = AccelProfile::from_index(index);
                        }
                        "scroll_method" => {
                            trackpoint.scroll_method = ScrollMethod::from_index(index);
                            needs_model_refresh = true; // scroll_button visibility depends on this
                        }
                        _ => {
                            debug!("Unknown trackpoint combo setting: {}", id_str);
                            return;
                        }
                    }

                    // Refresh models if visibility changed
                    if needs_model_refresh {
                        if let Some(ui) = ui_weak.upgrade() {
                            sync_all_models(&ui, trackpoint);
                        }
                    }

                    debug!("Trackpoint combo {} = {}", id_str, index);
                    save_manager.mark_dirty(SettingsCategory::Trackpoint);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Generic text callback (currently unused for trackpoint, but included for completeness)
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        ui.on_trackpoint_setting_text_changed(move |id, _value| {
            let id_str = id.to_string();
            match settings.lock() {
                Ok(mut _s) => {
                    // No text settings for trackpoint currently
                    debug!("Unknown trackpoint text setting: {}", id_str);
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
            // Mark dirty and save would go here if we had text settings
            let _ = save_manager;
        });
    }
}
