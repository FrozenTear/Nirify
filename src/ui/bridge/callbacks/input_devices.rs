//! Input devices callbacks for trackpoint, trackball, tablet, and touch
//!
//! Handles settings for additional input devices beyond mouse and touchpad.

use crate::config::category_section::{Tablet, Touch, Trackball, Trackpoint};
use crate::config::SettingsCategory;
use crate::constants::*;
use crate::MainWindow;
use log::{debug, error};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::{
    register_bool_callbacks, register_clamped_f64_callbacks, register_enum_callbacks,
    register_option_i32_callbacks, SaveManager,
};
use crate::config::Settings;
use crate::types::{AccelProfile, ScrollMethod};

/// Set up all input device callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    setup_trackpoint(ui, settings.clone(), Rc::clone(&save_manager));
    setup_trackball(ui, settings.clone(), Rc::clone(&save_manager));
    setup_tablet(ui, settings.clone(), Rc::clone(&save_manager));
    setup_touch(ui, settings, save_manager);
}

/// Set up trackpoint callbacks
fn setup_trackpoint(
    ui: &MainWindow,
    settings: Arc<Mutex<Settings>>,
    save_manager: Rc<SaveManager>,
) {
    // Boolean callbacks
    register_bool_callbacks!(
        ui,
        settings,
        save_manager,
        Trackpoint,
        [
            (on_trackpoint_off_toggled, off, "Trackpoint off"),
            (
                on_trackpoint_natural_scroll_toggled,
                natural_scroll,
                "Trackpoint natural scroll"
            ),
            (
                on_trackpoint_left_handed_toggled,
                left_handed,
                "Trackpoint left-handed"
            ),
            (
                on_trackpoint_scroll_button_lock_toggled,
                scroll_button_lock,
                "Trackpoint scroll button lock"
            ),
            (
                on_trackpoint_middle_emulation_toggled,
                middle_emulation,
                "Trackpoint middle emulation"
            ),
        ]
    );

    // Clamped f64 callbacks
    register_clamped_f64_callbacks!(
        ui,
        settings,
        save_manager,
        Trackpoint,
        [(
            on_trackpoint_accel_speed_changed,
            accel_speed,
            ACCEL_SPEED_MIN,
            ACCEL_SPEED_MAX,
            "Trackpoint accel speed: {:.2}"
        ),]
    );

    // Enum index callbacks
    register_enum_callbacks!(
        ui,
        settings,
        save_manager,
        Trackpoint,
        [
            (
                on_trackpoint_accel_profile_changed,
                accel_profile,
                AccelProfile,
                "Trackpoint accel profile"
            ),
            (
                on_trackpoint_scroll_method_changed,
                scroll_method,
                ScrollMethod,
                "Trackpoint scroll method"
            ),
        ]
    );

    // Optional i32 callbacks (scroll button)
    register_option_i32_callbacks!(
        ui,
        settings,
        save_manager,
        Trackpoint,
        [(
            on_trackpoint_scroll_button_changed,
            scroll_button,
            "Trackpoint scroll button"
        ),]
    );
}

