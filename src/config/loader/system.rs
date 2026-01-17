//! System settings loaders
//!
//! Handles startup commands, environment variables, debug settings,
//! switch events, and recent windows.

use super::helpers::{load_color, read_kdl_file};
use super::rules::has_flag_in_node;
use crate::config::models::{
    EnvironmentVariable, RecentWindowsBind, RecentWindowsScope, Settings, StartupCommand,
};
use crate::config::parser::{get_f64, get_i64, get_string, has_flag};
use kdl::KdlDocument;
use log::debug;
use std::path::Path;

// ============================================================================
// STARTUP COMMANDS
// ============================================================================

/// Parse spawn-at-startup commands from a document
///
/// Shared parsing logic used by both file loader and import.
pub fn parse_startup_from_doc(doc: &KdlDocument, settings: &mut Settings) {
    settings.startup.commands.clear();
    let mut next_id = 0u32;

    for node in doc.nodes() {
        if node.name().value() == "spawn-at-startup" {
            let mut args: Vec<String> = Vec::new();

            // Collect all string arguments from the node
            for entry in node.entries() {
                if entry.name().is_none() {
                    // Positional argument
                    if let Some(s) = entry.value().as_string() {
                        args.push(s.to_string());
                    }
                }
            }

            if !args.is_empty() {
                settings.startup.commands.push(StartupCommand {
                    id: next_id,
                    command: args,
                });
                next_id += 1;
            }
        }
    }

    settings.startup.next_id = next_id;
}

/// Load startup commands from KDL file
pub fn load_startup(path: &Path, settings: &mut Settings) {
    let Some(doc) = read_kdl_file(path) else {
        return;
    };

    parse_startup_from_doc(&doc, settings);
    debug!(
        "Loaded {} startup commands from {:?}",
        settings.startup.commands.len(),
        path
    );
}

// ============================================================================
// ENVIRONMENT VARIABLES
// ============================================================================

/// Parse environment variables from a document
///
/// Shared parsing logic used by both file loader and import.
pub fn parse_environment_from_doc(doc: &KdlDocument, settings: &mut Settings) {
    settings.environment.variables.clear();
    let mut next_id = 0u32;

    // Look for environment block
    if let Some(env_node) = doc.get("environment") {
        if let Some(env_children) = env_node.children() {
            for node in env_children.nodes() {
                let name = node.name().value().to_string();
                // Get the first string argument as the value
                let value = node
                    .entries()
                    .iter()
                    .find(|e| e.name().is_none())
                    .and_then(|e| e.value().as_string())
                    .map(|s| s.to_string())
                    .unwrap_or_default();

                settings.environment.variables.push(EnvironmentVariable {
                    id: next_id,
                    name,
                    value,
                });
                next_id += 1;
            }
        }
    }

    settings.environment.next_id = next_id;
}

/// Load environment variables from KDL file
pub fn load_environment(path: &Path, settings: &mut Settings) {
    let Some(doc) = read_kdl_file(path) else {
        return;
    };

    parse_environment_from_doc(&doc, settings);
    debug!(
        "Loaded {} environment variables from {:?}",
        settings.environment.variables.len(),
        path
    );
}

// ============================================================================
// DEBUG SETTINGS
// ============================================================================

