//! Behavior-related UI callbacks
//!
//! Handles focus follows mouse, warp mouse, column width, struts, and modifier keys.

use crate::config::category_section::Behavior;
use crate::config::{ColumnWidthType, Settings, SettingsCategory};
use crate::constants::*;
use crate::types::ModKey;
use crate::MainWindow;
use log::{debug, error};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::indices::{bool_to_center_focused_column, bool_to_warp_mouse_mode};
use super::super::macros::{
    register_bool_callbacks, register_clamped_callback, register_clamped_callbacks,
    register_enum_callback, SaveManager,
};

/// Set up behavior-related callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Boolean callbacks
    register_bool_callbacks!(
        ui,
        settings,
        save_manager,
        Behavior,
        [
            (
                on_focus_follows_mouse_toggled,
                focus_follows_mouse,
                "Focus follows mouse"
            ),
            (
                on_always_center_single_column_toggled,
                always_center_single_column,
                "Always center single column"
            ),
            (
                on_disable_power_key_handling_toggled,
                disable_power_key_handling,
                "Disable power key handling"
            ),
        ]
    );

    // Clamped callbacks (struts)
    register_clamped_callbacks!(
        ui,
        settings,
        save_manager,
        Behavior,
        [
            (
                on_strut_left_changed,
                strut_left,
                STRUT_SIZE_MIN,
                STRUT_SIZE_MAX,
                "Strut left: {}px"
            ),
            (
                on_strut_right_changed,
                strut_right,
                STRUT_SIZE_MIN,
                STRUT_SIZE_MAX,
                "Strut right: {}px"
            ),
            (
                on_strut_top_changed,
                strut_top,
                STRUT_SIZE_MIN,
                STRUT_SIZE_MAX,
                "Strut top: {}px"
            ),
            (
                on_strut_bottom_changed,
                strut_bottom,
                STRUT_SIZE_MIN,
                STRUT_SIZE_MAX,
                "Strut bottom: {}px"
            ),
        ]
    );

    // Warp mouse to focus - custom type conversion
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_warp_mouse_to_focus_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                s.behavior.warp_mouse_to_focus = bool_to_warp_mouse_mode(enabled);
                debug!("Warp mouse to focus: {}", enabled);
                save_manager.mark_dirty(SettingsCategory::Behavior);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Center focused column - custom type conversion
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_center_focused_column_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                s.behavior.center_focused_column = bool_to_center_focused_column(enabled);
                debug!("Center focused column: {}", enabled);
                save_manager.mark_dirty(SettingsCategory::Behavior);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Default column width type - enum mapping
    register_enum_callback!(
        ui,
        on_default_column_width_type_changed,
        settings,
        save_manager,
        SettingsCategory::Behavior,
        |s| s.behavior.default_column_width_type,
        ColumnWidthType,
        "Default column width type"
    );

    // Default column width proportion
    register_clamped_callback!(
        ui,
        on_default_column_width_proportion_changed,
        settings,
        save_manager,
        SettingsCategory::Behavior,
        COLUMN_PROPORTION_MIN,
        COLUMN_PROPORTION_MAX,
        |s| s.behavior.default_column_width_proportion,
        "Default column width proportion: {}"
    );

    // Default column width fixed
    register_clamped_callback!(
        ui,
        on_default_column_width_fixed_changed,
        settings,
        save_manager,
        SettingsCategory::Behavior,
        COLUMN_FIXED_MIN,
        COLUMN_FIXED_MAX,
        |s| s.behavior.default_column_width_fixed,
        "Default column width fixed: {}px"
    );

    // Modifier key - enum index
    register_enum_callback!(
        ui,
        on_mod_key_changed,
        settings,
        save_manager,
        SettingsCategory::Behavior,
        |s| s.behavior.mod_key,
        ModKey,
        "Mod key"
    );

    // Modifier key nested enabled - Option toggle
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_mod_key_nested_enabled_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                if enabled {
                    s.behavior.mod_key_nested = Some(ModKey::Alt);
                } else {
                    s.behavior.mod_key_nested = None;
                }
                debug!("Mod key nested enabled: {}", enabled);
                save_manager.mark_dirty(SettingsCategory::Behavior);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Modifier key nested value
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_mod_key_nested_changed(move |idx| match settings.lock() {
            Ok(mut s) => {
                s.behavior.mod_key_nested = Some(ModKey::from_index(idx));
                debug!("Mod key nested: {:?}", s.behavior.mod_key_nested);
                save_manager.mark_dirty(SettingsCategory::Behavior);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }
}
