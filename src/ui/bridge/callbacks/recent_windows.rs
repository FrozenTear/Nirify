//! Dynamic Recent Windows UI callbacks
//!
//! Handles recent windows (Alt-Tab) settings using model-driven dynamic UI.
//! This provides the same functionality as recent_windows.rs but uses a generic
//! callback approach with setting IDs instead of individual callbacks.

use crate::config::models::RecentWindowsSettings;
use crate::config::{Settings, SettingsCategory};
use crate::types::Color;
use crate::{MainWindow, RecentWindowsSettingModel};
use log::{debug, error};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::converters::{color_to_slint_color, slint_color_to_color};
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
) -> RecentWindowsSettingModel {
    RecentWindowsSettingModel {
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
) -> RecentWindowsSettingModel {
    RecentWindowsSettingModel {
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
) -> RecentWindowsSettingModel {
    RecentWindowsSettingModel {
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

fn make_color(
    id: &str,
    label: &str,
    desc: &str,
    color: &Color,
    visible: bool,
) -> RecentWindowsSettingModel {
    RecentWindowsSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 4,
        text_value: color.to_hex().into(),
        color_value: color_to_slint_color(color),
        placeholder: "#RRGGBB".into(),
        visible,
        ..Default::default()
    }
}

// ============================================================================
// SECTION MODEL BUILDERS
// ============================================================================

/// Build general settings model (enable toggle and timing)
fn populate_general_settings(rw: &RecentWindowsSettings) -> ModelRc<RecentWindowsSettingModel> {
    let enabled = !rw.off;

    let settings = vec![
        make_toggle(
            "enabled",
            "Enable recent windows switcher",
            "When disabled, the window switcher will not appear",
            enabled,
            true,
        ),
        make_slider_int(
            "debounce_ms",
            "Debounce delay",
            "Time before window is added to recent list (ms)",
            rw.debounce_ms,
            0.0,
            1000.0,
            "ms",
            enabled,
        ),
        make_slider_int(
            "open_delay_ms",
            "Open delay",
            "Time before the switcher UI appears (ms)",
            rw.open_delay_ms,
            0.0,
            1000.0,
            "ms",
            enabled,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

/// Build highlight settings model
fn populate_highlight_settings(rw: &RecentWindowsSettings) -> ModelRc<RecentWindowsSettingModel> {
    let settings = vec![
        make_color(
            "highlight_active_color",
            "Active window color",
            "Highlight color for the currently selected window",
            &rw.highlight.active_color,
            true,
        ),
        make_color(
            "highlight_urgent_color",
            "Urgent window color",
            "Highlight color for urgent/attention-requesting windows",
            &rw.highlight.urgent_color,
            true,
        ),
        make_slider_int(
            "highlight_padding",
            "Padding",
            "Space between window and highlight border",
            rw.highlight.padding,
            0.0,
            32.0,
            "px",
            true,
        ),
        make_slider_int(
            "highlight_corner_radius",
            "Corner radius",
            "Roundness of the highlight corners",
            rw.highlight.corner_radius,
            0.0,
            32.0,
            "px",
            true,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

/// Build preview settings model
fn populate_preview_settings(rw: &RecentWindowsSettings) -> ModelRc<RecentWindowsSettingModel> {
    // max_scale is 0.0-1.0, display as percentage 0-100
    let max_scale_display = (rw.previews.max_scale * 100.0) as f32;

    let settings = vec![
        make_slider_int(
            "previews_max_height",
            "Maximum height",
            "Maximum height of window previews in logical pixels",
            rw.previews.max_height,
            50.0,
            500.0,
            "px",
            true,
        ),
        make_slider_float(
            "previews_max_scale",
            "Maximum scale",
            "Maximum scale factor for window previews",
            max_scale_display,
            10.0,
            100.0,
            "%",
            true,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

// ============================================================================
// SYNC ALL MODELS
// ============================================================================

/// Sync all UI models from recent windows settings
pub fn sync_recent_windows_models(ui: &MainWindow, rw: &RecentWindowsSettings) {
    let enabled = !rw.off;
    ui.set_recent_windows_dynamic_feature_enabled(enabled);
    ui.set_recent_windows_dynamic_general_settings(populate_general_settings(rw));
    ui.set_recent_windows_dynamic_highlight_settings(populate_highlight_settings(rw));
    ui.set_recent_windows_dynamic_preview_settings(populate_preview_settings(rw));
}

// ============================================================================
// CALLBACK SETUP
// ============================================================================

/// Set up dynamic recent windows callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Generic toggle callback
    {
        let settings = Arc::clone(&settings);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_recent_windows_dynamic_setting_toggle_changed(move |id, value| {
            let id_str = id.as_str();

            // Clone data needed for UI update, then release lock before UI operations
            let rw_clone = match settings.lock() {
                Ok(mut s) => {
                    let rw = &mut s.recent_windows;
                    #[allow(unused_assignments)]
                    let mut needs_model_refresh = false;

                    match id_str {
                        "enabled" => {
                            rw.off = !value;
                            needs_model_refresh = true;
                        }
                        _ => {
                            debug!("Unknown recent windows toggle setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Recent windows toggle {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::RecentWindows);
                    save_manager.request_save();

                    // Clone data for UI update if needed
                    if needs_model_refresh {
                        Some(rw.clone())
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
            if let Some(rw) = rw_clone {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_recent_windows_dynamic_feature_enabled(value);
                    sync_recent_windows_models(&ui, &rw);
                }
            }
        });
    }

    // Generic slider int callback
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        ui.on_recent_windows_dynamic_setting_slider_int_changed(move |id, value| {
            let id_str = id.as_str();
            match settings.lock() {
                Ok(mut s) => {
                    let rw = &mut s.recent_windows;

                    match id_str {
                        "debounce_ms" => {
                            rw.debounce_ms = value.max(0);
                        }
                        "open_delay_ms" => {
                            rw.open_delay_ms = value.max(0);
                        }
                        "highlight_padding" => {
                            rw.highlight.padding = value.clamp(0, 32);
                        }
                        "highlight_corner_radius" => {
                            rw.highlight.corner_radius = value.clamp(0, 32);
                        }
                        "previews_max_height" => {
                            rw.previews.max_height = value.clamp(50, 500);
                        }
                        _ => {
                            debug!("Unknown recent windows slider int setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Recent windows slider int {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::RecentWindows);
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
        ui.on_recent_windows_dynamic_setting_slider_float_changed(move |id, value| {
            let id_str = id.as_str();
            match settings.lock() {
                Ok(mut s) => {
                    let rw = &mut s.recent_windows;

                    match id_str {
                        "previews_max_scale" => {
                            // Convert from percentage (10-100) to actual value (0.1-1.0)
                            let actual_value = (value as f64 / 100.0).clamp(0.1, 1.0);
                            rw.previews.max_scale = actual_value;
                        }
                        _ => {
                            debug!("Unknown recent windows slider float setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Recent windows slider float {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::RecentWindows);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Generic text callback (handles color hex input)
    {
        let settings = Arc::clone(&settings);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_recent_windows_dynamic_setting_text_changed(move |id, value| {
            let id_str = id.as_str();
            let value_str = value.to_string();

            // Try to parse as color for color fields
            let color_result = Color::from_hex(&value_str);

            // Clone data needed for UI update, then release lock before UI operations
            let rw_clone = match settings.lock() {
                Ok(mut s) => {
                    let rw = &mut s.recent_windows;
                    let mut needs_refresh = false;

                    match id_str {
                        "highlight_active_color" => {
                            if let Some(color) = color_result {
                                rw.highlight.active_color = color;
                                needs_refresh = true;
                            }
                        }
                        "highlight_urgent_color" => {
                            if let Some(color) = color_result {
                                rw.highlight.urgent_color = color;
                                needs_refresh = true;
                            }
                        }
                        _ => {
                            debug!("Unknown recent windows text setting: {}", id_str);
                            return;
                        }
                    }

                    // Only save if we had a valid color
                    if color_result.is_some() {
                        debug!("Recent windows text {} = {}", id_str, value_str);
                        save_manager.mark_dirty(SettingsCategory::RecentWindows);
                        save_manager.request_save();
                    }

                    // Clone data for UI update if needed
                    if needs_refresh {
                        Some(rw.clone())
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
            if let Some(rw) = rw_clone {
                if let Some(ui) = ui_weak.upgrade() {
                    sync_recent_windows_models(&ui, &rw);
                }
            }
        });
    }

    // Color callback (from swatch picker)
    {
        let settings = Arc::clone(&settings);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_recent_windows_dynamic_setting_color_changed(move |id, color| {
            let id_str = id.as_str();
            let rust_color = slint_color_to_color(color);

            // Clone data needed for UI update, then release lock before UI operations
            let rw_clone = match settings.lock() {
                Ok(mut s) => {
                    let rw = &mut s.recent_windows;

                    match id_str {
                        "highlight_active_color" => {
                            rw.highlight.active_color = rust_color;
                        }
                        "highlight_urgent_color" => {
                            rw.highlight.urgent_color = rust_color;
                        }
                        _ => {
                            debug!("Unknown recent windows color setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Recent windows color {} changed", id_str);
                    save_manager.mark_dirty(SettingsCategory::RecentWindows);
                    save_manager.request_save();

                    // Clone data for UI update
                    rw.clone()
                }
                Err(e) => {
                    error!("Settings lock error: {}", e);
                    return;
                }
            };

            // UI updates happen after lock is released
            if let Some(ui) = ui_weak.upgrade() {
                sync_recent_windows_models(&ui, &rw_clone);
            }
        });
    }
}
