//! Debug settings UI callbacks
//!
//! Handles advanced debug options for niri.
//! These are primarily for developers and troubleshooting.

use crate::config::category_section::Debug;
use crate::config::{Settings, SettingsCategory};
use crate::MainWindow;
use log::{debug, error};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::{register_bool_callbacks, SaveManager};

/// Set up debug settings callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Expert mode - special handling: updates in-memory only, does NOT save to disk
    // This is intentional: expert_mode should reset to false on app restart for safety
    {
        let settings = settings.clone();
        ui.on_debug_expert_mode_toggled(move |value| match settings.lock() {
            Ok(mut s) => {
                s.debug.expert_mode = value;
                debug!("Debug: expert mode toggled to {} (in-memory only, not persisted)", value);
                // Intentionally NOT calling mark_dirty or request_save
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Boolean callbacks - all debug settings are booleans
    register_bool_callbacks!(
        ui,
        settings,
        save_manager,
        Debug,
        [
            (
                on_debug_preview_render_toggled,
                preview_render,
                "Debug: preview render"
            ),
            (
                on_debug_enable_overlay_planes_toggled,
                enable_overlay_planes,
                "Debug: enable overlay planes"
            ),
            (
                on_debug_disable_cursor_plane_toggled,
                disable_cursor_plane,
                "Debug: disable cursor plane"
            ),
            (
                on_debug_disable_direct_scanout_toggled,
                disable_direct_scanout,
                "Debug: disable direct scanout"
            ),
            (
                on_debug_restrict_primary_scanout_to_matching_format_toggled,
                restrict_primary_scanout_to_matching_format,
                "Debug: restrict primary scanout to matching format"
            ),
            (
                on_debug_wait_for_frame_completion_toggled,
                wait_for_frame_completion_before_queueing,
                "Debug: wait for frame completion"
            ),
            (
                on_debug_disable_resize_throttling_toggled,
                disable_resize_throttling,
                "Debug: disable resize throttling"
            ),
            (
                on_debug_disable_transactions_toggled,
                disable_transactions,
                "Debug: disable transactions"
            ),
            (
                on_debug_emulate_zero_presentation_time_toggled,
                emulate_zero_presentation_time,
                "Debug: emulate zero presentation time"
            ),
            (
                on_debug_skip_cursor_only_updates_during_vrr_toggled,
                skip_cursor_only_updates_during_vrr,
                "Debug: skip cursor-only updates during VRR"
            ),
            (
                on_debug_dbus_interfaces_in_non_session_toggled,
                dbus_interfaces_in_non_session_instances,
                "Debug: D-Bus interfaces in non-session"
            ),
            (
                on_debug_keep_laptop_panel_on_lid_closed_toggled,
                keep_laptop_panel_on_when_lid_is_closed,
                "Debug: keep laptop panel on lid closed"
            ),
            (
                on_debug_disable_monitor_names_toggled,
                disable_monitor_names,
                "Debug: disable monitor names"
            ),
            (
                on_debug_force_disable_connectors_on_resume_toggled,
                force_disable_connectors_on_resume,
                "Debug: force disable connectors on resume"
            ),
            (
                on_debug_strict_new_window_focus_toggled,
                strict_new_window_focus_policy,
                "Debug: strict new window focus"
            ),
            (
                on_debug_honor_xdg_activation_with_invalid_serial_toggled,
                honor_xdg_activation_with_invalid_serial,
                "Debug: honor XDG activation with invalid serial"
            ),
            (
                on_debug_deactivate_unfocused_windows_toggled,
                deactivate_unfocused_windows,
                "Debug: deactivate unfocused windows"
            ),
            (
                on_debug_force_pipewire_invalid_modifier_toggled,
                force_pipewire_invalid_modifier,
                "Debug: force PipeWire invalid modifier"
            ),
        ]
    );

    // Render DRM device - needs custom handling for Option<String>
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_debug_render_drm_device_changed(move |value| match settings.lock() {
            Ok(mut s) => {
                let val = value.to_string();
                if val.is_empty() {
                    s.debug.render_drm_device = None;
                } else {
                    s.debug.render_drm_device = Some(val);
                }
                debug!("Debug: render DRM device changed");
                save_manager.mark_dirty(SettingsCategory::Debug);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Ignore DRM devices - comma-separated list of devices
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_debug_ignore_drm_devices_changed(move |value| match settings.lock() {
            Ok(mut s) => {
                let val = value.to_string();
                // Parse comma-separated list, trim whitespace
                s.debug.ignore_drm_devices = if val.is_empty() {
                    Vec::new()
                } else {
                    val.split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                };
                debug!("Debug: ignore DRM devices changed");
                save_manager.mark_dirty(SettingsCategory::Debug);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }
}