/// Set up trackball callbacks
fn setup_trackball(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Boolean callbacks
    register_bool_callbacks!(
        ui,
        settings,
        save_manager,
        Trackball,
        [
            (on_trackball_off_toggled, off, "Trackball off"),
            (
                on_trackball_natural_scroll_toggled,
                natural_scroll,
                "Trackball natural scroll"
            ),
            (
                on_trackball_left_handed_toggled,
                left_handed,
                "Trackball left-handed"
            ),
            (
                on_trackball_scroll_button_lock_toggled,
                scroll_button_lock,
                "Trackball scroll button lock"
            ),
            (
                on_trackball_middle_emulation_toggled,
                middle_emulation,
                "Trackball middle emulation"
            ),
        ]
    );

    // Clamped f64 callbacks
    register_clamped_f64_callbacks!(
        ui,
        settings,
        save_manager,
        Trackball,
        [(
            on_trackball_accel_speed_changed,
            accel_speed,
            ACCEL_SPEED_MIN,
            ACCEL_SPEED_MAX,
            "Trackball accel speed: {:.2}"
        ),]
    );

    // Enum index callbacks
    register_enum_callbacks!(
        ui,
        settings,
        save_manager,
        Trackball,
        [
            (
                on_trackball_accel_profile_changed,
                accel_profile,
                AccelProfile,
                "Trackball accel profile"
            ),
            (
                on_trackball_scroll_method_changed,
                scroll_method,
                ScrollMethod,
                "Trackball scroll method"
            ),
        ]
    );

    // Optional i32 callbacks (scroll button)
    register_option_i32_callbacks!(
        ui,
        settings,
        save_manager,
        Trackball,
        [(
            on_trackball_scroll_button_changed,
            scroll_button,
            "Trackball scroll button"
        ),]
    );
}

/// Set up tablet callbacks
fn setup_tablet(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Boolean callbacks
    register_bool_callbacks!(
        ui,
        settings,
        save_manager,
        Tablet,
        [
            (on_tablet_off_toggled, off, "Tablet off"),
            (
                on_tablet_left_handed_toggled,
                left_handed,
                "Tablet left-handed"
            ),
        ]
    );

    // Map to output - string (custom logic for debug format)
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_tablet_map_to_output_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                let output: String = val.into();
                s.tablet.map_to_output = output.clone();
                debug!(
                    "Tablet map to output: {}",
                    if output.is_empty() { "(all)" } else { &output }
                );
                save_manager.mark_dirty(SettingsCategory::Tablet);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Calibration toggle - Option toggle (custom logic)
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_tablet_calibration_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                if !enabled {
                    s.tablet.calibration_matrix = None;
                } else if s.tablet.calibration_matrix.is_none() {
                    s.tablet.calibration_matrix = Some([1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
                }
                debug!("Tablet calibration: {}", enabled);
                save_manager.mark_dirty(SettingsCategory::Tablet);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Calibration matrix (custom logic)
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_tablet_calibration_changed(move |m0, m1, m2, m3, m4, m5| match settings.lock() {
            Ok(mut s) => {
                s.tablet.calibration_matrix = Some([
                    m0 as f64, m1 as f64, m2 as f64, m3 as f64, m4 as f64, m5 as f64,
                ]);
                debug!("Tablet calibration matrix updated");
                save_manager.mark_dirty(SettingsCategory::Tablet);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }
}

/// Set up touch callbacks
fn setup_touch(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Boolean callbacks
    register_bool_callbacks!(
        ui,
        settings,
        save_manager,
        Touch,
        [(on_touch_off_toggled, off, "Touch off"),]
    );

    // Map to output - string (custom logic for debug format)
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_touch_map_to_output_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                let output: String = val.into();
                s.touch.map_to_output = output.clone();
                debug!(
                    "Touch map to output: {}",
                    if output.is_empty() { "(all)" } else { &output }
                );
                save_manager.mark_dirty(SettingsCategory::Touch);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Calibration toggle - Option toggle (custom logic)
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_touch_calibration_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                if !enabled {
                    s.touch.calibration_matrix = None;
                } else if s.touch.calibration_matrix.is_none() {
                    s.touch.calibration_matrix = Some([1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
                }
                debug!("Touch calibration: {}", enabled);
                save_manager.mark_dirty(SettingsCategory::Touch);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Calibration matrix (custom logic)
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_touch_calibration_changed(move |m0, m1, m2, m3, m4, m5| match settings.lock() {
            Ok(mut s) => {
                s.touch.calibration_matrix = Some([
                    m0 as f64, m1 as f64, m2 as f64, m3 as f64, m4 as f64, m5 as f64,
                ]);
                debug!("Touch calibration matrix updated");
                save_manager.mark_dirty(SettingsCategory::Touch);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }
}
