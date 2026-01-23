//! Input device settings (keyboard, mouse, touchpad, trackpoint, trackball, tablet, touch)

use crate::constants::{DEFAULT_REPEAT_DELAY, DEFAULT_REPEAT_RATE};
use crate::types::{AccelProfile, ClickMethod, PointerDeviceSettings, ScrollMethod, TapButtonMap};
use nirify_macros::SlintIndex;

/// Keyboard settings
#[derive(Debug, Clone, PartialEq)]
pub struct KeyboardSettings {
    /// Disable the keyboard device entirely (WARNING: may lock you out!)
    pub off: bool,
    pub xkb_layout: String,
    pub xkb_variant: String,
    pub xkb_model: String,
    pub xkb_rules: String,
    pub xkb_options: String,
    /// Path to a custom XKB keymap file (overrides other xkb settings)
    pub xkb_file: String,
    pub repeat_delay: i32,
    pub repeat_rate: i32,
    pub numlock: bool,
    pub track_layout: String,
}

impl Default for KeyboardSettings {
    fn default() -> Self {
        Self {
            off: false,
            xkb_layout: String::from("us"),
            xkb_variant: String::new(),
            xkb_model: String::new(),
            xkb_rules: String::new(),
            xkb_options: String::new(),
            xkb_file: String::new(),
            repeat_delay: DEFAULT_REPEAT_DELAY,
            repeat_rate: DEFAULT_REPEAT_RATE,
            numlock: false,
            track_layout: String::from("global"),
        }
    }
}

/// Mouse settings
#[derive(Debug, Clone, PartialEq)]
pub struct MouseSettings {
    /// Disable the mouse device entirely
    pub off: bool,
    pub natural_scroll: bool,
    pub left_handed: bool,
    pub accel_speed: f64,
    pub accel_profile: AccelProfile,
    pub scroll_factor: f64,
    /// Optional separate horizontal scroll factor (if different from vertical)
    pub scroll_factor_horizontal: Option<f64>,
    pub scroll_method: ScrollMethod,
    pub middle_emulation: bool,
    /// Mouse button for on-button-down scrolling (device-dependent)
    pub scroll_button: Option<i32>,
    /// Lock scroll button state (don't need to hold)
    pub scroll_button_lock: bool,
}

impl Default for MouseSettings {
    fn default() -> Self {
        Self {
            off: false,
            natural_scroll: false,
            left_handed: false,
            accel_speed: 0.0,
            accel_profile: AccelProfile::Adaptive,
            scroll_factor: 1.0,
            scroll_factor_horizontal: None,
            scroll_method: ScrollMethod::NoScroll, // Mouse doesn't use scroll method by default
            middle_emulation: false,
            scroll_button: None,
            scroll_button_lock: false,
        }
    }
}

/// Touchpad settings
#[derive(Debug, Clone, PartialEq)]
pub struct TouchpadSettings {
    /// Disable the touchpad device entirely
    pub off: bool,
    pub tap: bool,
    pub natural_scroll: bool,
    pub left_handed: bool,
    pub dwt: bool,
    pub dwtp: bool,
    pub drag: bool,
    pub drag_lock: bool,
    pub accel_speed: f64,
    pub accel_profile: AccelProfile,
    pub scroll_factor: f64,
    /// Optional separate horizontal scroll factor (if different from vertical)
    pub scroll_factor_horizontal: Option<f64>,
    pub scroll_method: ScrollMethod,
    pub click_method: ClickMethod,
    pub tap_button_map: TapButtonMap,
    pub middle_emulation: bool,
    pub disabled_on_external_mouse: bool,
    /// Mouse button for on-button-down scrolling (device-dependent)
    pub scroll_button: Option<i32>,
    /// Lock scroll button state (don't need to hold)
    pub scroll_button_lock: bool,
}

