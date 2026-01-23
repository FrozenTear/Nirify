//! Input device settings loaders
//!
//! Loads settings for keyboard, mouse, touchpad, trackpoint, trackball, tablet, and touch.
//!
//! Each device type has a shared `parse_*_from_children()` function that handles the actual
//! parsing logic, which is used by both the file loaders and the import system.
//!
//! Pointer devices (mouse, touchpad, trackpoint, trackball) share common settings which are
//! parsed by `parse_pointer_device_from_children()` using the `PointerDeviceSettings` trait.
//!
//! The `load_input_device` helper eliminates boilerplate for loading device settings from
//! KDL files by handling file reading, node navigation, and logging generically.

use super::super::parser::{get_f64, get_i64, get_string, has_flag};
use super::helpers::{
    parse_accel_profile, parse_click_method, parse_scroll_method, parse_tap_button_map,
    read_kdl_file,
};
use crate::config::models::Settings;
use crate::types::PointerDeviceSettings;
use kdl::KdlDocument;
use log::debug;
use std::path::Path;

/// Generic helper for loading input device settings from a KDL file.
///
/// Handles the common pattern of:
/// 1. Reading the KDL file
/// 2. Navigating to `input { <device_name> { ... } }`
/// 3. Calling the device-specific parser
/// 4. Logging the result
///
/// # Arguments
/// * `path` - Path to the KDL file
/// * `device_name` - Name of the device node (e.g., "mouse", "touchpad")
/// * `settings` - Settings struct to populate
/// * `parser` - Device-specific parsing function
fn load_input_device<F>(path: &Path, device_name: &str, settings: &mut Settings, parser: F)
where
    F: FnOnce(&KdlDocument, &mut Settings),
{
    let Some(doc) = read_kdl_file(path) else {
        return;
    };

    if let Some(input) = doc.get("input") {
        if let Some(input_children) = input.children() {
            if let Some(device) = input_children.get(device_name) {
                if let Some(device_children) = device.children() {
                    parser(device_children, settings);
                }
            }
        }
    }

    debug!("Loaded {} settings from {:?}", device_name, path);
}

/// Parse keyboard settings from keyboard node children
///
/// Shared parsing logic used by both file loader and import.
pub fn parse_keyboard_from_children(kbd_children: &KdlDocument, settings: &mut Settings) {
    // Off directive - disables the keyboard entirely
    settings.keyboard.off = has_flag(kbd_children, &["off"]);

    // XKB settings
    if let Some(xkb) = kbd_children.get("xkb") {
        if let Some(xkb_children) = xkb.children() {
            if let Some(v) = get_string(xkb_children, &["layout"]) {
                settings.keyboard.xkb_layout = v;
            }
            if let Some(v) = get_string(xkb_children, &["variant"]) {
                settings.keyboard.xkb_variant = v;
            }
            if let Some(v) = get_string(xkb_children, &["model"]) {
                settings.keyboard.xkb_model = v;
            }
            if let Some(v) = get_string(xkb_children, &["rules"]) {
                settings.keyboard.xkb_rules = v;
            }
            if let Some(v) = get_string(xkb_children, &["options"]) {
                settings.keyboard.xkb_options = v;
            }
            if let Some(v) = get_string(xkb_children, &["file"]) {
                settings.keyboard.xkb_file = v;
            }
        }
    }

    // Repeat settings
    if let Some(v) = get_i64(kbd_children, &["repeat-delay"]) {
        settings.keyboard.repeat_delay = v as i32;
    }
    if let Some(v) = get_i64(kbd_children, &["repeat-rate"]) {
        settings.keyboard.repeat_rate = v as i32;
    }

    // Numlock
    if has_flag(kbd_children, &["numlock"]) {
        settings.keyboard.numlock = true;
    }

    // Track layout
    if let Some(v) = get_string(kbd_children, &["track-layout"]) {
        settings.keyboard.track_layout = v;
    }
}

/// Load keyboard settings from KDL file
pub fn load_keyboard(path: &Path, settings: &mut Settings) {
    load_input_device(path, "keyboard", settings, parse_keyboard_from_children);
}

