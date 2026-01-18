//! Dynamic touchpad UI callbacks
//!
//! Handles touchpad configuration using model-driven dynamic UI.
//! This provides the same functionality as touchpad.rs but uses a generic
//! callback approach with setting IDs instead of individual callbacks.

use crate::config::models::TouchpadSettings;
use crate::config::{Settings, SettingsCategory};
use crate::constants::{ACCEL_SPEED_MAX, ACCEL_SPEED_MIN, SCROLL_FACTOR_MAX, SCROLL_FACTOR_MIN};
use crate::types::{AccelProfile, ClickMethod, ScrollMethod, TapButtonMap};
use crate::{MainWindow, TouchpadSettingModel};
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
) -> TouchpadSettingModel {
    TouchpadSettingModel {
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
) -> TouchpadSettingModel {
    TouchpadSettingModel {
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
) -> TouchpadSettingModel {
    TouchpadSettingModel {
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
) -> TouchpadSettingModel {
    let opts: Vec<SharedString> = options.iter().map(|s| (*s).into()).collect();
    TouchpadSettingModel {
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
) -> TouchpadSettingModel {
    TouchpadSettingModel {
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

/// Build device control settings model
fn populate_device_control_settings(touchpad: &TouchpadSettings) -> ModelRc<TouchpadSettingModel> {
    let settings = vec![make_toggle(
        "off",
        "Disable touchpad",
        "Turn off the touchpad device entirely",
        touchpad.off,
        true,
    )];
    ModelRc::new(VecModel::from(settings))
}

/// Build tap settings model
fn populate_tap_settings(touchpad: &TouchpadSettings) -> ModelRc<TouchpadSettingModel> {
    let tap_button_map_index = touchpad.tap_button_map.to_index();

    let settings = vec![
        make_toggle(
            "tap",
            "Enable tap to click",
            "Tap the touchpad to click",
            touchpad.tap,
            true,
        ),
        make_combo(
            "tap_button_map",
            "Tap button mapping",
            "Button assignment for multi-finger taps",
            tap_button_map_index,
            &["Left-Right-Middle", "Left-Middle-Right"],
            touchpad.tap,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

/// Build scroll settings model
fn populate_scroll_settings(touchpad: &TouchpadSettings) -> ModelRc<TouchpadSettingModel> {
    let scroll_method_index = touchpad.scroll_method.to_index();
    let scroll_factor_display = (touchpad.scroll_factor * 100.0) as f32;
    let is_on_button_down = matches!(touchpad.scroll_method, ScrollMethod::OnButtonDown);

    let settings = vec![
        make_toggle(
            "natural_scroll",
            "Natural scrolling",
            "Scroll content in the direction of finger movement",
            touchpad.natural_scroll,
            true,
        ),
        make_combo(
            "scroll_method",
            "Scroll method",
            "How to scroll with the touchpad",
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
            touchpad.scroll_button.unwrap_or(0),
            0.0,
            999.0,
            "",
            is_on_button_down,
        ),
        make_toggle(
            "scroll_button_lock",
            "Lock scroll mode",
            "Toggle scrolling without holding the button",
            touchpad.scroll_button_lock,
            is_on_button_down,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

/// Build click settings model
fn populate_click_settings(touchpad: &TouchpadSettings) -> ModelRc<TouchpadSettingModel> {
    let click_method_index = touchpad.click_method.to_index();

    let settings = vec![
        make_combo(
            "click_method",
            "Click method",
            "How physical clicks are detected",
            click_method_index,
            &["Button areas", "Clickfinger"],
            true,
        ),
        make_toggle(
            "middle_emulation",
            "Middle click emulation",
            "Emulate middle click with left+right click",
            touchpad.middle_emulation,
            true,
        ),
        make_toggle(
            "left_handed",
            "Left-handed mode",
            "Swap left and right buttons",
            touchpad.left_handed,
            true,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

/// Build palm rejection settings model
fn populate_palm_rejection_settings(touchpad: &TouchpadSettings) -> ModelRc<TouchpadSettingModel> {
    let settings = vec![
        make_toggle(
            "dwt",
            "Disable while typing (DWT)",
            "Prevents accidental touchpad input while typing on the keyboard",
            touchpad.dwt,
            true,
        ),
        make_toggle(
            "dwtp",
            "Disable while trackpointing (DWTP)",
            "Prevents accidental touchpad input while using the trackpoint (ThinkPad-style pointing stick)",
            touchpad.dwtp,
            true,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

/// Build drag settings model
fn populate_drag_settings(touchpad: &TouchpadSettings) -> ModelRc<TouchpadSettingModel> {
    let settings = vec![
        make_toggle(
            "drag",
            "Enable tap-and-drag",
            "Tap and hold to drag items",
            touchpad.drag,
            true,
        ),
        make_toggle(
            "drag_lock",
            "Drag lock",
            "Continue dragging after lifting finger briefly",
            touchpad.drag_lock,
            touchpad.drag,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

/// Build acceleration settings model
fn populate_acceleration_settings(touchpad: &TouchpadSettings) -> ModelRc<TouchpadSettingModel> {
    let accel_profile_index = touchpad.accel_profile.to_index();
    // accel_speed is -1.0 to 1.0, display as percentage -100 to 100
    let accel_speed_display = (touchpad.accel_speed * 100.0) as f32;

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

/// Build other settings model
fn populate_other_settings(touchpad: &TouchpadSettings) -> ModelRc<TouchpadSettingModel> {
    let settings = vec![make_toggle(
        "disabled_on_external_mouse",
        "Disable on external mouse",
        "Disable touchpad when an external mouse is connected",
        touchpad.disabled_on_external_mouse,
        true,
    )];
    ModelRc::new(VecModel::from(settings))
}

// ============================================================================
// SYNC ALL MODELS
// ============================================================================

/// Sync all UI models from touchpad settings
pub fn sync_all_models(ui: &MainWindow, touchpad: &TouchpadSettings) {
    ui.set_touchpad_dynamic_device_off(touchpad.off);
    ui.set_touchpad_dynamic_device_control_settings(populate_device_control_settings(touchpad));
    ui.set_touchpad_dynamic_tap_settings(populate_tap_settings(touchpad));
    ui.set_touchpad_dynamic_scroll_settings(populate_scroll_settings(touchpad));
    ui.set_touchpad_dynamic_click_settings(populate_click_settings(touchpad));
    ui.set_touchpad_dynamic_palm_rejection_settings(populate_palm_rejection_settings(touchpad));
    ui.set_touchpad_dynamic_drag_settings(populate_drag_settings(touchpad));
    ui.set_touchpad_dynamic_acceleration_settings(populate_acceleration_settings(touchpad));
    ui.set_touchpad_dynamic_other_settings(populate_other_settings(touchpad));
}

// ============================================================================
// CALLBACK SETUP
// ============================================================================

/// Set up dynamic touchpad callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Generic toggle callback
    {
        let settings = Arc::clone(&settings);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_touchpad_setting_toggle_changed(move |id, value| {
            let id_str = id.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    let touchpad = &mut s.touchpad;
                    let mut needs_model_refresh = false;

                    match id_str.as_str() {
                        "off" => {
                            touchpad.off = value;
                            // Update device_off property for section visibility
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_touchpad_dynamic_device_off(value);
                            }
                        }
                        "tap" => {
                            touchpad.tap = value;
                            needs_model_refresh = true; // tap_button_map visibility depends on this
                        }
                        "natural_scroll" => {
                            touchpad.natural_scroll = value;
                        }
                        "middle_emulation" => {
                            touchpad.middle_emulation = value;
                        }
                        "left_handed" => {
                            touchpad.left_handed = value;
                        }
                        "dwt" => {
                            touchpad.dwt = value;
                        }
                        "dwtp" => {
                            touchpad.dwtp = value;
                        }
                        "drag" => {
                            touchpad.drag = value;
                            needs_model_refresh = true; // drag_lock visibility depends on this
                        }
                        "drag_lock" => {
                            touchpad.drag_lock = value;
                        }
                        "disabled_on_external_mouse" => {
                            touchpad.disabled_on_external_mouse = value;
                        }
                        "scroll_button_lock" => {
                            touchpad.scroll_button_lock = value;
                        }
                        _ => {
                            debug!("Unknown touchpad toggle setting: {}", id_str);
                            return;
                        }
                    }

                    // Refresh models if visibility changed
                    if needs_model_refresh {
                        if let Some(ui) = ui_weak.upgrade() {
                            sync_all_models(&ui, touchpad);
                        }
                    }

                    debug!("Touchpad toggle {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Touchpad);
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
        ui.on_touchpad_setting_slider_int_changed(move |id, value| {
            let id_str = id.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    let touchpad = &mut s.touchpad;

                    match id_str.as_str() {
                        "scroll_button" => {
                            touchpad.scroll_button = if value == 0 { None } else { Some(value) };
                        }
                        _ => {
                            debug!("Unknown touchpad slider int setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Touchpad slider int {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Touchpad);
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
        ui.on_touchpad_setting_slider_float_changed(move |id, value| {
            let id_str = id.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    let touchpad = &mut s.touchpad;

                    match id_str.as_str() {
                        "scroll_factor" => {
                            // Convert from percentage (10-1000) to actual value (0.1-10.0)
                            let actual_value =
                                (value as f64 / 100.0).clamp(SCROLL_FACTOR_MIN, SCROLL_FACTOR_MAX);
                            touchpad.scroll_factor = actual_value;
                        }
                        "accel_speed" => {
                            // Convert from percentage (-100 to 100) to actual value (-1.0 to 1.0)
                            let actual_value =
                                (value as f64 / 100.0).clamp(ACCEL_SPEED_MIN, ACCEL_SPEED_MAX);
                            touchpad.accel_speed = actual_value;
                        }
                        _ => {
                            debug!("Unknown touchpad slider float setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Touchpad slider float {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Touchpad);
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
        ui.on_touchpad_setting_combo_changed(move |id, index| {
            let id_str = id.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    let touchpad = &mut s.touchpad;
                    let mut needs_model_refresh = false;

                    match id_str.as_str() {
                        "tap_button_map" => {
                            touchpad.tap_button_map = TapButtonMap::from_index(index);
                        }
                        "scroll_method" => {
                            touchpad.scroll_method = ScrollMethod::from_index(index);
                            needs_model_refresh = true; // scroll_button visibility depends on this
                        }
                        "click_method" => {
                            touchpad.click_method = ClickMethod::from_index(index);
                        }
                        "accel_profile" => {
                            touchpad.accel_profile = AccelProfile::from_index(index);
                        }
                        _ => {
                            debug!("Unknown touchpad combo setting: {}", id_str);
                            return;
                        }
                    }

                    // Refresh models if visibility changed
                    if needs_model_refresh {
                        if let Some(ui) = ui_weak.upgrade() {
                            sync_all_models(&ui, touchpad);
                        }
                    }

                    debug!("Touchpad combo {} = {}", id_str, index);
                    save_manager.mark_dirty(SettingsCategory::Touchpad);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Generic text callback (currently unused for touchpad, but included for completeness)
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        ui.on_touchpad_setting_text_changed(move |id, _value| {
            let id_str = id.to_string();
            match settings.lock() {
                Ok(mut _s) => {
                    // No text settings for touchpad currently
                    debug!("Unknown touchpad text setting: {}", id_str);
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
            // Mark dirty and save would go here if we had text settings
            let _ = save_manager;
        });
    }
}