/// Parse debug settings from a document
///
/// Shared parsing logic used by both file loader and import.
pub fn parse_debug_from_doc(doc: &KdlDocument, settings: &mut Settings) {
    if let Some(debug_node) = doc.get("debug") {
        if let Some(debug_children) = debug_node.children() {
            if has_flag(debug_children, &["preview-render"]) {
                settings.debug.preview_render = true;
            }
            if has_flag(debug_children, &["enable-overlay-planes"]) {
                settings.debug.enable_overlay_planes = true;
            }
            if has_flag(debug_children, &["disable-cursor-plane"]) {
                settings.debug.disable_cursor_plane = true;
            }
            if has_flag(debug_children, &["disable-direct-scanout"]) {
                settings.debug.disable_direct_scanout = true;
            }
            if let Some(v) = get_string(debug_children, &["render-drm-device"]) {
                settings.debug.render_drm_device = Some(v);
            }
            if has_flag(
                debug_children,
                &["wait-for-frame-completion-before-queueing"],
            ) {
                settings.debug.wait_for_frame_completion_before_queueing = true;
            }
            if has_flag(debug_children, &["disable-resize-throttling"]) {
                settings.debug.disable_resize_throttling = true;
            }
            if has_flag(debug_children, &["disable-transactions"]) {
                settings.debug.disable_transactions = true;
            }
            if has_flag(debug_children, &["emulate-zero-presentation-time"]) {
                settings.debug.emulate_zero_presentation_time = true;
            }
            if has_flag(
                debug_children,
                &["dbus-interfaces-in-non-session-instances"],
            ) {
                settings.debug.dbus_interfaces_in_non_session_instances = true;
            }
            if has_flag(debug_children, &["keep-laptop-panel-on-when-lid-is-closed"]) {
                settings.debug.keep_laptop_panel_on_when_lid_is_closed = true;
            }
            if has_flag(debug_children, &["disable-monitor-names"]) {
                settings.debug.disable_monitor_names = true;
            }
            if has_flag(debug_children, &["strict-new-window-focus-policy"]) {
                settings.debug.strict_new_window_focus_policy = true;
            }
            if has_flag(
                debug_children,
                &["restrict-primary-scanout-to-matching-format"],
            ) {
                settings.debug.restrict_primary_scanout_to_matching_format = true;
            }
            if has_flag(debug_children, &["skip-cursor-only-updates-during-vrr"]) {
                settings.debug.skip_cursor_only_updates_during_vrr = true;
            }
            if has_flag(debug_children, &["force-disable-connectors-on-resume"]) {
                settings.debug.force_disable_connectors_on_resume = true;
            }
            if has_flag(
                debug_children,
                &["honor-xdg-activation-with-invalid-serial"],
            ) {
                settings.debug.honor_xdg_activation_with_invalid_serial = true;
            }
            if has_flag(debug_children, &["deactivate-unfocused-windows"]) {
                settings.debug.deactivate_unfocused_windows = true;
            }
            if has_flag(debug_children, &["force-pipewire-invalid-modifier"]) {
                settings.debug.force_pipewire_invalid_modifier = true;
            }
            // Parse ignore-drm-device entries (can have multiple)
            for node in debug_children.nodes() {
                if node.name().value() == "ignore-drm-device" {
                    if let Some(device) = node
                        .entries()
                        .iter()
                        .find(|e| e.name().is_none())
                        .and_then(|e| e.value().as_string())
                    {
                        settings.debug.ignore_drm_devices.push(device.to_string());
                    }
                }
            }
        }
    }
}

/// Load debug settings from KDL file
pub fn load_debug(path: &Path, settings: &mut Settings) {
    let Some(doc) = read_kdl_file(path) else {
        return;
    };

    parse_debug_from_doc(&doc, settings);
    debug!("Loaded debug settings from {:?}", path);
}

// ============================================================================
// SWITCH EVENTS
// ============================================================================

/// Parse switch events settings from a document
///
/// Shared parsing logic used by both file loader and import.
pub fn parse_switch_events_from_doc(doc: &KdlDocument, settings: &mut Settings) {
    if let Some(switch_node) = doc.get("switch-events") {
        if let Some(switch_children) = switch_node.children() {
            // Lid close
            if let Some(lid_close) = switch_children.get("lid-close") {
                if let Some(lc_children) = lid_close.children() {
                    settings.switch_events.lid_close.spawn = parse_spawn_entries(lc_children);
                }
            }
            // Lid open
            if let Some(lid_open) = switch_children.get("lid-open") {
                if let Some(lo_children) = lid_open.children() {
                    settings.switch_events.lid_open.spawn = parse_spawn_entries(lo_children);
                }
            }
            // Tablet mode on
            if let Some(tablet_on) = switch_children.get("tablet-mode-on") {
                if let Some(to_children) = tablet_on.children() {
                    settings.switch_events.tablet_mode_on.spawn = parse_spawn_entries(to_children);
                }
            }
            // Tablet mode off
            if let Some(tablet_off) = switch_children.get("tablet-mode-off") {
                if let Some(tof_children) = tablet_off.children() {
                    settings.switch_events.tablet_mode_off.spawn =
                        parse_spawn_entries(tof_children);
                }
            }
        }
    }
}

/// Parse spawn entries from a switch event children document
fn parse_spawn_entries(doc: &KdlDocument) -> Vec<String> {
    let mut commands = Vec::new();
    for node in doc.nodes() {
        if node.name().value() == "spawn" {
            // Collect all string arguments
            let args: Vec<String> = node
                .entries()
                .iter()
                .filter(|e| e.name().is_none())
                .filter_map(|e| e.value().as_string().map(|s| s.to_string()))
                .collect();

            if !args.is_empty() {
                // Join arguments as a single command string for display
                commands.push(args.join(" "));
            }
        }
    }
    commands
}

