//! Touchpad-related UI callbacks
//!
//! Handles tap, scrolling, click method, drag, and acceleration settings.

use crate::config::category_section::Touchpad;
use crate::constants::*;
use crate::MainWindow;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::{
    register_bool_callbacks, register_clamped_f64_callbacks, register_enum_callbacks,
    register_option_i32_callbacks, SaveManager,
};
use crate::config::Settings;
use crate::types::{AccelProfile, ClickMethod, ScrollMethod, TapButtonMap};

/// Set up touchpad-related callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Boolean callbacks
    register_bool_callbacks!(
        ui,
        settings,
        save_manager,
        Touchpad,
        [
            (on_touchpad_off_toggled, off, "Touchpad off"),
            (on_touchpad_tap_toggled, tap, "Touchpad tap"),
            (
                on_touchpad_natural_scroll_toggled,
                natural_scroll,
                "Touchpad natural scroll"
            ),
            (
                on_touchpad_middle_emulation_toggled,
                middle_emulation,
                "Touchpad middle emulation"
            ),
            (
                on_touchpad_left_handed_toggled,
                left_handed,
                "Touchpad left-handed"
            ),
            (on_touchpad_dwt_toggled, dwt, "Touchpad DWT"),
            (on_touchpad_dwtp_toggled, dwtp, "Touchpad DWTP"),
            (on_touchpad_drag_toggled, drag, "Touchpad drag"),
            (
                on_touchpad_drag_lock_toggled,
                drag_lock,
                "Touchpad drag lock"
            ),
            (
                on_touchpad_disabled_on_external_mouse_toggled,
                disabled_on_external_mouse,
                "Touchpad disabled on external mouse"
            ),
            (
                on_touchpad_scroll_button_lock_toggled,
                scroll_button_lock,
                "Touchpad scroll button lock"
            ),
        ]
    );

    // Clamped f64 callbacks
    register_clamped_f64_callbacks!(
        ui,
        settings,
        save_manager,
        Touchpad,
        [
            (
                on_touchpad_scroll_factor_changed,
                scroll_factor,
                SCROLL_FACTOR_MIN,
                SCROLL_FACTOR_MAX,
                "Touchpad scroll factor: {:.2}x"
            ),
            (
                on_touchpad_accel_speed_changed,
                accel_speed,
                ACCEL_SPEED_MIN,
                ACCEL_SPEED_MAX,
                "Touchpad accel speed: {:.2}"
            ),
        ]
    );

    // Enum index callbacks
    register_enum_callbacks!(
        ui,
        settings,
        save_manager,
        Touchpad,
        [
            (
                on_touchpad_tap_button_map_changed,
                tap_button_map,
                TapButtonMap,
                "Touchpad tap button map"
            ),
            (
                on_touchpad_scroll_method_changed,
                scroll_method,
                ScrollMethod,
                "Touchpad scroll method"
            ),
            (
                on_touchpad_click_method_changed,
                click_method,
                ClickMethod,
                "Touchpad click method"
            ),
            (
                on_touchpad_accel_profile_changed,
                accel_profile,
                AccelProfile,
                "Touchpad accel profile"
            ),
        ]
    );

    // Optional i32 callbacks (scroll button)
    register_option_i32_callbacks!(
        ui,
        settings,
        save_manager,
        Touchpad,
        [(
            on_touchpad_scroll_button_changed,
            scroll_button,
            "Touchpad scroll button"
        ),]
    );
}
