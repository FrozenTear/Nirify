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
    fn scroll_method(&self) -> ScrollMethod;
    fn scroll_button(&self) -> Option<i32>;
    fn scroll_button_lock(&self) -> bool;
    /// The default scroll method for this device type (used to determine whether to output scroll-method)
    fn default_scroll_method(&self) -> ScrollMethod;
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
    fn scroll_method(&self) -> ScrollMethod {
        self.scroll_method
    }
    fn scroll_button(&self) -> Option<i32> {
        self.scroll_button
    }
    fn scroll_button_lock(&self) -> bool {
        self.scroll_button_lock
    }
    fn default_scroll_method(&self) -> ScrollMethod {
        ScrollMethod::OnButtonDown
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
    fn scroll_method(&self) -> ScrollMethod {
        self.scroll_method
    }
    fn scroll_button(&self) -> Option<i32> {
        self.scroll_button
    }
    fn scroll_button_lock(&self) -> bool {
        self.scroll_button_lock
    }
    fn default_scroll_method(&self) -> ScrollMethod {
        ScrollMethod::OnButtonDown
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
        "// {} settings - managed by niri-settings-rust\n\ninput {{\n    {} {{\n",
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

    // Scroll method (only output if different from device default)
    if settings.scroll_method() != settings.default_scroll_method() {
        content.push_str(&format!(
            "        scroll-method \"{}\"\n",
            scroll_method_to_kdl(settings.scroll_method())
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
        r#"// Keyboard settings - managed by niri-settings-rust

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
    content.push_str("// Mouse settings - managed by niri-settings-rust\n\ninput {\n    mouse {\n");

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
    );

    // Only output scroll-method if it's not the default (no-scroll for mouse)
    if !matches!(settings.scroll_method, ScrollMethod::NoScroll) {
        content.push_str(&format!(
            "        scroll-method \"{}\"\n",
            scroll_method_to_kdl(settings.scroll_method)
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
    content.push_str(
        "// Touchpad settings - managed by niri-settings-rust\n\ninput {\n    touchpad {\n",
    );

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
    // drag requires a boolean argument in niri
    content.push_str(&format!("        drag {}\n", settings.drag));
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
    );

    // Touchpad-specific enums
    content.push_str(&format!(
        "        tap-button-map \"{}\"\n",
        tap_button_map_to_kdl(settings.tap_button_map)
    ));
    content.push_str(&format!(
        "        click-method \"{}\"\n",
        click_method_to_kdl(settings.click_method)
    ));
    content.push_str(&format!(
        "        scroll-method \"{}\"\n",
        scroll_method_to_kdl(settings.scroll_method)
    ));

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
    /// Device-specific properties to write after the common ones
    fn write_specific(&self, content: &mut String);
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
    fn write_specific(&self, content: &mut String) {
        // Tablet-specific: left_handed
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
    fn write_specific(&self, _content: &mut String) {
        // Touch has no device-specific properties
    }
}

/// Generate KDL for a mapped input device (tablet or touch)
fn generate_mapped_input_kdl(device: &impl MappedInputDevice) -> String {
    let name = device.device_name();
    let title = device.device_title();
    let mut content = String::with_capacity(256);

    // Header
    content.push_str(&format!(
        "// {} settings - managed by niri-settings-rust\n\ninput {{\n    {} {{\n",
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
    device.write_specific(&mut content);

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

/// Generate tablet.kdl content
pub fn generate_tablet_kdl(settings: &TabletSettings) -> String {
    generate_mapped_input_kdl(settings)
}

/// Generate touch.kdl content
pub fn generate_touch_kdl(settings: &TouchSettings) -> String {
    generate_mapped_input_kdl(settings)
}
