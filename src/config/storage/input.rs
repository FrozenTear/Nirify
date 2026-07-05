//! Input device KDL generation
//!
//! Generates KDL configuration for all input devices: keyboard, mouse, touchpad,
//! trackpoint, trackball, tablet, and touch.

use super::helpers::{
    accel_profile_to_kdl, click_method_to_kdl, escape_kdl_string, scroll_method_to_kdl,
    tap_button_map_to_kdl, write_common_input_settings,
};
use crate::config::models::{
    KeyboardSettings, MouseSettings, TabletSettings, TouchSettings, TouchpadSettings,
    TrackballSettings, TrackpointSettings,
};
use crate::types::{AccelProfile, ScrollMethod};
use crate::version::FeatureCompat;

/// Trait for pointer devices that share common settings (trackpoint, trackball).
///
/// This trait allows generating KDL for any pointer device with a common set of
/// properties: off, natural_scroll, left_handed, middle_emulation, accel_speed,
/// accel_profile, scroll_method, scroll_button, and scroll_button_lock.
pub trait PointerDeviceSettings {
    fn off(&self) -> bool;
    fn natural_scroll(&self) -> bool;
    fn left_handed(&self) -> bool;
    fn middle_emulation(&self) -> bool;
    fn accel_speed(&self) -> f64;
    fn accel_profile(&self) -> AccelProfile;
    /// Scroll method (None = libinput device default; emitted only when Some)
    fn scroll_method(&self) -> Option<ScrollMethod>;
    fn scroll_button(&self) -> Option<i32>;
    fn scroll_button_lock(&self) -> bool;
}

impl PointerDeviceSettings for TrackpointSettings {
    fn off(&self) -> bool {
        self.off
    }
    fn natural_scroll(&self) -> bool {
        self.natural_scroll
    }
    fn left_handed(&self) -> bool {
        self.left_handed
    }
    fn middle_emulation(&self) -> bool {
        self.middle_emulation
    }
    fn accel_speed(&self) -> f64 {
        self.accel_speed
    }
    fn accel_profile(&self) -> AccelProfile {
        self.accel_profile
    }
    fn scroll_method(&self) -> Option<ScrollMethod> {
        self.scroll_method
    }
    fn scroll_button(&self) -> Option<i32> {
        self.scroll_button
    }
    fn scroll_button_lock(&self) -> bool {
        self.scroll_button_lock
    }
}

impl PointerDeviceSettings for TrackballSettings {
    fn off(&self) -> bool {
        self.off
    }
    fn natural_scroll(&self) -> bool {
        self.natural_scroll
    }
    fn left_handed(&self) -> bool {
        self.left_handed
    }
    fn middle_emulation(&self) -> bool {
        self.middle_emulation
    }
    fn accel_speed(&self) -> f64 {
        self.accel_speed
    }
    fn accel_profile(&self) -> AccelProfile {
        self.accel_profile
    }
    fn scroll_method(&self) -> Option<ScrollMethod> {
        self.scroll_method
    }
    fn scroll_button(&self) -> Option<i32> {
        self.scroll_button
    }
    fn scroll_button_lock(&self) -> bool {
        self.scroll_button_lock
    }
}