impl Default for TouchpadSettings {
    fn default() -> Self {
        Self {
            off: false,
            tap: true,
            natural_scroll: true,
            left_handed: false,
            dwt: true,
            dwtp: false,
            drag: true,
            drag_lock: false,
            accel_speed: 0.0,
            accel_profile: AccelProfile::Adaptive,
            scroll_factor: 1.0,
            scroll_factor_horizontal: None,
            scroll_method: ScrollMethod::TwoFinger,
            click_method: ClickMethod::ButtonAreas,
            tap_button_map: TapButtonMap::LeftRightMiddle,
            middle_emulation: false,
            disabled_on_external_mouse: false,
            scroll_button: None,
            scroll_button_lock: false,
        }
    }
}

/// Trackpoint settings (pointing stick / TrackPoint / nipple mouse)
#[derive(Debug, Clone, PartialEq)]
pub struct TrackpointSettings {
    /// Disable the trackpoint device entirely
    pub off: bool,
    pub accel_speed: f64,
    pub accel_profile: AccelProfile,
    pub natural_scroll: bool,
    pub left_handed: bool,
    pub scroll_method: ScrollMethod,
    /// Mouse button for on-button-down scrolling
    pub scroll_button: Option<i32>,
    /// Lock scroll button state (don't need to hold)
    pub scroll_button_lock: bool,
    pub middle_emulation: bool,
}

impl Default for TrackpointSettings {
    fn default() -> Self {
        Self {
            off: false,
            accel_speed: 0.0,
            accel_profile: AccelProfile::Adaptive,
            natural_scroll: false,
            left_handed: false,
            scroll_method: ScrollMethod::OnButtonDown,
            scroll_button: None, // Uses default middle button
            scroll_button_lock: false,
            middle_emulation: false,
        }
    }
}

/// Trackball settings
#[derive(Debug, Clone, PartialEq)]
pub struct TrackballSettings {
    /// Disable the trackball device entirely
    pub off: bool,
    pub accel_speed: f64,
    pub accel_profile: AccelProfile,
    pub natural_scroll: bool,
    pub left_handed: bool,
    pub scroll_method: ScrollMethod,
    /// Mouse button for on-button-down scrolling
    pub scroll_button: Option<i32>,
    /// Lock scroll button state (don't need to hold)
    pub scroll_button_lock: bool,
    pub middle_emulation: bool,
}

impl Default for TrackballSettings {
    fn default() -> Self {
        Self {
            off: false,
            accel_speed: 0.0,
            accel_profile: AccelProfile::Adaptive,
            natural_scroll: false,
            left_handed: false,
            scroll_method: ScrollMethod::OnButtonDown,
            scroll_button: None,
            scroll_button_lock: false,
            middle_emulation: false,
        }
    }
}

// ============================================================================
// PointerDeviceSettings trait implementations
// ============================================================================

/// Macro to implement the PointerDeviceSettings trait for a given settings struct.
///
/// This macro reduces boilerplate by generating the getter and setter methods
/// for the common pointer device fields.
macro_rules! impl_pointer_device_settings {
    ($struct_name:ident) => {
        impl PointerDeviceSettings for $struct_name {
            fn off(&self) -> bool {
                self.off
            }
            fn set_off(&mut self, value: bool) {
                self.off = value;
            }

            fn natural_scroll(&self) -> bool {
                self.natural_scroll
            }
            fn set_natural_scroll(&mut self, value: bool) {
                self.natural_scroll = value;
            }

            fn left_handed(&self) -> bool {
                self.left_handed
            }
            fn set_left_handed(&mut self, value: bool) {
                self.left_handed = value;
            }

            fn middle_emulation(&self) -> bool {
                self.middle_emulation
            }
            fn set_middle_emulation(&mut self, value: bool) {
                self.middle_emulation = value;
            }

            fn scroll_button_lock(&self) -> bool {
                self.scroll_button_lock
            }
            fn set_scroll_button_lock(&mut self, value: bool) {
                self.scroll_button_lock = value;
            }

            fn accel_speed(&self) -> f64 {
                self.accel_speed
            }
            fn set_accel_speed(&mut self, value: f64) {
                self.accel_speed = value;
            }

            fn accel_profile(&self) -> AccelProfile {
                self.accel_profile
            }
            fn set_accel_profile(&mut self, value: AccelProfile) {
                self.accel_profile = value;
            }

            fn scroll_method(&self) -> ScrollMethod {
                self.scroll_method
            }
            fn set_scroll_method(&mut self, value: ScrollMethod) {
                self.scroll_method = value;
            }

            fn scroll_button(&self) -> Option<i32> {
                self.scroll_button
            }
            fn set_scroll_button(&mut self, value: Option<i32>) {
                self.scroll_button = value;
            }
        }
    };
}

