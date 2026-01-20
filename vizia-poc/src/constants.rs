//! Constants for value bounds and defaults (simplified for PoC)

// Keyboard
pub const KEYBOARD_REPEAT_RATE_MIN: i32 = 1;
pub const KEYBOARD_REPEAT_RATE_MAX: i32 = 100;
pub const KEYBOARD_REPEAT_RATE_DEFAULT: i32 = 25;

pub const KEYBOARD_REPEAT_DELAY_MIN: i32 = 100;
pub const KEYBOARD_REPEAT_DELAY_MAX: i32 = 1000;
pub const KEYBOARD_REPEAT_DELAY_DEFAULT: i32 = 600;

// Mouse
pub const MOUSE_ACCEL_SPEED_MIN: f64 = -1.0;
pub const MOUSE_ACCEL_SPEED_MAX: f64 = 1.0;
pub const MOUSE_ACCEL_SPEED_DEFAULT: f64 = 0.0;

pub const MOUSE_SCROLL_FACTOR_MIN: f64 = 0.1;
pub const MOUSE_SCROLL_FACTOR_MAX: f64 = 10.0;
pub const MOUSE_SCROLL_FACTOR_DEFAULT: f64 = 1.0;

// Touchpad
pub const TOUCHPAD_ACCEL_SPEED_MIN: f64 = -1.0;
pub const TOUCHPAD_ACCEL_SPEED_MAX: f64 = 1.0;
pub const TOUCHPAD_ACCEL_SPEED_DEFAULT: f64 = 0.0;

pub const TOUCHPAD_SCROLL_FACTOR_MIN: f64 = 0.1;
pub const TOUCHPAD_SCROLL_FACTOR_MAX: f64 = 10.0;
pub const TOUCHPAD_SCROLL_FACTOR_DEFAULT: f64 = 1.0;