/// Generate KDL content for a pointer device (trackpoint, trackball).
///
/// This factory function generates KDL for any device implementing `PointerDeviceSettings`.
/// The generated KDL includes:
/// - Device off flag
/// - Common pointer settings (natural-scroll, left-handed, middle-emulation)
/// - Acceleration settings (accel-speed, accel-profile)
/// - Scroll settings (scroll-method, scroll-button, scroll-button-lock)
///
/// # Arguments
/// * `device_name` - The name of the device (e.g., "trackpoint", "trackball")
/// * `settings` - The device settings implementing `PointerDeviceSettings`
///
/// # Returns
/// A formatted KDL string for the device configuration
fn generate_pointer_device_kdl(device_name: &str, settings: &impl PointerDeviceSettings) -> String {
    let mut content = String::with_capacity(512);
    content.push_str(&format!(
        "// {} settings - managed by Nirify\n\ninput {{\n    {} {{\n",
        // Capitalize first letter for comment
        {
            let mut chars = device_name.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        },
        device_name
    ));

    if settings.off() {
        content.push_str("        off\n");
    }

    // Common pointer device settings
    if settings.natural_scroll() {
        content.push_str("        natural-scroll\n");
    }
    if settings.left_handed() {
        content.push_str("        left-handed\n");
    }
    if settings.middle_emulation() {
        content.push_str("        middle-emulation\n");
    }
    if settings.accel_speed().abs() > 0.001 {
        content.push_str(&format!(
            "        accel-speed {:.2}\n",
            settings.accel_speed()
        ));
    }
    if !matches!(settings.accel_profile(), AccelProfile::Adaptive) {
        content.push_str(&format!(
            "        accel-profile \"{}\"\n",
            accel_profile_to_kdl(settings.accel_profile())
        ));
    }

    // Scroll method: only output when explicitly set (None = libinput default)
    if let Some(method) = settings.scroll_method() {
        content.push_str(&format!(
            "        scroll-method \"{}\"\n",
            scroll_method_to_kdl(method)
        ));
    }

    if let Some(button) = settings.scroll_button() {
        content.push_str(&format!("        scroll-button {}\n", button));
    }
    if settings.scroll_button_lock() {
        content.push_str("        scroll-button-lock\n");
    }

    content.push_str("    }\n}\n");
    content
}

/// Generate keyboard.kdl content
pub fn generate_keyboard_kdl(settings: &KeyboardSettings) -> String {
    let mut xkb_extra = String::new();
    if !settings.xkb_variant.is_empty() {
        xkb_extra.push_str(&format!(
            "\n            variant \"{}\"",
            escape_kdl_string(&settings.xkb_variant)
        ));
    }
    if !settings.xkb_model.is_empty() {
        xkb_extra.push_str(&format!(
            "\n            model \"{}\"",
            escape_kdl_string(&settings.xkb_model)
        ));
    }
    if !settings.xkb_rules.is_empty() {
        xkb_extra.push_str(&format!(
            "\n            rules \"{}\"",
            escape_kdl_string(&settings.xkb_rules)
        ));
    }
    if !settings.xkb_options.is_empty() {
        xkb_extra.push_str(&format!(
            "\n            options \"{}\"",
            escape_kdl_string(&settings.xkb_options)
        ));
    }
    // xkb file overrides other xkb settings
    if !settings.xkb_file.is_empty() {
        xkb_extra.push_str(&format!(
            "\n            file \"{}\"",
            escape_kdl_string(&settings.xkb_file)
        ));
    }

    // Note: Keyboard does not support 'off' flag in niri - keyboards cannot be disabled
    format!(
        r#"// Keyboard settings - managed by Nirify

input {{
    keyboard {{
        xkb {{
            layout "{}"{}
        }}
        repeat-delay {}
        repeat-rate {}{}
        track-layout "{}"
    }}
}}
"#,
        escape_kdl_string(&settings.xkb_layout),
        xkb_extra,
        settings.repeat_delay,
        settings.repeat_rate,
        if settings.numlock {
            "\n        numlock"
        } else {
            ""
        },
        escape_kdl_string(&settings.track_layout),
    )
}

/// Generate mouse.kdl content
pub fn generate_mouse_kdl(settings: &MouseSettings) -> String {
    // Pre-allocate ~512 bytes for typical mouse config
    let mut content = String::with_capacity(512);
    content.push_str("// Mouse settings - managed by Nirify\n\ninput {\n    mouse {\n");

    // Check if device is disabled
    if settings.off {
        content.push_str("        off\n");
    }

    // Common input settings
    write_common_input_settings(
        &mut content,
        settings.natural_scroll,
        settings.left_handed,
        settings.middle_emulation,
        settings.accel_speed,
        settings.accel_profile,
        settings.scroll_factor,
        settings.scroll_factor_horizontal,
    );

    // Scroll method: only output when explicitly set (None = libinput default)
    if let Some(method) = settings.scroll_method {
        content.push_str(&format!(
            "        scroll-method \"{}\"\n",
            scroll_method_to_kdl(method)
        ));
    }

    // Scroll button for on-button-down scrolling
    if let Some(button) = settings.scroll_button {
        content.push_str(&format!("        scroll-button {}\n", button));
    }

    // Scroll button lock
    if settings.scroll_button_lock {
        content.push_str("        scroll-button-lock\n");
    }

    content.push_str("    }\n}\n");
    content
}