impl_pointer_device_settings!(MouseSettings);
impl_pointer_device_settings!(TouchpadSettings);
impl_pointer_device_settings!(TrackpointSettings);
impl_pointer_device_settings!(TrackballSettings);

/// Tablet (drawing tablet / stylus) settings
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TabletSettings {
    /// Disable the tablet device entirely
    pub off: bool,
    /// Map tablet to a specific output (monitor name)
    pub map_to_output: String,
    pub left_handed: bool,
    /// Calibration matrix (6 values for libinput transformation)
    pub calibration_matrix: Option<[f64; 6]>,
}

/// Touch screen settings
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TouchSettings {
    /// Disable the touch device entirely
    pub off: bool,
    /// Map touch input to a specific output (monitor name)
    pub map_to_output: String,
    /// Calibration matrix (6 values for libinput transformation)
    pub calibration_matrix: Option<[f64; 6]>,
}

/// Input device identifier for indexed callbacks
///
/// Maps to the 6 input device types. Used for unified callbacks
/// instead of having separate callbacks for each device type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, SlintIndex)]
pub enum InputDeviceId {
    #[default]
    #[slint_index(default)]
    Mouse,
    Touchpad,
    Trackpoint,
    Trackball,
    Tablet,
    Touch,
}

impl InputDeviceId {
    /// Get the human-readable name for logging
    pub fn name(&self) -> &'static str {
        match self {
            Self::Mouse => "Mouse",
            Self::Touchpad => "Touchpad",
            Self::Trackpoint => "Trackpoint",
            Self::Trackball => "Trackball",
            Self::Tablet => "Tablet",
            Self::Touch => "Touch",
        }
    }

    /// Get the settings category for dirty tracking
    pub fn category(&self) -> crate::config::SettingsCategory {
        match self {
            Self::Mouse => crate::config::SettingsCategory::Mouse,
            Self::Touchpad => crate::config::SettingsCategory::Touchpad,
            Self::Trackpoint => crate::config::SettingsCategory::Trackpoint,
            Self::Trackball => crate::config::SettingsCategory::Trackball,
            Self::Tablet => crate::config::SettingsCategory::Tablet,
            Self::Touch => crate::config::SettingsCategory::Touch,
        }
    }

    /// Check if this device type has the 'off' setting
    /// All input device types support the off setting
    pub fn has_off(&self) -> bool {
        true
    }

    /// Check if this device has pointer settings (accel, scroll method, etc.)
    pub fn has_pointer(&self) -> bool {
        matches!(
            self,
            Self::Mouse | Self::Touchpad | Self::Trackpoint | Self::Trackball
        )
    }

    /// Check if this device has scroll factor setting
    pub fn has_scroll_factor(&self) -> bool {
        matches!(self, Self::Mouse | Self::Touchpad)
    }

    /// Check if this device has tap settings (touchpad only)
    pub fn has_tap(&self) -> bool {
        matches!(self, Self::Touchpad)
    }

    /// Check if this device has scroll button settings
    pub fn has_scroll_button(&self) -> bool {
        matches!(
            self,
            Self::Mouse | Self::Touchpad | Self::Trackpoint | Self::Trackball
        )
    }

    /// Check if this device has map-to-output setting
    pub fn has_map_output(&self) -> bool {
        matches!(self, Self::Tablet | Self::Touch)
    }

    /// Check if this device has calibration matrix
    pub fn has_calibration(&self) -> bool {
        matches!(self, Self::Tablet | Self::Touch)
    }
}
