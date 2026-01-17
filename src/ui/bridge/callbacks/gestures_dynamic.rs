//! Dynamic gestures UI callbacks
//!
//! Handles gesture settings using model-driven dynamic UI.

use crate::config::{Settings, SettingsCategory};
use crate::{GestureSettingModel, MainWindow};
use log::{debug, error};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::SaveManager;

// ============================================================================
// Helper functions for creating setting models
// ============================================================================

fn make_toggle(
    id: &str,
    label: &str,
    desc: &str,
    value: bool,
    visible: bool,
) -> GestureSettingModel {
    GestureSettingModel {
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
) -> GestureSettingModel {
    GestureSettingModel {
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

#[allow(dead_code)]
fn make_slider_float(
    id: &str,
    label: &str,
    desc: &str,
    value: f32,
    min: f32,
    max: f32,
    suffix: &str,
    visible: bool,
) -> GestureSettingModel {
    GestureSettingModel {
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
) -> GestureSettingModel {
    let opts: Vec<SharedString> = options.iter().map(|s| (*s).into()).collect();
    GestureSettingModel {
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
) -> GestureSettingModel {
    GestureSettingModel {
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
// Model population functions
// ============================================================================

/// Populate hot corners settings model
fn populate_hot_corners_settings(settings: &Settings) -> ModelRc<GestureSettingModel> {
    let hc = &settings.gestures.hot_corners;

    let items = vec![make_toggle(
        "hot_corners_enabled",
        "Enable hot corners",
        "Trigger overview when cursor reaches corners",
        hc.enabled,
        true,
    )];

    ModelRc::new(VecModel::from(items))
}

/// Populate DND edge scroll settings model
fn populate_dnd_edge_scroll_settings(settings: &Settings) -> ModelRc<GestureSettingModel> {
    let dnd = &settings.gestures.dnd_edge_view_scroll;

    let items = vec![
        make_toggle(
            "dnd_edge_scroll_enabled",
            "Enable edge scrolling",
            "Scroll view when dragging near edges",
            dnd.enabled,
            true,
        ),
        make_slider_int(
            "dnd_edge_scroll_trigger_width",
            "Trigger width",
            "Distance from edge to trigger scrolling",
            dnd.trigger_size,
            10.0,
            100.0,
            "px",
            dnd.enabled,
        ),
        make_slider_int(
            "dnd_edge_scroll_delay",
            "Delay",
            "Time before scrolling starts",
            dnd.delay_ms,
            0.0,
            500.0,
            "ms",
            dnd.enabled,
        ),
        make_slider_int(
            "dnd_edge_scroll_max_speed",
            "Max speed",
            "Maximum scroll speed",
            dnd.max_speed,
            100.0,
            5000.0,
            "px/s",
            dnd.enabled,
        ),
    ];

    ModelRc::new(VecModel::from(items))
}

/// Populate DND workspace switch settings model
fn populate_dnd_workspace_switch_settings(settings: &Settings) -> ModelRc<GestureSettingModel> {
    let dnd = &settings.gestures.dnd_edge_workspace_switch;

    let items = vec![
        make_toggle(
            "dnd_workspace_switch_enabled",
            "Enable workspace switching",
            "Switch workspaces when dragging in overview",
            dnd.enabled,
            true,
        ),
        make_slider_int(
            "dnd_workspace_switch_trigger_height",
            "Trigger height",
            "Distance from edge to trigger switching",
            dnd.trigger_size,
            20.0,
            150.0,
            "px",
            dnd.enabled,
        ),
        make_slider_int(
            "dnd_workspace_switch_delay",
            "Delay",
            "Time before switching starts",
            dnd.delay_ms,
            0.0,
            500.0,
            "ms",
            dnd.enabled,
        ),
        make_slider_int(
            "dnd_workspace_switch_max_speed",
            "Max speed",
            "Maximum switch speed",
            dnd.max_speed,
            100.0,
            5000.0,
            "px/s",
            dnd.enabled,
        ),
    ];

    ModelRc::new(VecModel::from(items))
}

// ============================================================================
// Sync functions
// ============================================================================

/// Sync all gesture UI models from settings
fn sync_gesture_models(ui: &MainWindow, settings: &Settings) {
    let hc = &settings.gestures.hot_corners;

    // Update hot corner selector state
    ui.set_gestures_hot_corners_enabled(hc.enabled);
    ui.set_gestures_hot_corner_top_left(hc.top_left);
    ui.set_gestures_hot_corner_top_right(hc.top_right);
    ui.set_gestures_hot_corner_bottom_left(hc.bottom_left);
    ui.set_gestures_hot_corner_bottom_right(hc.bottom_right);

    // Update dynamic models
    ui.set_gestures_hot_corners_settings(populate_hot_corners_settings(settings));
    ui.set_gestures_dnd_edge_scroll_settings(populate_dnd_edge_scroll_settings(settings));
    ui.set_gestures_dnd_workspace_switch_settings(populate_dnd_workspace_switch_settings(settings));
}

// ============================================================================
// Setup function
// ============================================================================

/// Set up dynamic gestures callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Generic toggle callback
    {
        let settings = Arc::clone(&settings);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_gestures_setting_toggle_changed(move |id, value| {
            let id_str = id.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    #[allow(unused_assignments)]
                    let mut needs_model_refresh = false;

                    match id_str.as_str() {
                        // Hot corners
                        "hot_corners_enabled" => {
                            s.gestures.hot_corners.enabled = value;
                            needs_model_refresh = true;
                        }

                        // DND edge scroll
                        "dnd_edge_scroll_enabled" => {
                            s.gestures.dnd_edge_view_scroll.enabled = value;
                            needs_model_refresh = true;
                        }

                        // DND workspace switch
                        "dnd_workspace_switch_enabled" => {
                            s.gestures.dnd_edge_workspace_switch.enabled = value;
                            needs_model_refresh = true;
                        }

                        _ => {
                            debug!("Unknown gesture toggle setting: {}", id_str);
                            return;
                        }
                    }

                    // Refresh models if visibility changed
                    if needs_model_refresh {
                        if let Some(ui) = ui_weak.upgrade() {
                            sync_gesture_models(&ui, &s);
                        }
                    }

                    debug!("Gesture toggle {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Gestures);
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
        ui.on_gestures_setting_slider_int_changed(move |id, value| {
            let id_str = id.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    match id_str.as_str() {
                        // DND edge scroll
                        "dnd_edge_scroll_trigger_width" => {
                            s.gestures.dnd_edge_view_scroll.trigger_size = value.clamp(10, 100);
                        }
                        "dnd_edge_scroll_delay" => {
                            s.gestures.dnd_edge_view_scroll.delay_ms = value.clamp(0, 500);
                        }
                        "dnd_edge_scroll_max_speed" => {
                            s.gestures.dnd_edge_view_scroll.max_speed = value.clamp(100, 5000);
                        }

                        // DND workspace switch
                        "dnd_workspace_switch_trigger_height" => {
                            s.gestures.dnd_edge_workspace_switch.trigger_size =
                                value.clamp(20, 150);
                        }
                        "dnd_workspace_switch_delay" => {
                            s.gestures.dnd_edge_workspace_switch.delay_ms = value.clamp(0, 500);
                        }
                        "dnd_workspace_switch_max_speed" => {
                            s.gestures.dnd_edge_workspace_switch.max_speed = value.clamp(100, 5000);
                        }

                        _ => {
                            debug!("Unknown gesture slider int setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Gesture slider int {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Gestures);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Hot corner visual selector callback
    {
        let settings = Arc::clone(&settings);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_gestures_hot_corner_toggled(move |corner, value| {
            let corner_str = corner.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    match corner_str.as_str() {
                        "top_left" => {
                            s.gestures.hot_corners.top_left = value;
                        }
                        "top_right" => {
                            s.gestures.hot_corners.top_right = value;
                        }
                        "bottom_left" => {
                            s.gestures.hot_corners.bottom_left = value;
                        }
                        "bottom_right" => {
                            s.gestures.hot_corners.bottom_right = value;
                        }
                        _ => {
                            debug!("Unknown hot corner: {}", corner_str);
                            return;
                        }
                    }

                    // Update UI state
                    if let Some(ui) = ui_weak.upgrade() {
                        match corner_str.as_str() {
                            "top_left" => ui.set_gestures_hot_corner_top_left(value),
                            "top_right" => ui.set_gestures_hot_corner_top_right(value),
                            "bottom_left" => ui.set_gestures_hot_corner_bottom_left(value),
                            "bottom_right" => ui.set_gestures_hot_corner_bottom_right(value),
                            _ => {}
                        }
                    }

                    debug!("Hot corner {} = {}", corner_str, value);
                    save_manager.mark_dirty(SettingsCategory::Gestures);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }
}

// ============================================================================
// Public sync function for use by sync.rs
// ============================================================================

/// Public function to sync all gesture models from settings
pub fn sync_all_gesture_models(ui: &MainWindow, settings: &Settings) {
    sync_gesture_models(ui, settings);
}
