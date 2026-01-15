//! Hardware detection for input devices
//!
//! Detects presence of touchpad, trackpoint, and other input devices
//! to conditionally show/hide relevant settings tabs.

use log::debug;
use std::fs;
use std::path::Path;

/// Input device capabilities detected from the system
#[derive(Debug, Clone, Default)]
pub struct InputDevices {
    pub has_touchpad: bool,
    pub has_trackpoint: bool,
    pub has_trackball: bool,
    pub has_tablet: bool,
    pub has_touch: bool,
}

impl InputDevices {
    /// Detect available input devices from /sys/class/input
    ///
    /// This reads the Linux input subsystem to determine what devices exist.
    /// Falls back to showing all tabs if detection fails.
    pub fn detect() -> Self {
        let mut devices = Self::default();

        // Read /sys/class/input to find input devices
        let input_path = Path::new("/sys/class/input");
        if !input_path.exists() {
            debug!("Cannot detect hardware: /sys/class/input not found");
            return Self::assume_all_present();
        }

        let entries = match fs::read_dir(input_path) {
            Ok(e) => e,
            Err(e) => {
                debug!("Cannot read /sys/class/input: {}", e);
                return Self::assume_all_present();
            }
        };

        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            // Only process eventN devices
            if !name_str.starts_with("event") {
                continue;
            }

            let device_path = entry.path().join("device");

            // Read device name
            let name_file = device_path.join("name");
            let device_name = fs::read_to_string(&name_file)
                .map(|s| s.trim().to_lowercase())
                .unwrap_or_default();

            // Read device capabilities
            let caps_path = device_path.join("capabilities");

            // Check for touchpad indicators
            if is_touchpad(&device_name, &caps_path) {
                debug!("Detected touchpad: {}", device_name);
                devices.has_touchpad = true;
            }

            // Check for trackpoint indicators
            if is_trackpoint(&device_name) {
                debug!("Detected trackpoint: {}", device_name);
                devices.has_trackpoint = true;
            }

            // Check for trackball
            if is_trackball(&device_name) {
                debug!("Detected trackball: {}", device_name);
                devices.has_trackball = true;
            }

            // Check for tablet/stylus
            if is_tablet(&device_name, &caps_path) {
                debug!("Detected tablet: {}", device_name);
                devices.has_tablet = true;
            }

            // Check for touchscreen
            if is_touchscreen(&device_name, &caps_path) {
                debug!("Detected touchscreen: {}", device_name);
                devices.has_touch = true;
            }
        }

        debug!("Hardware detection complete: {:?}", devices);
        devices
    }

    /// Assume all devices are present (fallback)
    fn assume_all_present() -> Self {
        Self {
            has_touchpad: true,
            has_trackpoint: true,
            has_trackball: true,
            has_tablet: true,
            has_touch: true,
        }
    }
}

/// Check if device is likely a touchpad
fn is_touchpad(name: &str, caps_path: &Path) -> bool {
    // Name-based detection
    if name.contains("touchpad")
        || name.contains("trackpad")
        || name.contains("clickpad")
        || name.contains("synaptics")
        || name.contains("elan")
        || name.contains("alps")
    {
        return true;
    }

    // Capability-based detection: touchpads have absolute positioning (ABS)
    // and typically BTN_TOOL_FINGER
    let abs_caps = caps_path.join("abs");
    if abs_caps.exists() {
        if let Ok(caps) = fs::read_to_string(&abs_caps) {
            // Non-zero ABS capabilities suggest absolute positioning device
            let caps_val = u64::from_str_radix(caps.trim(), 16).unwrap_or(0);
            if caps_val != 0 {
                // Also check if it has key capabilities (for clicking)
                let key_caps = caps_path.join("key");
                if key_caps.exists() {
                    return true;
                }
            }
        }
    }

    false
}

/// Check if device is likely a trackpoint
fn is_trackpoint(name: &str) -> bool {
    name.contains("trackpoint")
        || name.contains("pointing stick")
        || name.contains("tpps")
        || name.contains("nipple")
        || (name.contains("lenovo") && name.contains("stick"))
}

/// Check if device is likely a trackball
fn is_trackball(name: &str) -> bool {
    name.contains("trackball")
        || name.contains("kensington")
        || name.contains("logitech marble")
        || name.contains("elecom")
}

/// Check if device is likely a drawing tablet
fn is_tablet(name: &str, caps_path: &Path) -> bool {
    // Name-based detection
    if name.contains("wacom")
        || name.contains("tablet")
        || name.contains("stylus")
        || name.contains("pen")
        || name.contains("huion")
        || name.contains("xp-pen")
        || name.contains("gaomon")
    {
        return true;
    }

    // Check for stylus/pen capabilities
    let key_caps = caps_path.join("key");
    if key_caps.exists() {
        if let Ok(caps) = fs::read_to_string(&key_caps) {
            // BTN_STYLUS and BTN_TOOL_PEN are indicators
            let caps_val = u64::from_str_radix(caps.trim(), 16).unwrap_or(0);
            // High bits often indicate stylus buttons
            if caps_val > 0x1_0000_0000 {
                return true;
            }
        }
    }

    false
}

/// Check if device is likely a touchscreen
fn is_touchscreen(name: &str, caps_path: &Path) -> bool {
    // Name-based detection
    if name.contains("touchscreen") || name.contains("touch screen") {
        return true;
    }

    // Touchscreens have ABS_MT_POSITION capabilities (multi-touch)
    let abs_caps = caps_path.join("abs");
    if abs_caps.exists() {
        if let Ok(caps) = fs::read_to_string(&abs_caps) {
            let caps_val = u64::from_str_radix(caps.trim(), 16).unwrap_or(0);
            // ABS_MT_POSITION_X is bit 53, so high values indicate multi-touch
            if caps_val > 0x10_0000_0000_0000 {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_runs_without_panic() {
        // Just verify detection doesn't crash
        let devices = InputDevices::detect();
        // On CI/servers without input devices, we get the fallback
        // On real systems, we get actual detection
        println!("Detected: {:?}", devices);
    }

    #[test]
    fn test_touchpad_name_detection() {
        assert!(is_touchpad("synaptics touchpad", Path::new("")));
        assert!(is_touchpad("elan trackpad", Path::new("")));
        assert!(is_touchpad("alps clickpad", Path::new("")));
        assert!(!is_touchpad("logitech mouse", Path::new("")));
    }

    #[test]
    fn test_trackpoint_name_detection() {
        assert!(is_trackpoint("tpps/2 ibm trackpoint"));
        assert!(is_trackpoint("lenovo pointing stick"));
        assert!(!is_trackpoint("logitech mouse"));
    }

    #[test]
    fn test_tablet_name_detection() {
        assert!(is_tablet("wacom intuos", Path::new("")));
        assert!(is_tablet("huion tablet", Path::new("")));
        assert!(is_tablet("xp-pen artist", Path::new("")));
        assert!(!is_tablet("logitech mouse", Path::new("")));
    }
}