/// Load switch events from KDL file
pub fn load_switch_events(path: &Path, settings: &mut Settings) {
    let Some(doc) = read_kdl_file(path) else {
        return;
    };

    parse_switch_events_from_doc(&doc, settings);
    debug!("Loaded switch events from {:?}", path);
}

// ============================================================================
// RECENT WINDOWS (v25.05+)
// ============================================================================

/// Parse recent windows settings from a document
///
/// Shared parsing logic used by both file loader and import.
pub fn parse_recent_windows_from_doc(doc: &KdlDocument, settings: &mut Settings) {
    if let Some(rw_node) = doc.get("recent-windows") {
        // Check for "off" flag
        if has_flag_in_node(rw_node, "off") {
            settings.recent_windows.off = true;
            return;
        }

        if let Some(rw_children) = rw_node.children() {
            // Check for "off" as child node
            if has_flag(rw_children, &["off"]) {
                settings.recent_windows.off = true;
                return;
            }

            // debounce-ms
            if let Some(v) = get_i64(rw_children, &["debounce-ms"]) {
                settings.recent_windows.debounce_ms = v as i32;
            }

            // open-delay-ms
            if let Some(v) = get_i64(rw_children, &["open-delay-ms"]) {
                settings.recent_windows.open_delay_ms = v as i32;
            }

            // highlight block
            if let Some(highlight_node) = rw_children.get("highlight") {
                if let Some(h_children) = highlight_node.children() {
                    // active-color
                    load_color(
                        h_children,
                        &["active-color"],
                        &mut settings.recent_windows.highlight.active_color,
                    );
                    // urgent-color
                    load_color(
                        h_children,
                        &["urgent-color"],
                        &mut settings.recent_windows.highlight.urgent_color,
                    );
                    // padding
                    if let Some(v) = get_i64(h_children, &["padding"]) {
                        settings.recent_windows.highlight.padding = v as i32;
                    }
                    // corner-radius
                    if let Some(v) = get_i64(h_children, &["corner-radius"]) {
                        settings.recent_windows.highlight.corner_radius = v as i32;
                    }
                }
            }

            // previews block
            if let Some(previews_node) = rw_children.get("previews") {
                if let Some(p_children) = previews_node.children() {
                    // max-height
                    if let Some(v) = get_i64(p_children, &["max-height"]) {
                        settings.recent_windows.previews.max_height = v as i32;
                    }
                    // max-scale
                    if let Some(v) = get_f64(p_children, &["max-scale"]) {
                        settings.recent_windows.previews.max_scale = v;
                    }
                }
            }

            // binds block
            if let Some(binds_node) = rw_children.get("binds") {
                if let Some(b_children) = binds_node.children() {
                    settings.recent_windows.binds.clear();
                    for bind_node in b_children.nodes() {
                        // The node name is the key combo (e.g., "Alt+Tab")
                        let key_combo = bind_node.name().value().to_string();

                        // Parse cooldown-ms from the bind node entries
                        let mut cooldown_ms = None;
                        for entry in bind_node.entries() {
                            if let Some(name) = entry.name() {
                                if name.value() == "cooldown-ms" {
                                    if let Some(v) = entry.value().as_integer() {
                                        cooldown_ms = Some(v as i32);
                                    }
                                }
                            }
                        }

                        // Parse the action from children (next-window or previous-window)
                        if let Some(action_children) = bind_node.children() {
                            for action_node in action_children.nodes() {
                                let action_name = action_node.name().value();
                                let is_next = action_name == "next-window";
                                let is_prev = action_name == "previous-window";

                                if is_next || is_prev {
                                    let mut filter_app_id = false;
                                    let mut scope = None;

                                    for entry in action_node.entries() {
                                        if let Some(name) = entry.name() {
                                            match name.value() {
                                                "filter" => {
                                                    if let Some(v) = entry.value().as_string() {
                                                        filter_app_id = v == "app-id";
                                                    }
                                                }
                                                "scope" => {
                                                    if let Some(v) = entry.value().as_string() {
                                                        scope = RecentWindowsScope::from_kdl(v);
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                    }

                                    settings.recent_windows.binds.push(RecentWindowsBind {
                                        key_combo: key_combo.clone(),
                                        is_next,
                                        filter_app_id,
                                        scope,
                                        cooldown_ms,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Load recent windows settings from KDL file
pub fn load_recent_windows(path: &Path, settings: &mut Settings) {
    let Some(doc) = read_kdl_file(path) else {
        return;
    };

    parse_recent_windows_from_doc(&doc, settings);
    debug!("Loaded recent windows settings from {:?}", path);
}
