//! Dynamic Trackball UI callbacks
//!
//! Handles trackball configuration using model-driven dynamic UI.
//! This provides the same functionality as the static trackball page but uses a generic
//! callback approach with setting IDs instead of individual callbacks.

use crate::config::models::TrackballSettings;
use crate::config::{Settings, SettingsCategory};
use crate::constants::{ACCEL_SPEED_MAX, ACCEL_SPEED_MIN};
use crate::types::{AccelProfile, ScrollMethod};
use crate::{MainWindow, TrackballSettingModel};
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
) -> TrackballSettingModel {
    TrackballSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 0,
        bool_value: value,
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
) -> TrackballSettingModel {
    TrackballSettingModel {
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

fn make_slider_int(
    id: &str,
    label: &str,
    desc: &str,
    value: i32,
    min: f32,
    max: f32,
    suffix: &str,
    visible: bool,
) -> TrackballSettingModel {
    TrackballSettingModel {
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

fn make_combo(
    id: &str,
    label: &str,
    desc: &str,
    index: i32,
    options: &[&str],
    visible: bool,
) -> TrackballSettingModel {
    let opts: Vec<SharedString> = options.iter().map(|s| (*s).into()).collect();
    TrackballSettingModel {
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
) -> TrackballSettingModel {
    TrackballSettingModel {
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
fn populate_general_settings(trackball: &TrackballSettings) -> ModelRc<TrackballSettingModel> {
    let settings = vec![make_toggle(
        "off",
        "Disable trackball",
        "Turn off the trackball device entirely",
        trackball.off,
        true,
    )];
    ModelRc::new(VecModel::from(settings))
}

/// Build behavior settings model
fn populate_behavior_settings(trackball: &TrackballSettings) -> ModelRc<TrackballSettingModel> {
    let settings = vec![
        make_toggle(
            "natural_scroll",
            "Natural scrolling",
            "Scroll content in the direction of movement",
            trackball.natural_scroll,
            true,
        ),
        make_toggle(
            "left_handed",
            "Left-handed mode",
            "Swap left and right buttons",
            trackball.left_handed,
            true,
        ),
        make_toggle(
            "middle_emulation",
            "Middle click emulation",
            "Emulate middle click with left+right click",
            trackball.middle_emulation,
            true,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

/// Build acceleration settings model
fn populate_acceleration_settings(
    trackball: &TrackballSettings,
) -> ModelRc<TrackballSettingModel> {
    let accel_profile_index = trackball.accel_profile.to_index();
    // accel_speed is -1.0 to 1.0, display as percentage -100 to 100
    let accel_speed_display = (trackball.accel_speed * 100.0) as f32;

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
fn populate_scrolling_settings(trackball: &TrackballSettings) -> ModelRc<TrackballSettingModel> {
    let scroll_method_index = trackball.scroll_method.to_index();
    let is_on_button_down = matches!(trackball.scroll_method, ScrollMethod::OnButtonDown);

    let settings = vec![
        make_combo(
            "scroll_method",
            "Scroll method",
            "How to scroll with the trackball",
            scroll_method_index,
            &["Two finger", "Edge", "On button down", "Disabled"],
            true,
        ),
        make_slider_int(
            "scroll_button",
            "Scroll button",
            "Button code for on-button-down scrolling",
            trackball.scroll_button.unwrap_or(0),
            0.0,
            999.0,
            "",
            is_on_button_down,
        ),
        make_toggle(
            "scroll_button_lock",
            "Scroll button lock",
            "Toggle scroll mode instead of holding button",
            trackball.scroll_button_lock,
            is_on_button_down,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

// ============================================================================
// SYNC ALL MODELS
// ============================================================================

/// Sync all UI models from trackball settings
pub fn sync_all_models(ui: &MainWindow, trackball: &TrackballSettings) {
    ui.set_trackball_dynamic_device_off(trackball.off);
    ui.set_trackball_dynamic_general_settings(populate_general_settings(trackball));
    ui.set_trackball_dynamic_behavior_settings(populate_behavior_settings(trackball));
    ui.set_trackball_dynamic_acceleration_settings(populate_acceleration_settings(trackball));
    ui.set_trackball_dynamic_scrolling_settings(populate_scrolling_settings(trackball));
}

// ============================================================================
// CALLBACK SETUP
// ============================================================================

/// Set up dynamic trackball callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Generic toggle callback
    {
        let settings = Arc::clone(&settings);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_trackball_setting_toggle_changed(move |id, value| {
            let id_str = id.as_str();

            // Perform settings update and collect data for UI update
            let update_device_off = match settings.lock() {
                Ok(mut s) => {
                    let trackball = &mut s.trackball;
                    let mut update_device_off = false;

                    match id_str {
                        "off" => {
                            trackball.off = value;
                            update_device_off = true;
                        }
                        "natural_scroll" => {
                            trackball.natural_scroll = value;
                        }
                        "left_handed" => {
                            trackball.left_handed = value;
                        }
                        "middle_emulation" => {
                            trackball.middle_emulation = value;
                        }
                        "scroll_button_lock" => {
                            trackball.scroll_button_lock = value;
                        }
                        _ => {
                            debug!("Unknown trackball toggle setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Trackball toggle {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Trackball);
                    save_manager.request_save();

                    update_device_off
                }
                Err(e) => {
                    error!("Settings lock error: {}", e);
                    return;
                }
            };

            // UI updates happen after lock is released
            if update_device_off {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_trackball_dynamic_device_off(value);
                }
            }
        });
    }

    // Generic slider int callback
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        ui.on_trackball_setting_slider_int_changed(move |id, value| {
            let id_str = id.as_str();
            match settings.lock() {
                Ok(mut s) => {
                    let trackball = &mut s.trackball;

                    match id_str {
                        "scroll_button" => {
                            trackball.scroll_button = if value == 0 { None } else { Some(value) };
                        }
                        _ => {
                            debug!("Unknown trackball slider int setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Trackball slider int {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Trackball);
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
        ui.on_trackball_setting_slider_float_changed(move |id, value| {
            let id_str = id.as_str();
            match settings.lock() {
                Ok(mut s) => {
                    let trackball = &mut s.trackball;

                    match id_str {
                        "accel_speed" => {
                            // Convert from percentage (-100 to 100) to actual value (-1.0 to 1.0)
                            let actual_value =
                                (value as f64 / 100.0).clamp(ACCEL_SPEED_MIN, ACCEL_SPEED_MAX);
                            trackball.accel_speed = actual_value;
                        }
                        _ => {
                            debug!("Unknown trackball slider float setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Trackball slider float {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Trackball);
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
        ui.on_trackball_setting_combo_changed(move |id, index| {
            let id_str = id.as_str();
            match settings.lock() {
                Ok(mut s) => {
                    let trackball = &mut s.trackball;
                    let mut needs_model_refresh = false;

                    match id_str {
                        "accel_profile" => {
                            trackball.accel_profile = AccelProfile::from_index(index);
                        }
                        "scroll_method" => {
                            trackball.scroll_method = ScrollMethod::from_index(index);
                            needs_model_refresh = true; // scroll_button visibility depends on this
                        }
                        _ => {
                            debug!("Unknown trackball combo setting: {}", id_str);
                            return;
                        }
                    }

                    // Refresh models if visibility changed
                    if needs_model_refresh {
                        if let Some(ui) = ui_weak.upgrade() {
                            sync_all_models(&ui, trackball);
                        }
                    }

                    debug!("Trackball combo {} = {}", id_str, index);
                    save_manager.mark_dirty(SettingsCategory::Trackball);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Generic text callback (currently unused for trackball, but included for completeness)
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        ui.on_trackball_setting_text_changed(move |id, _value| {
            let id_str = id.as_str();
            match settings.lock() {
                Ok(mut _s) => {
                    // No text settings for trackball currently
                    debug!("Unknown trackball text setting: {}", id_str);
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
            // Mark dirty and save would go here if we had text settings
            let _ = save_manager;
        });
    }
}
