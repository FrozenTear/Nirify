//! Dynamic Tablet UI callbacks
//!
//! Handles tablet (drawing tablet) configuration using model-driven dynamic UI.
//! This provides the same functionality as the static tablet page but uses a generic
//! callback approach with setting IDs instead of individual callbacks.

use crate::config::models::TabletSettings;
use crate::config::{Settings, SettingsCategory};
use crate::{MainWindow, TabletSettingModel};
use log::{debug, error};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::SaveManager;

// Generate helper functions for TabletSettingModel
crate::impl_setting_builders!(TabletSettingModel);

// ============================================================================
// SECTION MODEL BUILDERS
// ============================================================================

/// Build general settings model (contains disable toggle)
fn populate_general_settings(tablet: &TabletSettings) -> ModelRc<TabletSettingModel> {
    let settings = vec![make_toggle(
        "off",
        "Disable tablet",
        "Turn off the tablet device entirely",
        tablet.off,
        true,
    )];
    ModelRc::new(VecModel::from(settings))
}

/// Build mapping settings model (map-to-output, left-handed)
fn populate_mapping_settings(tablet: &TabletSettings) -> ModelRc<TabletSettingModel> {
    let settings = vec![
        make_text(
            "map_to_output",
            "Map to output",
            "Monitor name to map tablet to (e.g., DP-1)",
            &tablet.map_to_output,
            "All outputs",
            true,
        ),
        make_toggle(
            "left_handed",
            "Left-handed mode",
            "Rotate tablet 180 degrees for left-handed use",
            tablet.left_handed,
            true,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

/// Build calibration settings model (6-value matrix)
fn populate_calibration_settings(tablet: &TabletSettings) -> ModelRc<TabletSettingModel> {
    let has_calibration = tablet.calibration_matrix.is_some();
    let matrix = tablet.calibration_matrix.unwrap_or([1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);

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

/// Sync all UI models from tablet settings
pub fn sync_all_models(ui: &MainWindow, tablet: &TabletSettings) {
    ui.set_tablet_dynamic_device_off(tablet.off);
    ui.set_tablet_dynamic_general_settings(populate_general_settings(tablet));
    ui.set_tablet_dynamic_mapping_settings(populate_mapping_settings(tablet));
    ui.set_tablet_dynamic_calibration_settings(populate_calibration_settings(tablet));
}

// ============================================================================
// CALLBACK SETUP
// ============================================================================

/// Set up dynamic tablet callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Generic toggle callback
    {
        let settings = Arc::clone(&settings);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_tablet_setting_toggle_changed(move |id, value| {
            let id_str = id.as_str();

            // Perform settings update and collect data for UI update
            let ui_update = match settings.lock() {
                Ok(mut s) => {
                    let tablet = &mut s.tablet;
                    let mut needs_model_refresh = false;
                    let mut update_device_off = false;

                    match id_str {
                        "off" => {
                            tablet.off = value;
                            update_device_off = true;
                        }
                        "left_handed" => {
                            tablet.left_handed = value;
                        }
                        "has_calibration" => {
                            if value {
                                // Enable calibration with default identity matrix
                                if tablet.calibration_matrix.is_none() {
                                    tablet.calibration_matrix =
                                        Some([1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
                                }
                            } else {
                                tablet.calibration_matrix = None;
                            }
                            needs_model_refresh = true;
                        }
                        _ => {
                            debug!("Unknown tablet toggle setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Tablet toggle {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Tablet);
                    save_manager.request_save();

                    // Clone data for UI update if needed
                    if needs_model_refresh || update_device_off {
                        Some((tablet.clone(), needs_model_refresh, update_device_off))
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
            if let Some((tablet_clone, needs_model_refresh, update_device_off)) = ui_update {
                if let Some(ui) = ui_weak.upgrade() {
                    if update_device_off {
                        ui.set_tablet_dynamic_device_off(value);
                    }
                    if needs_model_refresh {
                        sync_all_models(&ui, &tablet_clone);
                    }
                }
            }
        });
    }

    // Generic slider int callback (unused for tablet)
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        ui.on_tablet_setting_slider_int_changed(move |id, _value| {
            let id_str = id.as_str();
            match settings.lock() {
                Ok(mut _s) => {
                    // No int sliders for tablet
                    debug!("Unknown tablet slider int setting: {}", id_str);
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
        ui.on_tablet_setting_slider_float_changed(move |id, value| {
            let id_str = id.as_str();
            match settings.lock() {
                Ok(mut s) => {
                    let tablet = &mut s.tablet;

                    // Get or create calibration matrix
                    let matrix = tablet
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
                            debug!("Unknown tablet slider float setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Tablet slider float {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Tablet);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Generic combo callback (unused for tablet)
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        ui.on_tablet_setting_combo_changed(move |id, _index| {
            let id_str = id.as_str();
            match settings.lock() {
                Ok(mut _s) => {
                    // No combo settings for tablet
                    debug!("Unknown tablet combo setting: {}", id_str);
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
        ui.on_tablet_setting_text_changed(move |id, value| {
            let id_str = id.as_str();
            match settings.lock() {
                Ok(mut s) => {
                    let tablet = &mut s.tablet;

                    match id_str {
                        "map_to_output" => {
                            tablet.map_to_output = value.to_string();
                        }
                        _ => {
                            debug!("Unknown tablet text setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Tablet text {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Tablet);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }
}
