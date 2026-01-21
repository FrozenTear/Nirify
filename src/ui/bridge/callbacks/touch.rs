//! Dynamic Touch UI callbacks
//!
//! Handles touch screen configuration using model-driven dynamic UI.
//! This provides the same functionality as the static touch page but uses a generic
//! callback approach with setting IDs instead of individual callbacks.

use crate::config::models::TouchSettings;
use crate::config::{Settings, SettingsCategory};
use crate::{MainWindow, TouchSettingModel};
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
) -> TouchSettingModel {
    TouchSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 0,
        bool_value: value,
        visible,
        ..Default::default()
    }
}

fn make_text(
    id: &str,
    label: &str,
    desc: &str,
    value: &str,
    placeholder: &str,
    visible: bool,
) -> TouchSettingModel {
    TouchSettingModel {
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

fn make_slider_float(
    id: &str,
    label: &str,
    desc: &str,
    value: f32,
    min: f32,
    max: f32,
    suffix: &str,
    visible: bool,
) -> TouchSettingModel {
    TouchSettingModel {
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

#[allow(dead_code)]
fn make_combo(
    id: &str,
    label: &str,
    desc: &str,
    index: i32,
    options: &[&str],
    visible: bool,
) -> TouchSettingModel {
    let opts: Vec<SharedString> = options.iter().map(|s| (*s).into()).collect();
    TouchSettingModel {
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
// SECTION MODEL BUILDERS
// ============================================================================

/// Build general settings model (contains disable toggle)
fn populate_general_settings(touch: &TouchSettings) -> ModelRc<TouchSettingModel> {
    let settings = vec![make_toggle(
        "off",
        "Disable touch",
        "Turn off the touch device entirely",
        touch.off,
        true,
    )];
    ModelRc::new(VecModel::from(settings))
}

/// Build mapping settings model (map-to-output only, no left-handed for touch)
fn populate_mapping_settings(touch: &TouchSettings) -> ModelRc<TouchSettingModel> {
    let settings = vec![make_text(
        "map_to_output",
        "Map to output",
        "Monitor name to map touch to (e.g., eDP-1)",
        &touch.map_to_output,
        "All outputs",
        true,
    )];
    ModelRc::new(VecModel::from(settings))
}

/// Build calibration settings model (6-value matrix)
fn populate_calibration_settings(touch: &TouchSettings) -> ModelRc<TouchSettingModel> {
    let has_calibration = touch.calibration_matrix.is_some();
    let matrix = touch.calibration_matrix.unwrap_or([1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);

    let settings = vec![
        make_toggle(
            "has_calibration",
            "Custom calibration",
            "Enable custom calibration matrix",
            has_calibration,
            true,
        ),
        make_slider_float(
            "cal_m0",
            "Matrix [0]",
            "First value of transformation matrix",
            matrix[0] as f32,
            -10.0,
            10.0,
            "",
            has_calibration,
        ),
        make_slider_float(
            "cal_m1",
            "Matrix [1]",
            "Second value of transformation matrix",
            matrix[1] as f32,
            -10.0,
            10.0,
            "",
            has_calibration,
        ),
        make_slider_float(
            "cal_m2",
            "Matrix [2]",
            "Third value of transformation matrix",
            matrix[2] as f32,
            -10.0,
            10.0,
            "",
            has_calibration,
        ),
        make_slider_float(
            "cal_m3",
            "Matrix [3]",
            "Fourth value of transformation matrix",
            matrix[3] as f32,
            -10.0,
            10.0,
            "",
            has_calibration,
        ),
        make_slider_float(
            "cal_m4",
            "Matrix [4]",
            "Fifth value of transformation matrix",
            matrix[4] as f32,
            -10.0,
            10.0,
            "",
            has_calibration,
        ),
        make_slider_float(
            "cal_m5",
            "Matrix [5]",
            "Sixth value of transformation matrix",
            matrix[5] as f32,
            -10.0,
            10.0,
            "",
            has_calibration,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

// ============================================================================
// SYNC ALL MODELS
// ============================================================================

/// Sync all UI models from touch settings
pub fn sync_all_models(ui: &MainWindow, touch: &TouchSettings) {
    ui.set_touch_dynamic_device_off(touch.off);
    ui.set_touch_dynamic_general_settings(populate_general_settings(touch));
    ui.set_touch_dynamic_mapping_settings(populate_mapping_settings(touch));
    ui.set_touch_dynamic_calibration_settings(populate_calibration_settings(touch));
}

// ============================================================================
// CALLBACK SETUP
// ============================================================================

/// Set up dynamic touch callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Generic toggle callback
    {
        let settings = Arc::clone(&settings);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_touch_setting_toggle_changed(move |id, value| {
            let id_str = id.as_str();

            // Perform settings update and collect data for UI update
            let ui_update = match settings.lock() {
                Ok(mut s) => {
                    let touch = &mut s.touch;
                    let mut needs_model_refresh = false;
                    let mut update_device_off = false;

                    match id_str {
                        "off" => {
                            touch.off = value;
                            update_device_off = true;
                        }
                        "has_calibration" => {
                            if value {
                                // Enable calibration with default identity matrix
                                if touch.calibration_matrix.is_none() {
                                    touch.calibration_matrix =
                                        Some([1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
                                }
                            } else {
                                touch.calibration_matrix = None;
                            }
                            needs_model_refresh = true;
                        }
                        _ => {
                            debug!("Unknown touch toggle setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Touch toggle {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Touch);
                    save_manager.request_save();

                    // Clone data for UI update if needed
                    if needs_model_refresh || update_device_off {
                        Some((touch.clone(), needs_model_refresh, update_device_off))
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
            if let Some((touch_clone, needs_model_refresh, update_device_off)) = ui_update {
                if let Some(ui) = ui_weak.upgrade() {
                    if update_device_off {
                        ui.set_touch_dynamic_device_off(value);
                    }
                    if needs_model_refresh {
                        sync_all_models(&ui, &touch_clone);
                    }
                }
            }
        });
    }

    // Generic slider int callback (unused for touch)
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        ui.on_touch_setting_slider_int_changed(move |id, _value| {
            let id_str = id.as_str();
            match settings.lock() {
                Ok(mut _s) => {
                    // No int sliders for touch
                    debug!("Unknown touch slider int setting: {}", id_str);
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
            let _ = save_manager;
        });
    }

    // Generic slider float callback (calibration matrix)
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        ui.on_touch_setting_slider_float_changed(move |id, value| {
            let id_str = id.as_str();
            match settings.lock() {
                Ok(mut s) => {
                    let touch = &mut s.touch;

                    // Get or create calibration matrix
                    let matrix = touch
                        .calibration_matrix
                        .get_or_insert([1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);

                    match id_str {
                        "cal_m0" => matrix[0] = value as f64,
                        "cal_m1" => matrix[1] = value as f64,
                        "cal_m2" => matrix[2] = value as f64,
                        "cal_m3" => matrix[3] = value as f64,
                        "cal_m4" => matrix[4] = value as f64,
                        "cal_m5" => matrix[5] = value as f64,
                        _ => {
                            debug!("Unknown touch slider float setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Touch slider float {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Touch);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Generic combo callback (unused for touch)
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        ui.on_touch_setting_combo_changed(move |id, _index| {
            let id_str = id.as_str();
            match settings.lock() {
                Ok(mut _s) => {
                    // No combo settings for touch
                    debug!("Unknown touch combo setting: {}", id_str);
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
            let _ = save_manager;
        });
    }

    // Generic text callback (map-to-output)
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        ui.on_touch_setting_text_changed(move |id, value| {
            let id_str = id.as_str();
            match settings.lock() {
                Ok(mut s) => {
                    let touch = &mut s.touch;

                    match id_str {
                        "map_to_output" => {
                            touch.map_to_output = value.to_string();
                        }
                        _ => {
                            debug!("Unknown touch text setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Touch text {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Touch);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }
}
