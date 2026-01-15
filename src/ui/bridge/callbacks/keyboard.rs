//! Keyboard-related UI callbacks
//!
//! Handles XKB layout settings, key repeat, numlock, and track layout.

use crate::config::category_section::Keyboard;
use crate::constants::*;
use crate::MainWindow;
use log::{debug, error};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::indices::TRACK_LAYOUT_GLOBAL;
use super::super::macros::{
    register_bool_callbacks, register_clamped_callbacks, register_string_callbacks, SaveManager,
};
use crate::config::{Settings, SettingsCategory};

/// Set up keyboard-related callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // String callbacks (XKB settings)
    register_string_callbacks!(
        ui,
        settings,
        save_manager,
        Keyboard,
        [
            (on_xkb_layout_changed, xkb_layout, "XKB layout"),
            (on_xkb_variant_changed, xkb_variant, "XKB variant"),
            (on_xkb_model_changed, xkb_model, "XKB model"),
            (on_xkb_rules_changed, xkb_rules, "XKB rules"),
            (on_xkb_options_changed, xkb_options, "XKB options"),
            (on_xkb_file_changed, xkb_file, "XKB file"),
        ]
    );

    // Clamped numeric callbacks
    register_clamped_callbacks!(
        ui,
        settings,
        save_manager,
        Keyboard,
        [
            (
                on_repeat_delay_changed,
                repeat_delay,
                REPEAT_DELAY_MIN,
                REPEAT_DELAY_MAX,
                "Repeat delay: {}ms"
            ),
            (
                on_repeat_rate_changed,
                repeat_rate,
                REPEAT_RATE_MIN,
                REPEAT_RATE_MAX,
                "Repeat rate: {}/s"
            ),
        ]
    );

    // Boolean callbacks
    register_bool_callbacks!(
        ui,
        settings,
        save_manager,
        Keyboard,
        [
            (on_keyboard_off_toggled, off, "Keyboard off"),
            (on_numlock_toggled, numlock, "NumLock on startup"),
        ]
    );

    // Track layout - special string mapping
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_track_layout_changed(move |idx| match settings.lock() {
            Ok(mut s) => {
                s.keyboard.track_layout = if idx == TRACK_LAYOUT_GLOBAL {
                    String::from("global")
                } else {
                    String::from("window")
                };
                debug!("Track layout: {}", s.keyboard.track_layout);
                save_manager.mark_dirty(SettingsCategory::Keyboard);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }
}
