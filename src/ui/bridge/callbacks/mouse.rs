//! Mouse-related UI callbacks
//!
//! Handles mouse scrolling, acceleration, and button behavior.

use crate::config::category_section::Mouse;
use crate::constants::*;
use crate::MainWindow;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::{
    register_bool_callbacks, register_clamped_f64_callbacks, register_enum_callbacks,
    register_option_i32_callbacks, SaveManager,
};
use crate::config::Settings;
use crate::types::{AccelProfile, ScrollMethod};

/// Set up mouse-related callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Boolean callbacks
    register_bool_callbacks!(
        ui,
        settings,
        save_manager,
        Mouse,
        [
            (on_mouse_off_toggled, off, "Mouse off"),
            (
                on_mouse_natural_scroll_toggled,
                natural_scroll,
                "Mouse natural scroll"
            ),
            (
                on_mouse_left_handed_toggled,
                left_handed,
                "Mouse left-handed"
            ),
            (
                on_mouse_middle_emulation_toggled,
                middle_emulation,
                "Mouse middle emulation"
            ),
            (
                on_mouse_scroll_button_lock_toggled,
                scroll_button_lock,
                "Mouse scroll button lock"
            ),
        ]
    );

    // Clamped f64 callbacks
    register_clamped_f64_callbacks!(
        ui,
        settings,
        save_manager,
        Mouse,
        [
            (
                on_mouse_accel_speed_changed,
                accel_speed,
                ACCEL_SPEED_MIN,
                ACCEL_SPEED_MAX,
                "Mouse accel speed: {:.2}"
            ),
            (
                on_mouse_scroll_factor_changed,
                scroll_factor,
                SCROLL_FACTOR_MIN,
                SCROLL_FACTOR_MAX,
                "Mouse scroll factor: {:.2}x"
            ),
        ]
    );

    // Enum index callbacks
    register_enum_callbacks!(
        ui,
        settings,
        save_manager,
        Mouse,
        [
            (
                on_mouse_accel_profile_changed,
                accel_profile,
                AccelProfile,
                "Mouse accel profile"
            ),
            (
                on_mouse_scroll_method_changed,
                scroll_method,
                ScrollMethod,
                "Mouse scroll method"
            ),
        ]
    );

    // Optional i32 callbacks (scroll button)
    register_option_i32_callbacks!(
        ui,
        settings,
        save_manager,
        Mouse,
        [(
            on_mouse_scroll_button_changed,
            scroll_button,
            "Mouse scroll button"
        ),]
    );
}
