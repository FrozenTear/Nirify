//! Dynamic Mouse UI callbacks
//!
//! Handles mouse configuration using model-driven dynamic UI.
//! This provides the same functionality as mouse.rs but uses a generic
//! callback approach with setting IDs instead of individual callbacks.

use crate::config::models::MouseSettings;
use crate::config::{Settings, SettingsCategory};
use crate::constants::{ACCEL_SPEED_MAX, ACCEL_SPEED_MIN, SCROLL_FACTOR_MAX, SCROLL_FACTOR_MIN};
use crate::types::{AccelProfile, ScrollMethod};
use crate::{MainWindow, MouseSettingModel};
use log::{debug, error};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::SaveManager;

// Generate helper functions for MouseSettingModel
impl_setting_builders!(MouseSettingModel);

// ============================================================================
// SECTION MODEL BUILDERS
// ============================================================================

/// Build device control settings model
fn populate_device_control_settings(mouse: &MouseSettings) -> ModelRc<MouseSettingModel> {
    let settings = vec![make_toggle(
        "off",
        "Disable mouse",
        "Turn off the mouse device entirely",
        mouse.off,
        true,
    )];
    ModelRc::new(VecModel::from(settings))
}

/// Build basic settings model
fn populate_basic_settings(mouse: &MouseSettings) -> ModelRc<MouseSettingModel> {
    let settings = vec![
        make_toggle(
            "natural_scroll",
            "Natural scrolling",
            "Scroll content in the direction of finger movement",
            mouse.natural_scroll,
            true,
        ),
        make_toggle(
            "left_handed",
            "Left-handed mode",
            "Swap left and right mouse buttons",
            mouse.left_handed,
            true,
        ),
        make_toggle(
            "middle_emulation",
            "Middle click emulation",
            "Emulate middle click by pressing left and right buttons together",
            mouse.middle_emulation,
            true,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

/// Build acceleration settings model
fn populate_acceleration_settings(mouse: &MouseSettings) -> ModelRc<MouseSettingModel> {
    let accel_profile_index = mouse.accel_profile.to_index();
    // accel_speed is -1.0 to 1.0, display as percentage -100 to 100
    let accel_speed_display = (mouse.accel_speed * 100.0) as f32;

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
fn populate_scrolling_settings(mouse: &MouseSettings) -> ModelRc<MouseSettingModel> {
    let scroll_method_index = mouse.scroll_method.to_index();
    // scroll_factor is 0.1 to 10.0, display as percentage 10 to 1000
    let scroll_factor_display = (mouse.scroll_factor * 100.0) as f32;
    let is_on_button_down = matches!(mouse.scroll_method, ScrollMethod::OnButtonDown);

    let settings = vec![
        make_combo(
            "scroll_method",
            "Scroll method",
            "How to scroll with the mouse",
            scroll_method_index,
            &["Two finger", "Edge", "On button down", "Disabled"],
            true,
        ),
        make_slider_float(
            "scroll_factor",
            "Scroll factor",
            "Scroll speed multiplier",
            scroll_factor_display,
            (SCROLL_FACTOR_MIN * 100.0) as f32,
            (SCROLL_FACTOR_MAX * 100.0) as f32,
            "%",
            true,
        ),
        make_slider_int(
            "scroll_button",
            "Scroll button",
            "Button code (Middle=274, Side=275)",
            mouse.scroll_button.unwrap_or(0),
            0.0,
            999.0,
            "",
            is_on_button_down,
        ),
        make_toggle(
            "scroll_button_lock",
            "Lock scroll mode",
            "Toggle scrolling without holding the button",
            mouse.scroll_button_lock,
            is_on_button_down,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

// ============================================================================
// SYNC ALL MODELS
// ============================================================================

/// Sync all UI models from mouse settings
pub fn sync_all_models(ui: &MainWindow, mouse: &MouseSettings) {
    ui.set_mouse_dynamic_device_off(mouse.off);
    ui.set_mouse_dynamic_device_settings(populate_device_control_settings(mouse));
    ui.set_mouse_dynamic_basic_settings(populate_basic_settings(mouse));
    ui.set_mouse_dynamic_acceleration_settings(populate_acceleration_settings(mouse));
    ui.set_mouse_dynamic_scrolling_settings(populate_scrolling_settings(mouse));
}

// ============================================================================
// CALLBACK SETUP
// ============================================================================

/// Set up dynamic mouse callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Generic toggle callback
    {
        let settings = Arc::clone(&settings);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_mouse_setting_toggle_changed(move |id, value| {
            let id_str = id.as_str();

            // Track if this is the "off" toggle for UI update after lock release
            let (is_off_toggle, mouse_clone) = match settings.lock() {
                Ok(mut s) => {
                    let mouse = &mut s.mouse;
                    let mut is_off = false;
                    let needs_model_refresh = false;

                    match id_str {
                        "off" => {
                            mouse.off = value;
                            is_off = true;
                        }
                        "natural_scroll" => {
                            mouse.natural_scroll = value;
                        }
                        "left_handed" => {
                            mouse.left_handed = value;
                        }
                        "middle_emulation" => {
                            mouse.middle_emulation = value;
                        }
                        "scroll_button_lock" => {
                            mouse.scroll_button_lock = value;
                        }
                        _ => {
                            debug!("Unknown mouse toggle setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Mouse toggle {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Mouse);
                    save_manager.request_save();

                    // Clone data for UI update if needed
                    let clone = if needs_model_refresh {
                        Some(mouse.clone())
                    } else {
                        None
                    };
                    (is_off, clone)
                }
                Err(e) => {
                    error!("Settings lock error: {}", e);
                    return;
                }
            };

            // UI updates happen after lock is released
            if let Some(ui) = ui_weak.upgrade() {
                if is_off_toggle {
                    ui.set_mouse_dynamic_device_off(value);
                }
                if let Some(mouse) = mouse_clone {
                    sync_all_models(&ui, &mouse);
                }
            }
        });
    }

    // Generic slider int callback
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        ui.on_mouse_setting_slider_int_changed(move |id, value| {
            let id_str = id.as_str();
            match settings.lock() {
                Ok(mut s) => {
                    let mouse = &mut s.mouse;

                    match id_str {
                        "scroll_button" => {
                            mouse.scroll_button = if value == 0 { None } else { Some(value) };
                        }
                        _ => {
                            debug!("Unknown mouse slider int setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Mouse slider int {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Mouse);
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
        ui.on_mouse_setting_slider_float_changed(move |id, value| {
            let id_str = id.as_str();
            match settings.lock() {
                Ok(mut s) => {
                    let mouse = &mut s.mouse;

                    match id_str {
                        "accel_speed" => {
                            // Convert from percentage (-100 to 100) to actual value (-1.0 to 1.0)
                            let actual_value =
                                (value as f64 / 100.0).clamp(ACCEL_SPEED_MIN, ACCEL_SPEED_MAX);
                            mouse.accel_speed = actual_value;
                        }
                        "scroll_factor" => {
                            // Convert from percentage (10-1000) to actual value (0.1-10.0)
                            let actual_value =
                                (value as f64 / 100.0).clamp(SCROLL_FACTOR_MIN, SCROLL_FACTOR_MAX);
                            mouse.scroll_factor = actual_value;
                        }
                        _ => {
                            debug!("Unknown mouse slider float setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Mouse slider float {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Mouse);
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
        ui.on_mouse_setting_combo_changed(move |id, index| {
            let id_str = id.as_str();

            // Clone data needed for UI update, then release lock before UI operations
            let mouse_clone = match settings.lock() {
                Ok(mut s) => {
                    let mouse = &mut s.mouse;
                    let mut needs_model_refresh = false;

                    match id_str {
                        "accel_profile" => {
                            mouse.accel_profile = AccelProfile::from_index(index);
                        }
                        "scroll_method" => {
                            mouse.scroll_method = ScrollMethod::from_index(index);
                            needs_model_refresh = true; // scroll_button visibility depends on this
                        }
                        _ => {
                            debug!("Unknown mouse combo setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Mouse combo {} = {}", id_str, index);
                    save_manager.mark_dirty(SettingsCategory::Mouse);
                    save_manager.request_save();

                    // Clone data for UI update if needed
                    if needs_model_refresh {
                        Some(mouse.clone())
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
            if let Some(mouse) = mouse_clone {
                if let Some(ui) = ui_weak.upgrade() {
                    sync_all_models(&ui, &mouse);
                }
            }
        });
    }

    // Generic text callback (currently unused for mouse, but included for completeness)
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        ui.on_mouse_setting_text_changed(move |id, _value| {
            let id_str = id.as_str();
            match settings.lock() {
                Ok(mut _s) => {
                    // No text settings for mouse currently
                    debug!("Unknown mouse text setting: {}", id_str);
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
            // Mark dirty and save would go here if we had text settings
            let _ = save_manager;
        });
    }
}
