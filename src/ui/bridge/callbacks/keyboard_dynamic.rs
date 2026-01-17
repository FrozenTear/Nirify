//! Dynamic keyboard settings UI callbacks
//!
//! Handles keyboard configuration using model-driven dynamic UI.

use crate::config::{Settings, SettingsCategory};
use crate::constants::{REPEAT_DELAY_MAX, REPEAT_DELAY_MIN, REPEAT_RATE_MAX, REPEAT_RATE_MIN};
use crate::{KeyboardSettingModel, MainWindow};
use log::{debug, error};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::indices::{TRACK_LAYOUT_GLOBAL, TRACK_LAYOUT_WINDOW};
use super::super::macros::SaveManager;

// ============================================================================
// HELPER FUNCTIONS FOR CREATING SETTING MODELS
// ============================================================================

/// Create a toggle setting model
fn make_toggle(
    id: &str,
    label: &str,
    desc: &str,
    value: bool,
    visible: bool,
) -> KeyboardSettingModel {
    KeyboardSettingModel {
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
fn make_text(
    id: &str,
    label: &str,
    desc: &str,
    value: &str,
    placeholder: &str,
    visible: bool,
) -> KeyboardSettingModel {
    KeyboardSettingModel {
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
fn make_slider_int(
    id: &str,
    label: &str,
    desc: &str,
    value: i32,
    min: f32,
    max: f32,
    suffix: &str,
    visible: bool,
) -> KeyboardSettingModel {
    KeyboardSettingModel {
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
fn make_combo(
    id: &str,
    label: &str,
    desc: &str,
    index: i32,
    options: &[&str],
    visible: bool,
) -> KeyboardSettingModel {
    let opts: Vec<SharedString> = options.iter().map(|s| (*s).into()).collect();
    KeyboardSettingModel {
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
// SECTION POPULATION FUNCTIONS
// ============================================================================

/// Populate device control section settings
// Note: populate_device_control_settings removed - keyboard cannot be disabled in niri

/// Populate keyboard layout section settings
fn populate_layout_settings(settings: &Settings) -> ModelRc<KeyboardSettingModel> {
    let kb = &settings.keyboard;

    // Convert track_layout string to combo index
    let track_layout_index = if kb.track_layout == "window" {
        TRACK_LAYOUT_WINDOW
    } else {
        TRACK_LAYOUT_GLOBAL
    };

    let models = vec![
        make_text(
            "xkb_layout",
            "Layout",
            "Keyboard layout (e.g., us, de, fr)",
            &kb.xkb_layout,
            "us",
            true,
        ),
        make_text(
            "xkb_variant",
            "Variant",
            "Layout variant (optional)",
            &kb.xkb_variant,
            "None",
            true,
        ),
        make_text(
            "xkb_model",
            "Model",
            "Keyboard model (optional, e.g., pc104)",
            &kb.xkb_model,
            "None",
            true,
        ),
        make_text(
            "xkb_rules",
            "Rules",
            "XKB rules file (optional)",
            &kb.xkb_rules,
            "None",
            true,
        ),
        make_text(
            "xkb_options",
            "Options",
            "XKB options (e.g., ctrl:nocaps)",
            &kb.xkb_options,
            "None",
            true,
        ),
        make_combo(
            "track_layout",
            "Track layout",
            "How to track keyboard layout changes",
            track_layout_index,
            &["Global", "Per window"],
            true,
        ),
    ];
    ModelRc::new(VecModel::from(models))
}

/// Populate key repeat section settings
fn populate_repeat_settings(settings: &Settings) -> ModelRc<KeyboardSettingModel> {
    let kb = &settings.keyboard;
    let models = vec![
        make_slider_int(
            "repeat_delay",
            "Repeat delay",
            "Delay before key starts repeating",
            kb.repeat_delay,
            REPEAT_DELAY_MIN as f32,
            REPEAT_DELAY_MAX as f32,
            "ms",
            true,
        ),
        make_slider_int(
            "repeat_rate",
            "Repeat rate",
            "Characters per second when repeating",
            kb.repeat_rate,
            REPEAT_RATE_MIN as f32,
            REPEAT_RATE_MAX as f32,
            "/s",
            true,
        ),
    ];
    ModelRc::new(VecModel::from(models))
}

/// Populate other settings section
fn populate_other_settings(settings: &Settings) -> ModelRc<KeyboardSettingModel> {
    let kb = &settings.keyboard;
    let models = vec![make_toggle(
        "numlock",
        "Enable NumLock on startup",
        "Turn on NumLock when niri starts",
        kb.numlock,
        true,
    )];
    ModelRc::new(VecModel::from(models))
}

/// Populate advanced settings section
fn populate_advanced_settings(settings: &Settings) -> ModelRc<KeyboardSettingModel> {
    let kb = &settings.keyboard;
    let models = vec![make_text(
        "xkb_file",
        "Custom keymap file",
        "Path to a .xkb file (overrides layout, variant, and options)",
        &kb.xkb_file,
        "~/.config/keymap.xkb",
        true,
    )];
    ModelRc::new(VecModel::from(models))
}

// ============================================================================
// UI SYNCHRONIZATION
// ============================================================================

/// Sync all keyboard settings models to the UI
fn sync_all_models(ui: &MainWindow, settings: &Settings) {
    // Note: Keyboard cannot be disabled in niri, so no "off" or device control settings
    ui.set_keyboard_dynamic_layout_settings(populate_layout_settings(settings));
    ui.set_keyboard_dynamic_repeat_settings(populate_repeat_settings(settings));
    ui.set_keyboard_dynamic_other_settings(populate_other_settings(settings));
    ui.set_keyboard_dynamic_advanced_settings(populate_advanced_settings(settings));
}

// ============================================================================
// CALLBACK SETUP
// ============================================================================

/// Set up dynamic keyboard settings callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Toggle callback handler
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        let ui_weak = ui.as_weak();
        ui.on_keyboard_dynamic_setting_toggle_changed(move |id, value| {
            let id_str = id.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    let needs_model_refresh = false;

                    match id_str.as_str() {
                        // Note: Keyboard cannot be disabled in niri, so no "off" case
                        "numlock" => {
                            s.keyboard.numlock = value;
                            debug!("NumLock on startup = {}", value);
                        }
                        _ => {
                            debug!("Unknown keyboard toggle setting: {}", id_str);
                            return;
                        }
                    }

                    // Refresh models if visibility might have changed (e.g., off toggle)
                    if needs_model_refresh {
                        if let Some(ui) = ui_weak.upgrade() {
                            sync_all_models(&ui, &s);
                        }
                    }

                    save_manager.mark_dirty(SettingsCategory::Keyboard);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Slider int callback handler
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        ui.on_keyboard_dynamic_setting_slider_int_changed(move |id, value| {
            let id_str = id.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    match id_str.as_str() {
                        "repeat_delay" => {
                            s.keyboard.repeat_delay =
                                value.clamp(REPEAT_DELAY_MIN, REPEAT_DELAY_MAX);
                            debug!("Repeat delay = {}ms", s.keyboard.repeat_delay);
                        }
                        "repeat_rate" => {
                            s.keyboard.repeat_rate = value.clamp(REPEAT_RATE_MIN, REPEAT_RATE_MAX);
                            debug!("Repeat rate = {}/s", s.keyboard.repeat_rate);
                        }
                        _ => {
                            debug!("Unknown keyboard slider int setting: {}", id_str);
                            return;
                        }
                    }

                    save_manager.mark_dirty(SettingsCategory::Keyboard);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Slider float callback handler (not currently used for keyboard, but included for completeness)
    {
        let settings = Arc::clone(&settings);
        let _save_manager = Rc::clone(&save_manager);
        ui.on_keyboard_dynamic_setting_slider_float_changed(move |id, _value| {
            let id_str = id.to_string();
            match settings.lock() {
                Ok(_s) => {
                    // No float sliders in keyboard settings currently
                    debug!("Unknown keyboard slider float setting: {}", id_str);
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Combo callback handler
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        ui.on_keyboard_dynamic_setting_combo_changed(move |id, index| {
            let id_str = id.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    match id_str.as_str() {
                        "track_layout" => {
                            s.keyboard.track_layout = if index == TRACK_LAYOUT_GLOBAL {
                                String::from("global")
                            } else {
                                String::from("window")
                            };
                            debug!("Track layout = {}", s.keyboard.track_layout);
                        }
                        _ => {
                            debug!("Unknown keyboard combo setting: {}", id_str);
                            return;
                        }
                    }

                    save_manager.mark_dirty(SettingsCategory::Keyboard);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Text callback handler
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        ui.on_keyboard_dynamic_setting_text_changed(move |id, value| {
            let id_str = id.to_string();
            let value_str = value.to_string();

            match settings.lock() {
                Ok(mut s) => {
                    match id_str.as_str() {
                        "xkb_layout" => {
                            s.keyboard.xkb_layout = value_str.clone();
                            debug!("XKB layout = {}", value_str);
                        }
                        "xkb_variant" => {
                            s.keyboard.xkb_variant = value_str.clone();
                            debug!("XKB variant = {}", value_str);
                        }
                        "xkb_model" => {
                            s.keyboard.xkb_model = value_str.clone();
                            debug!("XKB model = {}", value_str);
                        }
                        "xkb_rules" => {
                            s.keyboard.xkb_rules = value_str.clone();
                            debug!("XKB rules = {}", value_str);
                        }
                        "xkb_options" => {
                            s.keyboard.xkb_options = value_str.clone();
                            debug!("XKB options = {}", value_str);
                        }
                        "xkb_file" => {
                            s.keyboard.xkb_file = value_str.clone();
                            debug!("XKB file = {}", value_str);
                        }
                        _ => {
                            debug!("Unknown keyboard text setting: {}", id_str);
                            return;
                        }
                    }

                    save_manager.mark_dirty(SettingsCategory::Keyboard);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }
}

// ============================================================================
// PUBLIC SYNC FUNCTIONS
// ============================================================================

/// Public function to sync all keyboard models to UI
///
/// Called from sync.rs to populate the dynamic keyboard page
pub fn sync_keyboard_models(ui: &MainWindow, settings: &Settings) {
    sync_all_models(ui, settings);
}
