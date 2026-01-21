//! Dynamic Behavior settings UI callbacks
//!
//! Handles behavior configuration using model-driven dynamic UI.

use crate::config::models::ColumnWidthType;
use crate::config::{Settings, SettingsCategory};
use crate::constants::{
    COLUMN_FIXED_MAX, COLUMN_FIXED_MIN, COLUMN_PROPORTION_MAX, COLUMN_PROPORTION_MIN,
    STRUT_SIZE_MAX, STRUT_SIZE_MIN,
};
use crate::types::{CenterFocusedColumn, ModKey, WarpMouseMode};
use crate::BehaviorSettingModel;
use crate::MainWindow;
use log::{debug, error};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::SaveManager;

// ============================================================================
// SECTION ENUM FOR SELECTIVE SYNC
// ============================================================================

/// Identifies which section of behavior settings to refresh
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BehaviorSection {
    Focus,
    Workspace,
    WindowSize,
    Struts,
    Modifiers,
    Power,
    All,
}

// ============================================================================
// HELPER FUNCTIONS FOR CREATING SETTING MODELS
// ============================================================================

fn make_toggle(
    id: &str,
    label: &str,
    desc: &str,
    value: bool,
    visible: bool,
) -> BehaviorSettingModel {
    BehaviorSettingModel {
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
) -> BehaviorSettingModel {
    BehaviorSettingModel {
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
) -> BehaviorSettingModel {
    BehaviorSettingModel {
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
) -> BehaviorSettingModel {
    let opts: Vec<SharedString> = options.iter().map(|s| (*s).into()).collect();
    BehaviorSettingModel {
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
) -> BehaviorSettingModel {
    BehaviorSettingModel {
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
// POPULATE SECTION MODELS
// ============================================================================

/// Populate focus settings model
fn populate_focus_settings(settings: &Settings) -> ModelRc<BehaviorSettingModel> {
    let b = &settings.behavior;

    let warp_enabled = !matches!(b.warp_mouse_to_focus, WarpMouseMode::Off);

    let models = vec![
        make_toggle(
            "focus_follows_mouse",
            "Focus follows mouse",
            "Windows get focus when the mouse moves over them",
            b.focus_follows_mouse,
            true,
        ),
        make_toggle(
            "warp_mouse_to_focus",
            "Warp mouse to focus",
            "Move the mouse cursor when focus changes via keyboard",
            warp_enabled,
            true,
        ),
    ];
    ModelRc::new(VecModel::from(models))
}

/// Populate workspace layout settings model
fn populate_workspace_settings(settings: &Settings) -> ModelRc<BehaviorSettingModel> {
    let b = &settings.behavior;

    let center_enabled = !matches!(b.center_focused_column, CenterFocusedColumn::Never);

    let models = vec![
        make_toggle(
            "center_focused_column",
            "Center focused column",
            "Keep the focused column centered on screen",
            center_enabled,
            true,
        ),
        make_toggle(
            "always_center_single_column",
            "Always center single column",
            "Center even when there's only one column",
            b.always_center_single_column,
            center_enabled, // Only visible when centering is enabled
        ),
    ];
    ModelRc::new(VecModel::from(models))
}

/// Populate window size settings model
fn populate_window_size_settings(settings: &Settings) -> ModelRc<BehaviorSettingModel> {
    let b = &settings.behavior;

    let is_proportion = matches!(b.default_column_width_type, ColumnWidthType::Proportion);

    let models = vec![
        make_combo(
            "default_column_width_type",
            "Width type",
            "How to calculate default window width",
            b.default_column_width_type.to_index(),
            &["Proportion", "Fixed pixels"],
            true,
        ),
        make_slider_float(
            "default_column_width_proportion",
            "Width proportion",
            "Fraction of screen width for new windows",
            b.default_column_width_proportion,
            COLUMN_PROPORTION_MIN,
            COLUMN_PROPORTION_MAX,
            "%",
            is_proportion,
        ),
        make_slider_int(
            "default_column_width_fixed",
            "Width pixels",
            "Fixed width in pixels for new windows",
            b.default_column_width_fixed as i32,
            COLUMN_FIXED_MIN,
            COLUMN_FIXED_MAX,
            "px",
            !is_proportion,
        ),
    ];
    ModelRc::new(VecModel::from(models))
}

/// Populate strut settings model
fn populate_strut_settings(settings: &Settings) -> ModelRc<BehaviorSettingModel> {
    let b = &settings.behavior;

    let models = vec![
        make_slider_int(
            "strut_left",
            "Left",
            "Reserved space on the left edge",
            b.strut_left as i32,
            STRUT_SIZE_MIN,
            STRUT_SIZE_MAX,
            "px",
            true,
        ),
        make_slider_int(
            "strut_right",
            "Right",
            "Reserved space on the right edge",
            b.strut_right as i32,
            STRUT_SIZE_MIN,
            STRUT_SIZE_MAX,
            "px",
            true,
        ),
        make_slider_int(
            "strut_top",
            "Top",
            "Reserved space at the top edge",
            b.strut_top as i32,
            STRUT_SIZE_MIN,
            STRUT_SIZE_MAX,
            "px",
            true,
        ),
        make_slider_int(
            "strut_bottom",
            "Bottom",
            "Reserved space at the bottom edge",
            b.strut_bottom as i32,
            STRUT_SIZE_MIN,
            STRUT_SIZE_MAX,
            "px",
            true,
        ),
    ];
    ModelRc::new(VecModel::from(models))
}

/// Populate modifier keys settings model
fn populate_modifier_settings(settings: &Settings) -> ModelRc<BehaviorSettingModel> {
    let b = &settings.behavior;

    let nested_enabled = b.mod_key_nested.is_some();
    let nested_index = b.mod_key_nested.map(|k| k.to_index()).unwrap_or(1); // Default to Alt

    let models = vec![
        make_combo(
            "mod_key",
            "Modifier key",
            "Primary modifier for window management shortcuts",
            b.mod_key.to_index(),
            &["Super", "Alt", "Ctrl", "Shift", "Mod3", "Mod5"],
            true,
        ),
        make_toggle(
            "mod_key_nested_enabled",
            "Custom nested modifier",
            "Use a different modifier when running inside another compositor",
            nested_enabled,
            true,
        ),
        make_combo(
            "mod_key_nested",
            "Nested modifier key",
            "Modifier to use when niri runs nested",
            nested_index,
            &["Super", "Alt", "Ctrl", "Shift", "Mod3", "Mod5"],
            nested_enabled,
        ),
    ];
    ModelRc::new(VecModel::from(models))
}

/// Populate power settings model
fn populate_power_settings(settings: &Settings) -> ModelRc<BehaviorSettingModel> {
    let b = &settings.behavior;

    let models = vec![make_toggle(
        "disable_power_key_handling",
        "System handles power button",
        "Let the system handle power button events instead of niri",
        b.disable_power_key_handling,
        true,
    )];
    ModelRc::new(VecModel::from(models))
}

// ============================================================================
// SYNC ALL MODELS
// ============================================================================

/// Sync a specific section of behavior UI models from settings
///
/// This function allows selective refresh of UI models, avoiding the overhead
/// of refreshing all sections when only one has changed.
pub fn sync_models(ui: &MainWindow, settings: &Settings, section: BehaviorSection) {
    match section {
        BehaviorSection::Focus => {
            ui.set_behavior_focus_settings(populate_focus_settings(settings));
        }
        BehaviorSection::Workspace => {
            ui.set_behavior_workspace_settings(populate_workspace_settings(settings));
        }
        BehaviorSection::WindowSize => {
            ui.set_behavior_window_size_settings(populate_window_size_settings(settings));
        }
        BehaviorSection::Struts => {
            ui.set_behavior_strut_settings(populate_strut_settings(settings));
        }
        BehaviorSection::Modifiers => {
            ui.set_behavior_modifier_settings(populate_modifier_settings(settings));
        }
        BehaviorSection::Power => {
            ui.set_behavior_power_settings(populate_power_settings(settings));
        }
        BehaviorSection::All => {
            sync_all_models(ui, settings);
        }
    }
}

/// Sync all UI models from settings
pub fn sync_all_models(ui: &MainWindow, settings: &Settings) {
    ui.set_behavior_focus_settings(populate_focus_settings(settings));
    ui.set_behavior_workspace_settings(populate_workspace_settings(settings));
    ui.set_behavior_window_size_settings(populate_window_size_settings(settings));
    ui.set_behavior_strut_settings(populate_strut_settings(settings));
    ui.set_behavior_modifier_settings(populate_modifier_settings(settings));
    ui.set_behavior_power_settings(populate_power_settings(settings));
}

// ============================================================================
// CALLBACK SETUP
// ============================================================================

/// Set up dynamic behavior callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Generic toggle callback
    {
        let settings = Arc::clone(&settings);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_behavior_setting_toggle_changed(move |id, value| {
            let id_str = id.as_str();

            // Clone data needed for UI update, then release lock before UI operations
            let refresh_info = match settings.lock() {
                Ok(mut s) => {
                    let section = match id_str {
                        "focus_follows_mouse" => {
                            s.behavior.focus_follows_mouse = value;
                            None // No UI refresh needed
                        }
                        "warp_mouse_to_focus" => {
                            s.behavior.warp_mouse_to_focus = if value {
                                WarpMouseMode::CenterXY
                            } else {
                                WarpMouseMode::Off
                            };
                            None // No UI refresh needed
                        }
                        "center_focused_column" => {
                            s.behavior.center_focused_column = if value {
                                CenterFocusedColumn::OnOverflow
                            } else {
                                CenterFocusedColumn::Never
                            };
                            Some(BehaviorSection::Workspace)
                        }
                        "always_center_single_column" => {
                            s.behavior.always_center_single_column = value;
                            None // No UI refresh needed
                        }
                        "mod_key_nested_enabled" => {
                            s.behavior.mod_key_nested =
                                if value { Some(ModKey::Alt) } else { None };
                            Some(BehaviorSection::Modifiers)
                        }
                        "disable_power_key_handling" => {
                            s.behavior.disable_power_key_handling = value;
                            None // No UI refresh needed
                        }
                        _ => {
                            debug!("Unknown behavior toggle setting: {}", id_str);
                            return;
                        }
                    };

                    debug!("Behavior toggle {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Behavior);
                    save_manager.request_save();

                    // Clone data for UI update if needed
                    section.map(|sec| (s.clone(), sec))
                }
                Err(e) => {
                    error!("Settings lock error: {}", e);
                    return;
                }
            };

            // UI updates happen after lock is released
            if let Some((s, section)) = refresh_info {
                if let Some(ui) = ui_weak.upgrade() {
                    sync_models(&ui, &s, section);
                }
            }
        });
    }

    // Generic slider int callback
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        ui.on_behavior_setting_slider_int_changed(move |id, value| {
            let id_str = id.as_str();
            match settings.lock() {
                Ok(mut s) => {
                    match id_str {
                        "strut_left" => {
                            s.behavior.strut_left =
                                (value as f32).clamp(STRUT_SIZE_MIN, STRUT_SIZE_MAX);
                        }
                        "strut_right" => {
                            s.behavior.strut_right =
                                (value as f32).clamp(STRUT_SIZE_MIN, STRUT_SIZE_MAX);
                        }
                        "strut_top" => {
                            s.behavior.strut_top =
                                (value as f32).clamp(STRUT_SIZE_MIN, STRUT_SIZE_MAX);
                        }
                        "strut_bottom" => {
                            s.behavior.strut_bottom =
                                (value as f32).clamp(STRUT_SIZE_MIN, STRUT_SIZE_MAX);
                        }
                        "default_column_width_fixed" => {
                            s.behavior.default_column_width_fixed =
                                (value as f32).clamp(COLUMN_FIXED_MIN, COLUMN_FIXED_MAX);
                        }
                        _ => {
                            debug!("Unknown behavior slider int setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Behavior slider int {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Behavior);
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
        ui.on_behavior_setting_slider_float_changed(move |id, value| {
            let id_str = id.as_str();
            match settings.lock() {
                Ok(mut s) => {
                    match id_str {
                        "default_column_width_proportion" => {
                            s.behavior.default_column_width_proportion =
                                value.clamp(COLUMN_PROPORTION_MIN, COLUMN_PROPORTION_MAX);
                        }
                        _ => {
                            debug!("Unknown behavior slider float setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Behavior slider float {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Behavior);
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
        ui.on_behavior_setting_combo_changed(move |id, index| {
            let id_str = id.as_str();

            // Clone data needed for UI update, then release lock before UI operations
            let refresh_info = match settings.lock() {
                Ok(mut s) => {
                    let section = match id_str {
                        "default_column_width_type" => {
                            s.behavior.default_column_width_type =
                                ColumnWidthType::from_index(index);
                            Some(BehaviorSection::WindowSize)
                        }
                        "mod_key" => {
                            s.behavior.mod_key = ModKey::from_index(index);
                            None // No UI refresh needed
                        }
                        "mod_key_nested" => {
                            s.behavior.mod_key_nested = Some(ModKey::from_index(index));
                            None // No UI refresh needed
                        }
                        _ => {
                            debug!("Unknown behavior combo setting: {}", id_str);
                            return;
                        }
                    };

                    debug!("Behavior combo {} = {}", id_str, index);
                    save_manager.mark_dirty(SettingsCategory::Behavior);
                    save_manager.request_save();

                    // Clone data for UI update if needed
                    section.map(|sec| (s.clone(), sec))
                }
                Err(e) => {
                    error!("Settings lock error: {}", e);
                    return;
                }
            };

            // UI updates happen after lock is released
            if let Some((s, section)) = refresh_info {
                if let Some(ui) = ui_weak.upgrade() {
                    sync_models(&ui, &s, section);
                }
            }
        });
    }
}