/// Generate touchpad.kdl content
pub fn generate_touchpad_kdl(settings: &TouchpadSettings) -> String {
    // Pre-allocate ~768 bytes for typical touchpad config (more options than mouse)
    let mut content = String::with_capacity(768);
    content.push_str("// Touchpad settings - managed by Nirify\n\ninput {\n    touchpad {\n");

    // Check if device is disabled
    if settings.off {
        content.push_str("        off\n");
    }

    // Touchpad-specific flags
    if settings.tap {
        content.push_str("        tap\n");
    }
    if settings.dwt {
        content.push_str("        dwt\n");
    }
    if settings.dwtp {
        content.push_str("        dwtp\n");
    }
    // drag requires a boolean argument in niri; only emit when explicitly set
    if let Some(drag) = settings.drag {
        content.push_str(&format!("        drag {}\n", drag));
    }
    if settings.drag_lock {
        content.push_str("        drag-lock\n");
    }
    if settings.disabled_on_external_mouse {
        content.push_str("        disabled-on-external-mouse\n");
    }

    // Common input settings (natural_scroll, left_handed, middle_emulation, accel, scroll_factor)
    write_common_input_settings(
        &mut content,
        settings.natural_scroll,
        settings.left_handed,
        settings.middle_emulation,
        settings.accel_speed,
        settings.accel_profile,
        settings.scroll_factor,
        settings.scroll_factor_horizontal,
    );

    // Touchpad-specific enums: Nirify treats each enum's default as "unset" and
    // only emits when the user chose a non-default value (behavior-neutral save).
    if settings.tap_button_map != crate::types::TapButtonMap::default() {
        content.push_str(&format!(
            "        tap-button-map \"{}\"\n",
            tap_button_map_to_kdl(settings.tap_button_map)
        ));
    }
    if settings.click_method != crate::types::ClickMethod::default() {
        content.push_str(&format!(
            "        click-method \"{}\"\n",
            click_method_to_kdl(settings.click_method)
        ));
    }
    // Scroll method: only output when explicitly set (None = libinput default)
    if let Some(method) = settings.scroll_method {
        content.push_str(&format!(
            "        scroll-method \"{}\"\n",
            scroll_method_to_kdl(method)
        ));
    }

    // Scroll button for on-button-down scrolling
    if let Some(button) = settings.scroll_button {
        content.push_str(&format!("        scroll-button {}\n", button));
    }

    // Scroll button lock
    if settings.scroll_button_lock {
        content.push_str("        scroll-button-lock\n");
    }

    content.push_str("    }\n}\n");
    content
}

/// Generate trackpoint.kdl content
pub fn generate_trackpoint_kdl(settings: &TrackpointSettings) -> String {
    generate_pointer_device_kdl("trackpoint", settings)
}

/// Generate trackball.kdl content
pub fn generate_trackball_kdl(settings: &TrackballSettings) -> String {
    generate_pointer_device_kdl("trackball", settings)
}

/// Trait for mapped input devices (tablet, touch) that share common settings.
///
/// These devices can be mapped to outputs and have calibration matrices.
trait MappedInputDevice {
    fn device_name(&self) -> &'static str;
    /// Capitalized device name for comments (e.g., "Tablet", "Touch")
    fn device_title(&self) -> &'static str;
    fn off(&self) -> bool;
    fn map_to_output(&self) -> &str;
    fn calibration_matrix(&self) -> Option<[f64; 6]>;
    /// Device-specific properties to write after the common ones.
    /// `compat` gates version-dependent nodes (e.g. tablet map-to-focused-output).
    fn write_specific(&self, content: &mut String, compat: FeatureCompat);
}

