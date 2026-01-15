//! Cursor-related UI callbacks
//!
//! Handles cursor theme, size, and hide behavior.

use crate::config::category_section::Cursor;
use crate::constants::*;
use crate::MainWindow;
use log::{debug, error};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::{
    register_bool_callbacks, register_clamped_callbacks, register_string_callbacks, SaveManager,
};
use crate::config::{Settings, SettingsCategory};

/// Set up cursor-related callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // String callbacks
    register_string_callbacks!(
        ui,
        settings,
        save_manager,
        Cursor,
        [(on_cursor_theme_changed, theme, "Cursor theme"),]
    );

    // Clamped numeric callbacks
    register_clamped_callbacks!(
        ui,
        settings,
        save_manager,
        Cursor,
        [(
            on_cursor_size_changed,
            size,
            CURSOR_SIZE_MIN,
            CURSOR_SIZE_MAX,
            "Cursor size: {}px"
        ),]
    );

    // Boolean callbacks
    register_bool_callbacks!(
        ui,
        settings,
        save_manager,
        Cursor,
        [(
            on_hide_when_typing_toggled,
            hide_when_typing,
            "Hide cursor when typing"
        ),]
    );

    // Hide after inactive toggle - custom logic for Option<i32>
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_hide_after_inactive_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                if enabled {
                    if s.cursor.hide_after_inactive_ms.is_none() {
                        s.cursor.hide_after_inactive_ms = Some(1000);
                    }
                } else {
                    s.cursor.hide_after_inactive_ms = None;
                }
                debug!("Hide cursor after inactive: {}", enabled);
                save_manager.mark_dirty(SettingsCategory::Cursor);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Hide after inactive ms - custom logic for Option<i32>
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_hide_after_inactive_ms_changed(move |ms| {
            let clamped = ms.clamp(HIDE_INACTIVE_MIN, HIDE_INACTIVE_MAX);
            match settings.lock() {
                Ok(mut s) => {
                    s.cursor.hide_after_inactive_ms = Some(clamped);
                    debug!("Hide cursor after: {}ms", clamped);
                    save_manager.mark_dirty(SettingsCategory::Cursor);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }
}
