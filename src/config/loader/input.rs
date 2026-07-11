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
    read_kdl_file, slashdash_node_present,
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
    // Note: niri's Keyboard has no `off` field; keyboards cannot be disabled.

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

/// Parse `scroll-factor`, accepting all niri-native forms plus the legacy
/// Nirify quoted-string form.
///
/// niri accepts:
///   * `scroll-factor 2.0`            (bare base argument, 0..100)
///   * `scroll-factor horizontal=H vertical=V` (properties, each -100..100)
///   * `scroll-factor 2 vertical=-1`  (mixed: base + property override)
///
/// Legacy Nirify files wrote `scroll-factor "horizontal=X vertical=Y"` (a
/// quoted string). We still parse that so those files load once and get
/// rewritten in the correct form on next save.
fn parse_scroll_factor(children: &KdlDocument, vertical: &mut f64, horizontal: &mut Option<f64>) {
    let Some(node) = children.get("scroll-factor") else {
        return;
    };

    let mut base: Option<f64> = None;
    let mut h_prop: Option<f64> = None;
    let mut v_prop: Option<f64> = None;

    for entry in node.entries() {
        // float-or-int reader for a single entry value
        let as_num = |e: &kdl::KdlEntry| -> Option<f64> {
            e.value()
                .as_float()
                .or_else(|| e.value().as_integer().map(|i| i as f64))
        };

        match entry.name().map(|n| n.value()) {
            None => {
                // Positional entry. Could be the numeric base, or the legacy
                // quoted "horizontal=X vertical=Y" string.
                if let Some(n) = as_num(entry) {
                    base = Some(n);
                } else if let Some(s) = entry.value().as_string() {
                    log::warn!("Migrating legacy quoted scroll-factor string \"{}\"", s);
                    for part in s.split_whitespace() {
                        if let Some(val) = part.strip_prefix("horizontal=") {
                            h_prop = val.parse().ok();
                        } else if let Some(val) = part.strip_prefix("vertical=") {
                            v_prop = val.parse().ok();
                        }
                    }
                }
            }
            Some("horizontal") => h_prop = as_num(entry),
            Some("vertical") => v_prop = as_num(entry),
            Some(_) => {}
        }
    }

    // Nothing usable found.
    if base.is_none() && h_prop.is_none() && v_prop.is_none() {
        return;
    }

    // Mirror niri's ScrollFactor::h_v_factors: base fills in any missing axis.
    let bv = base.unwrap_or(1.0);
    let h = h_prop.unwrap_or(bv);
    let v = v_prop.unwrap_or(bv);

    *vertical = v;
    *horizontal = if h != v { Some(h) } else { None };
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
    // drag is Option<bool>: None when absent (libinput default), else its value
    settings.touchpad.drag = tp_children
        .get("drag")
        .map(|_| has_flag(tp_children, &["drag"]));
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
    settings.tablet.map_to_focused_output = has_flag(t_children, &["map-to-focused-output"]);
    settings.tablet.map_to_focused_window = has_flag(t_children, &["map-to-focused-window"]);

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

    // P1 preservation: version-gated / unreleased mapping flags are written
    // slashdashed (`/-map-to-focused-output`, `/-map-to-focused-window`) so they
    // survive round-trips without being applied by an incompatible niri. The KDL
    // parser drops slashdashed nodes, so read them back from the raw file text.
    // Use a string/comment-aware scan so the flags aren't falsely set by the
    // node name appearing inside a comment or quoted string.
    if let Ok(text) = std::fs::read_to_string(path) {
        if slashdash_node_present(&text, "map-to-focused-output") {
            settings.tablet.map_to_focused_output = true;
        }
        if slashdash_node_present(&text, "map-to-focused-window") {
            settings.tablet.map_to_focused_window = true;
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn write_tmp(name: &str, contents: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "nirify_loader_input_{}_{}.kdl",
            name,
            std::process::id()
        ));
        std::fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn load_tablet_reads_back_real_slashdash_flags() {
        let kdl = "tablet {\n    /-map-to-focused-output\n    /-map-to-focused-window\n}\n";
        let path = write_tmp("real_slashdash", kdl);
        let mut settings = Settings::default();
        load_tablet(&path, &mut settings);
        let _ = std::fs::remove_file(&path);
        assert!(settings.tablet.map_to_focused_output);
        assert!(settings.tablet.map_to_focused_window);
    }

    #[test]
    fn load_tablet_ignores_slashdash_text_in_comment() {
        // The token appears only inside a line comment and a quoted string, so the
        // string/comment-aware scan must NOT set the flags (regression guard for
        // the old naive `str::contains`).
        let kdl = "tablet {\n    // /-map-to-focused-output is documented here\n    /*\n      /-map-to-focused-window in a block comment\n    */\n    map-to-output \"note: /-map-to-focused-output /-map-to-focused-window\"\n}\n";
        let path = write_tmp("comment_only", kdl);
        let mut settings = Settings::default();
        load_tablet(&path, &mut settings);
        let _ = std::fs::remove_file(&path);
        assert!(!settings.tablet.map_to_focused_output);
        assert!(!settings.tablet.map_to_focused_window);
    }
}