impl MappedInputDevice for TabletSettings {
    fn device_name(&self) -> &'static str {
        "tablet"
    }
    fn device_title(&self) -> &'static str {
        "Tablet"
    }
    fn off(&self) -> bool {
        self.off
    }
    fn map_to_output(&self) -> &str {
        &self.map_to_output
    }
    fn calibration_matrix(&self) -> Option<[f64; 6]> {
        self.calibration_matrix
    }
    fn write_specific(&self, content: &mut String, compat: FeatureCompat) {
        // Tablet-specific: focused-output/window mapping and left_handed.
        //
        // `map-to-focused-output` is Since niri 26.04, gated on its own dedicated
        // compat flag (`map_to_focused_output`). When the running niri is too old
        // (or unknown), the value is preserved slashdashed (`/-`) so it is not
        // destroyed but also not applied.
        if self.map_to_focused_output {
            if compat.map_to_focused_output {
                content.push_str("        map-to-focused-output\n");
            } else {
                content.push_str(
                    "        // map-to-focused-output requires niri 26.04+ (preserved)\n",
                );
                content.push_str("        /-map-to-focused-output\n");
            }
        }
        // `map-to-focused-window` is unreleased in niri ("Since: next release").
        // Never emit it as an active node; preserve it slashdashed if set.
        if self.map_to_focused_window {
            content
                .push_str("        // map-to-focused-window is unreleased in niri (preserved)\n");
            content.push_str("        /-map-to-focused-window\n");
        }
        if self.left_handed {
            content.push_str("        left-handed\n");
        }
    }
}

impl MappedInputDevice for TouchSettings {
    fn device_name(&self) -> &'static str {
        "touch"
    }
    fn device_title(&self) -> &'static str {
        "Touch"
    }
    fn off(&self) -> bool {
        self.off
    }
    fn map_to_output(&self) -> &str {
        &self.map_to_output
    }
    fn calibration_matrix(&self) -> Option<[f64; 6]> {
        self.calibration_matrix
    }
    fn write_specific(&self, _content: &mut String, _compat: FeatureCompat) {
        // Touch has no device-specific properties
    }
}

/// Generate KDL for a mapped input device (tablet or touch)
fn generate_mapped_input_kdl(device: &impl MappedInputDevice, compat: FeatureCompat) -> String {
    let name = device.device_name();
    let title = device.device_title();
    let mut content = String::with_capacity(256);

    // Header
    content.push_str(&format!(
        "// {} settings - managed by Nirify\n\ninput {{\n    {} {{\n",
        title, name
    ));

    // Common properties
    if device.off() {
        content.push_str("        off\n");
    }

    let map_to_output = device.map_to_output();
    if !map_to_output.is_empty() {
        content.push_str(&format!(
            "        map-to-output \"{}\"\n",
            escape_kdl_string(map_to_output)
        ));
    }

    // Device-specific properties (e.g., tablet's left_handed)
    device.write_specific(&mut content, compat);

    // Calibration matrix (common but written after specific)
    if let Some(matrix) = device.calibration_matrix() {
        content.push_str(&format!(
            "        calibration-matrix {} {} {} {} {} {}\n",
            matrix[0], matrix[1], matrix[2], matrix[3], matrix[4], matrix[5]
        ));
    }

    content.push_str("    }\n}\n");
    content
}

/// Generate tablet.kdl content.
///
/// `compat` gates version-dependent nodes: `map-to-focused-output` (niri 26.04+)
/// is preserved slashdashed when unsupported, and the unreleased
/// `map-to-focused-window` is always preserved slashdashed.
pub fn generate_tablet_kdl(settings: &TabletSettings, compat: FeatureCompat) -> String {
    generate_mapped_input_kdl(settings, compat)
}

