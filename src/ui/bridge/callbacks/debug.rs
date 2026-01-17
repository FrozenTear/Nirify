//! Dynamic debug settings UI callbacks
//!
//! Handles debug settings using model-driven dynamic UI.
//! These are advanced settings primarily for developers and troubleshooting.

use crate::config::models::DebugSettings;
use crate::config::{Settings, SettingsCategory};
use crate::{DebugSettingModel, MainWindow};
use log::{debug, error};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::SaveManager;

// =============================================================================
// Helper functions for creating setting models
// =============================================================================

fn make_toggle(id: &str, label: &str, desc: &str, value: bool, visible: bool) -> DebugSettingModel {
    DebugSettingModel {
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
) -> DebugSettingModel {
    DebugSettingModel {
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

#[allow(dead_code)]
fn make_slider_int(
    id: &str,
    label: &str,
    desc: &str,
    value: i32,
    min: f32,
    max: f32,
    suffix: &str,
    visible: bool,
) -> DebugSettingModel {
    DebugSettingModel {
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
) -> DebugSettingModel {
    DebugSettingModel {
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
) -> DebugSettingModel {
    let opts: Vec<SharedString> = options.iter().map(|s| (*s).into()).collect();
    DebugSettingModel {
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

// =============================================================================
// Functions to populate each section's model from DebugSettings
// =============================================================================

/// Populate rendering settings model
pub fn populate_rendering_settings(debug: &DebugSettings) -> ModelRc<DebugSettingModel> {
    let settings = vec![
        make_toggle(
            "preview_render",
            "Preview render",
            "Enable render preview mode for debugging",
            debug.preview_render,
            true,
        ),
        make_toggle(
            "enable_overlay_planes",
            "Enable overlay planes",
            "Use hardware overlay planes (may improve or degrade performance)",
            debug.enable_overlay_planes,
            true,
        ),
        make_toggle(
            "disable_cursor_plane",
            "Disable cursor plane",
            "Render cursor in software instead of using hardware plane",
            debug.disable_cursor_plane,
            true,
        ),
        make_toggle(
            "disable_direct_scanout",
            "Disable direct scanout",
            "Prevent bypassing the compositor for fullscreen surfaces",
            debug.disable_direct_scanout,
            true,
        ),
        make_toggle(
            "restrict_primary_scanout_to_matching_format",
            "Restrict primary scanout to matching format",
            "Only scanout when buffer format matches composition format",
            debug.restrict_primary_scanout_to_matching_format,
            true,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

/// Populate GPU settings model
pub fn populate_gpu_settings(debug: &DebugSettings) -> ModelRc<DebugSettingModel> {
    let render_drm_device = debug.render_drm_device.as_deref().unwrap_or("");
    let ignore_drm_devices = debug.ignore_drm_devices.join(", ");

    let settings = vec![
        make_text(
            "render_drm_device",
            "Render DRM device",
            "Override the GPU used for rendering (leave empty for default)",
            render_drm_device,
            "/dev/dri/renderD128",
            true,
        ),
        make_text(
            "ignore_drm_devices",
            "Ignore DRM devices",
            "DRM devices to exclude, comma-separated (useful for GPU passthrough)",
            &ignore_drm_devices,
            "/dev/dri/card1, /dev/dri/renderD129",
            true,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

/// Populate frame timing settings model
pub fn populate_frame_timing_settings(debug: &DebugSettings) -> ModelRc<DebugSettingModel> {
    let settings = vec![
        make_toggle(
            "wait_for_frame_completion",
            "Wait for frame completion before queueing",
            "May fix tearing issues but can reduce performance",
            debug.wait_for_frame_completion_before_queueing,
            true,
        ),
        make_toggle(
            "emulate_zero_presentation_time",
            "Emulate zero presentation time",
            "Report zero presentation time to clients for testing",
            debug.emulate_zero_presentation_time,
            true,
        ),
        make_toggle(
            "skip_cursor_only_updates_during_vrr",
            "Skip cursor-only updates during VRR",
            "Skip redraws from cursor movement during variable refresh rate",
            debug.skip_cursor_only_updates_during_vrr,
            true,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

/// Populate window management settings model
pub fn populate_window_management_settings(debug: &DebugSettings) -> ModelRc<DebugSettingModel> {
    let settings = vec![
        make_toggle(
            "disable_resize_throttling",
            "Disable resize throttling",
            "Don't throttle window resize events",
            debug.disable_resize_throttling,
            true,
        ),
        make_toggle(
            "disable_transactions",
            "Disable transactions",
            "Disable synchronized window updates",
            debug.disable_transactions,
            true,
        ),
        make_toggle(
            "strict_new_window_focus",
            "Strict new window focus policy",
            "Use stricter rules for focusing newly opened windows",
            debug.strict_new_window_focus_policy,
            true,
        ),
        make_toggle(
            "honor_xdg_activation_with_invalid_serial",
            "Honor XDG activation with invalid serial",
            "Allow focus via invalid xdg-activation serial",
            debug.honor_xdg_activation_with_invalid_serial,
            true,
        ),
        make_toggle(
            "deactivate_unfocused_windows",
            "Deactivate unfocused windows",
            "Drop activated state for windows that lose focus",
            debug.deactivate_unfocused_windows,
            true,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

/// Populate system settings model
pub fn populate_system_settings(debug: &DebugSettings) -> ModelRc<DebugSettingModel> {
    let settings = vec![
        make_toggle(
            "dbus_interfaces_in_non_session",
            "D-Bus interfaces in non-session instances",
            "Expose D-Bus interfaces when running outside a user session",
            debug.dbus_interfaces_in_non_session_instances,
            true,
        ),
        make_toggle(
            "keep_laptop_panel_on_lid_closed",
            "Keep laptop panel on when lid closed",
            "Don't turn off the built-in display when closing the lid",
            debug.keep_laptop_panel_on_when_lid_is_closed,
            true,
        ),
        make_toggle(
            "disable_monitor_names",
            "Disable monitor names",
            "Don't show monitor names in configuration",
            debug.disable_monitor_names,
            true,
        ),
        make_toggle(
            "force_disable_connectors_on_resume",
            "Force disable connectors on resume",
            "Force blank all outputs on TTY switch or resume",
            debug.force_disable_connectors_on_resume,
            true,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

/// Populate screencasting settings model
pub fn populate_screencasting_settings(debug: &DebugSettings) -> ModelRc<DebugSettingModel> {
    let settings = vec![make_toggle(
        "force_pipewire_invalid_modifier",
        "Force PipeWire invalid modifier",
        "Force invalid DRM modifier for PipeWire screencasting",
        debug.force_pipewire_invalid_modifier,
        true,
    )];
    ModelRc::new(VecModel::from(settings))
}

/// Sync all UI models from DebugSettings
pub fn sync_all_models(ui: &MainWindow, debug: &DebugSettings) {
    ui.set_debug_rendering_settings(populate_rendering_settings(debug));
    ui.set_debug_gpu_settings(populate_gpu_settings(debug));
    ui.set_debug_frame_timing_settings(populate_frame_timing_settings(debug));
    ui.set_debug_window_management_settings(populate_window_management_settings(debug));
    ui.set_debug_system_settings(populate_system_settings(debug));
    ui.set_debug_screencasting_settings(populate_screencasting_settings(debug));
}

// =============================================================================
// Generic callback handlers that dispatch based on setting ID
// =============================================================================

/// Handle toggle changes dispatched by setting ID
fn handle_toggle_change(settings: &mut Settings, id: &str, value: bool) -> bool {
    match id {
        // Rendering settings
        "preview_render" => {
            settings.debug.preview_render = value;
        }
        "enable_overlay_planes" => {
            settings.debug.enable_overlay_planes = value;
        }
        "disable_cursor_plane" => {
            settings.debug.disable_cursor_plane = value;
        }
        "disable_direct_scanout" => {
            settings.debug.disable_direct_scanout = value;
        }
        "restrict_primary_scanout_to_matching_format" => {
            settings.debug.restrict_primary_scanout_to_matching_format = value;
        }

        // Frame timing settings
        "wait_for_frame_completion" => {
            settings.debug.wait_for_frame_completion_before_queueing = value;
        }
        "emulate_zero_presentation_time" => {
            settings.debug.emulate_zero_presentation_time = value;
        }
        "skip_cursor_only_updates_during_vrr" => {
            settings.debug.skip_cursor_only_updates_during_vrr = value;
        }

        // Window management settings
        "disable_resize_throttling" => {
            settings.debug.disable_resize_throttling = value;
        }
        "disable_transactions" => {
            settings.debug.disable_transactions = value;
        }
        "strict_new_window_focus" => {
            settings.debug.strict_new_window_focus_policy = value;
        }
        "honor_xdg_activation_with_invalid_serial" => {
            settings.debug.honor_xdg_activation_with_invalid_serial = value;
        }
        "deactivate_unfocused_windows" => {
            settings.debug.deactivate_unfocused_windows = value;
        }

        // System settings
        "dbus_interfaces_in_non_session" => {
            settings.debug.dbus_interfaces_in_non_session_instances = value;
        }
        "keep_laptop_panel_on_lid_closed" => {
            settings.debug.keep_laptop_panel_on_when_lid_is_closed = value;
        }
        "disable_monitor_names" => {
            settings.debug.disable_monitor_names = value;
        }
        "force_disable_connectors_on_resume" => {
            settings.debug.force_disable_connectors_on_resume = value;
        }

        // Screencasting settings
        "force_pipewire_invalid_modifier" => {
            settings.debug.force_pipewire_invalid_modifier = value;
        }

        _ => {
            debug!("Unknown debug toggle setting: {}", id);
            return false;
        }
    }
    true
}

/// Handle text changes dispatched by setting ID
fn handle_text_change(settings: &mut Settings, id: &str, value: &str) -> bool {
    match id {
        "render_drm_device" => {
            settings.debug.render_drm_device = if value.is_empty() {
                None
            } else {
                Some(value.to_string())
            };
        }
        "ignore_drm_devices" => {
            settings.debug.ignore_drm_devices = if value.is_empty() {
                Vec::new()
            } else {
                value
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            };
        }
        _ => {
            debug!("Unknown debug text setting: {}", id);
            return false;
        }
    }
    true
}

// =============================================================================
// Setup function
// =============================================================================

/// Set up dynamic debug settings callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Expert mode - special handling: updates in-memory only, does NOT save to disk
    // This is intentional: expert_mode should reset to false on app restart for safety
    {
        let settings = settings.clone();
        ui.on_debug_dynamic_expert_mode_toggled(move |value| match settings.lock() {
            Ok(mut s) => {
                s.debug.expert_mode = value;
                debug!(
                    "Debug: expert mode toggled to {} (in-memory only, not persisted)",
                    value
                );
                // Intentionally NOT calling mark_dirty or request_save
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Generic toggle callback
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        let ui_weak = ui.as_weak();
        ui.on_debug_setting_toggle_changed(move |id, value| {
            let id_str = id.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    if handle_toggle_change(&mut s, &id_str, value) {
                        debug!("Debug toggle {} = {}", id_str, value);
                        save_manager.mark_dirty(SettingsCategory::Debug);
                        save_manager.request_save();

                        // Update UI models if needed (for conditional visibility)
                        if let Some(ui) = ui_weak.upgrade() {
                            sync_all_models(&ui, &s.debug);
                        }
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Generic text callback
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        let ui_weak = ui.as_weak();
        ui.on_debug_setting_text_changed(move |id, value| {
            let id_str = id.to_string();
            let value_str = value.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    if handle_text_change(&mut s, &id_str, &value_str) {
                        debug!("Debug text {} = {}", id_str, value_str);
                        save_manager.mark_dirty(SettingsCategory::Debug);
                        save_manager.request_save();

                        // Update UI models
                        if let Some(ui) = ui_weak.upgrade() {
                            sync_all_models(&ui, &s.debug);
                        }
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }
}