/// Parse common pointer device settings from KDL children.
///
/// This generic function parses the settings shared by all pointer devices
/// (mouse, touchpad, trackpoint, trackball):
/// - off, natural-scroll, left-handed, middle-emulation, scroll-button-lock
/// - accel-speed, accel-profile, scroll-method, scroll-button
///
/// Device-specific settings (like tap, dwt, scroll-factor) must be parsed
/// separately by the individual device parsers.
pub fn parse_pointer_device_from_children<T: PointerDeviceSettings>(
    children: &KdlDocument,
    device: &mut T,
) {
    // Boolean flags
    device.set_off(has_flag(children, &["off"]));
    device.set_natural_scroll(has_flag(children, &["natural-scroll"]));
    device.set_left_handed(has_flag(children, &["left-handed"]));
    device.set_middle_emulation(has_flag(children, &["middle-emulation"]));
    device.set_scroll_button_lock(has_flag(children, &["scroll-button-lock"]));

    // Acceleration settings
    if let Some(v) = get_f64(children, &["accel-speed"]) {
        device.set_accel_speed(v);
    }
    if let Some(v) = get_string(children, &["accel-profile"]) {
        device.set_accel_profile(parse_accel_profile(&v));
    }

    // Scroll settings
    if let Some(v) = get_string(children, &["scroll-method"]) {
        device.set_scroll_method(parse_scroll_method(&v));
    }
    if let Some(v) = get_i64(children, &["scroll-button"]) {
        device.set_scroll_button(Some(v as i32));
    }
}

/// Parse mouse settings from mouse node children
///
/// Shared parsing logic used by both file loader and import.
pub fn parse_mouse_from_children(m_children: &KdlDocument, settings: &mut Settings) {
    // Parse common pointer device settings
    parse_pointer_device_from_children(m_children, &mut settings.mouse);

    // Mouse-specific: scroll-factor (can be single value or "horizontal=X vertical=Y")
    parse_scroll_factor(
        m_children,
        &mut settings.mouse.scroll_factor,
        &mut settings.mouse.scroll_factor_horizontal,
    );
}

/// Parse scroll-factor which can be either a single f64 or a string "horizontal=X vertical=Y"
fn parse_scroll_factor(children: &KdlDocument, vertical: &mut f64, horizontal: &mut Option<f64>) {
    // First try to get as a string (for split format)
    if let Some(s) = get_string(children, &["scroll-factor"]) {
        // Parse "horizontal=X vertical=Y" format
        let mut h_val: Option<f64> = None;
        let mut v_val: Option<f64> = None;

        for part in s.split_whitespace() {
            if let Some(val_str) = part.strip_prefix("horizontal=") {
                h_val = val_str.parse().ok();
            } else if let Some(val_str) = part.strip_prefix("vertical=") {
                v_val = val_str.parse().ok();
            }
        }

        if let Some(v) = v_val {
            *vertical = v;
        }
        if let Some(h) = h_val {
            // Only set horizontal if it differs from vertical
            if h_val != v_val {
                *horizontal = Some(h);
            }
        }
    } else if let Some(v) = get_f64(children, &["scroll-factor"]) {
        // Single value: applies to both directions
        *vertical = v;
        *horizontal = None;
    }
}

/// Load mouse settings from KDL file
pub fn load_mouse(path: &Path, settings: &mut Settings) {
    load_input_device(path, "mouse", settings, parse_mouse_from_children);
}

/// Parse touchpad settings from touchpad node children
///
/// Shared parsing logic used by both file loader and import.
pub fn parse_touchpad_from_children(tp_children: &KdlDocument, settings: &mut Settings) {
    // Parse common pointer device settings
    parse_pointer_device_from_children(tp_children, &mut settings.touchpad);

    // Touchpad-specific boolean flags
    settings.touchpad.tap = has_flag(tp_children, &["tap"]);
    settings.touchpad.dwt = has_flag(tp_children, &["dwt"]);
    settings.touchpad.dwtp = has_flag(tp_children, &["dwtp"]);
    settings.touchpad.drag = has_flag(tp_children, &["drag"]);
    settings.touchpad.drag_lock = has_flag(tp_children, &["drag-lock"]);
    settings.touchpad.disabled_on_external_mouse =
        has_flag(tp_children, &["disabled-on-external-mouse"]);

    // Touchpad-specific: scroll-factor (can be single value or "horizontal=X vertical=Y")
    parse_scroll_factor(
        tp_children,
        &mut settings.touchpad.scroll_factor,
        &mut settings.touchpad.scroll_factor_horizontal,
    );

    // Touchpad-specific: click-method, tap-button-map
    if let Some(v) = get_string(tp_children, &["click-method"]) {
        settings.touchpad.click_method = parse_click_method(&v);
    }
    if let Some(v) = get_string(tp_children, &["tap-button-map"]) {
        settings.touchpad.tap_button_map = parse_tap_button_map(&v);
    }
}