/// Generate touch.kdl content
pub fn generate_touch_kdl(settings: &TouchSettings) -> String {
    // Touch has no version-gated nodes; compat is irrelevant here.
    generate_mapped_input_kdl(settings, FeatureCompat::all_enabled())
}

#[cfg(test)]
// Test setup mutates a couple fields after default() for readability.
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use crate::config::loader::{
        parse_mouse_from_children, parse_tablet_from_children, parse_touchpad_from_children,
        parse_trackpoint_from_children,
    };
    use crate::config::models::Settings;
    use crate::types::ScrollMethod;

    /// Re-parse generated device KDL via the kdl crate, navigate `input > device`,
    /// then run the loader's parse fn against the device children.
    fn load_device(
        kdl: &str,
        device: &str,
        parse: fn(&kdl::KdlDocument, &mut Settings),
    ) -> Settings {
        let doc: kdl::KdlDocument = kdl.parse().expect("generated input KDL must re-parse");
        let mut settings = Settings::default();
        if let Some(input) = doc.get("input").and_then(|n| n.children()) {
            if let Some(dev) = input.get(device).and_then(|n| n.children()) {
                parse(dev, &mut settings);
            }
        }
        settings
    }

    #[test]
    fn roundtrip_scroll_factor_uniform() {
        let mut m = MouseSettings::default();
        m.scroll_factor = 2.5;
        m.scroll_factor_horizontal = None;
        let kdl = generate_mouse_kdl(&m);
        assert!(kdl.contains("scroll-factor 2.50"));
        assert!(!kdl.contains('"'));
        let loaded = load_device(&kdl, "mouse", parse_mouse_from_children);
        assert_eq!(loaded.mouse.scroll_factor, 2.5);
        assert_eq!(loaded.mouse.scroll_factor_horizontal, None);
    }

    #[test]
    fn roundtrip_scroll_factor_split() {
        let mut t = TouchpadSettings::default();
        t.scroll_factor = 1.0;
        t.scroll_factor_horizontal = Some(-1.5);
        let kdl = generate_touchpad_kdl(&t);
        assert!(kdl.contains("scroll-factor horizontal=-1.50 vertical=1.00"));
        let loaded = load_device(&kdl, "touchpad", parse_touchpad_from_children);
        assert_eq!(loaded.touchpad.scroll_factor, 1.0);
        assert_eq!(loaded.touchpad.scroll_factor_horizontal, Some(-1.5));
    }

    #[test]
    fn roundtrip_scroll_factor_uniform_negative() {
        let mut m = MouseSettings::default();
        m.scroll_factor = -1.0;
        m.scroll_factor_horizontal = None;
        let kdl = generate_mouse_kdl(&m);
        assert!(kdl.contains("scroll-factor horizontal=-1.00 vertical=-1.00"));
        let loaded = load_device(&kdl, "mouse", parse_mouse_from_children);
        assert_eq!(loaded.mouse.scroll_factor, -1.0);
        assert_eq!(loaded.mouse.scroll_factor_horizontal, None);
    }

    #[test]
    fn scroll_factor_default_omitted() {
        let m = MouseSettings::default();
        let kdl = generate_mouse_kdl(&m);
        assert!(!kdl.contains("scroll-factor"));
    }

    #[test]
    fn legacy_scroll_factor_string_still_loads() {
        let kdl =
            "input {\n  mouse {\n    scroll-factor \"horizontal=2.00 vertical=1.00\"\n  }\n}\n";
        let loaded = load_device(kdl, "mouse", parse_mouse_from_children);
        assert_eq!(loaded.mouse.scroll_factor, 1.0);
        assert_eq!(loaded.mouse.scroll_factor_horizontal, Some(2.0));
    }

    #[test]
    fn roundtrip_mouse_no_scroll() {
        let mut m = MouseSettings::default();
        m.scroll_method = Some(ScrollMethod::NoScroll);
        let kdl = generate_mouse_kdl(&m);
        assert!(kdl.contains("scroll-method \"no-scroll\""));
        let loaded = load_device(&kdl, "mouse", parse_mouse_from_children);
        assert_eq!(loaded.mouse.scroll_method, Some(ScrollMethod::NoScroll));

        // None => node absent => loads None.
        let m2 = MouseSettings::default();
        let kdl2 = generate_mouse_kdl(&m2);
        assert!(!kdl2.contains("scroll-method"));
        let loaded2 = load_device(&kdl2, "mouse", parse_mouse_from_children);
        assert_eq!(loaded2.mouse.scroll_method, None);
    }

    #[test]
    fn roundtrip_trackpoint_on_button_down() {
        let mut tp = TrackpointSettings::default();
        tp.scroll_method = Some(ScrollMethod::OnButtonDown);
        let kdl = generate_trackpoint_kdl(&tp);
        assert!(kdl.contains("scroll-method \"on-button-down\""));
        let loaded = load_device(&kdl, "trackpoint", parse_trackpoint_from_children);
        assert_eq!(
            loaded.trackpoint.scroll_method,
            Some(ScrollMethod::OnButtonDown)
        );
    }

    #[test]
    fn tablet_focused_output_active_when_supported() {
        // niri 26.04+ (map_to_focused_output gate true): output mapping is emitted
        // as a live node; the unreleased window mapping stays slashdashed.
        let mut t = TabletSettings::default();
        t.map_to_focused_output = true;
        t.map_to_focused_window = true;
        let kdl = generate_tablet_kdl(&t, crate::version::FeatureCompat::all_enabled());
        // Must re-parse via the kdl crate.
        let _doc: kdl::KdlDocument = kdl.parse().expect("tablet KDL must re-parse");
        assert!(kdl.contains("        map-to-focused-output\n"));
        assert!(!kdl.contains("/-map-to-focused-output"));
        assert!(kdl.contains("/-map-to-focused-window"));
        // Parser (live nodes only) sees the active output mapping.
        let loaded = load_device(&kdl, "tablet", parse_tablet_from_children);
        assert!(loaded.tablet.map_to_focused_output);
        assert!(!loaded.tablet.map_to_focused_window);
    }

    #[test]
    fn tablet_gated_flags_preserved_via_slashdash() {
        use crate::config::loader::load_tablet;
        // Unknown/old niri (compat default = all false): both mappings are
        // preserved slashdashed, and the file loader reads them back (P1).
        let mut t = TabletSettings::default();
        t.map_to_focused_output = true;
        t.map_to_focused_window = true;
        let kdl = generate_tablet_kdl(&t, crate::version::FeatureCompat::default());
        let _doc: kdl::KdlDocument = kdl.parse().expect("tablet KDL must re-parse");
        assert!(kdl.contains("/-map-to-focused-output"));
        assert!(kdl.contains("/-map-to-focused-window"));

        // Live-node parser cannot see slashdashed nodes.
        let live = load_device(&kdl, "tablet", parse_tablet_from_children);
        assert!(!live.tablet.map_to_focused_output);
        assert!(!live.tablet.map_to_focused_window);

        // File loader restores them from the raw slashdash text.
        let path = std::env::temp_dir().join(format!(
            "nirify_tablet_slashdash_{}.kdl",
            std::process::id()
        ));
        std::fs::write(&path, &kdl).unwrap();
        let mut settings = Settings::default();
        load_tablet(&path, &mut settings);
        let _ = std::fs::remove_file(&path);
        assert!(settings.tablet.map_to_focused_output);
        assert!(settings.tablet.map_to_focused_window);
    }

    #[test]
    fn touchpad_default_save_is_neutral() {
        let kdl = generate_touchpad_kdl(&TouchpadSettings::default());
        // Must re-parse.
        let _doc: kdl::KdlDocument = kdl.parse().expect("touchpad KDL must re-parse");
        for node in [
            "tap",
            "natural-scroll",
            "dwt",
            "drag",
            "scroll-method",
            "tap-button-map",
            "click-method",
            "accel-speed",
            "accel-profile",
            "scroll-factor",
        ] {
            assert!(
                !kdl.contains(node),
                "default touchpad save should not emit `{}`",
                node
            );
        }
    }
}