/// Load touchpad settings from KDL file
pub fn load_touchpad(path: &Path, settings: &mut Settings) {
    load_input_device(path, "touchpad", settings, parse_touchpad_from_children);
}

/// Parse trackpoint settings from trackpoint node children
///
/// Shared parsing logic used by both file loader and import.
/// Trackpoint has no device-specific settings beyond the common pointer settings.
pub fn parse_trackpoint_from_children(tp_children: &KdlDocument, settings: &mut Settings) {
    parse_pointer_device_from_children(tp_children, &mut settings.trackpoint);
}

/// Load trackpoint settings from KDL file
pub fn load_trackpoint(path: &Path, settings: &mut Settings) {
    load_input_device(path, "trackpoint", settings, parse_trackpoint_from_children);
}

/// Parse trackball settings from trackball node children
///
/// Shared parsing logic used by both file loader and import.
/// Trackball has no device-specific settings beyond the common pointer settings.
pub fn parse_trackball_from_children(tb_children: &KdlDocument, settings: &mut Settings) {
    parse_pointer_device_from_children(tb_children, &mut settings.trackball);
}

/// Load trackball settings from KDL file
pub fn load_trackball(path: &Path, settings: &mut Settings) {
    load_input_device(path, "trackball", settings, parse_trackball_from_children);
}

/// Parse tablet settings from tablet node children
///
/// Shared parsing logic used by both file loader and import.
pub fn parse_tablet_from_children(t_children: &KdlDocument, settings: &mut Settings) {
    settings.tablet.off = has_flag(t_children, &["off"]);
    settings.tablet.left_handed = has_flag(t_children, &["left-handed"]);

    if let Some(v) = get_string(t_children, &["map-to-output"]) {
        settings.tablet.map_to_output = v;
    }

    // Calibration matrix - 6 floats
    if let Some(matrix_node) = t_children.get("calibration-matrix") {
        let entries: Vec<f64> = matrix_node
            .entries()
            .iter()
            .filter_map(|e| e.value().as_float())
            .collect();
        if entries.len() == 6 {
            settings.tablet.calibration_matrix = Some([
                entries[0], entries[1], entries[2], entries[3], entries[4], entries[5],
            ]);
        }
    }
}

/// Load tablet (drawing tablet / stylus) settings from KDL file
pub fn load_tablet(path: &Path, settings: &mut Settings) {
    load_input_device(path, "tablet", settings, parse_tablet_from_children);
}

/// Parse touch settings from touch node children
///
/// Shared parsing logic used by both file loader and import.
pub fn parse_touch_from_children(t_children: &KdlDocument, settings: &mut Settings) {
    settings.touch.off = has_flag(t_children, &["off"]);

    if let Some(v) = get_string(t_children, &["map-to-output"]) {
        settings.touch.map_to_output = v;
    }

    // Calibration matrix - 6 floats
    if let Some(matrix_node) = t_children.get("calibration-matrix") {
        let entries: Vec<f64> = matrix_node
            .entries()
            .iter()
            .filter_map(|e| e.value().as_float())
            .collect();
        if entries.len() == 6 {
            settings.touch.calibration_matrix = Some([
                entries[0], entries[1], entries[2], entries[3], entries[4], entries[5],
            ]);
        }
    }
}

/// Load touch screen settings from KDL file
pub fn load_touch(path: &Path, settings: &mut Settings) {
    load_input_device(path, "touch", settings, parse_touch_from_children);
}
