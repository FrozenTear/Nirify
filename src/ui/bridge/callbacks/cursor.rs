//! Dynamic cursor UI callbacks
//!
//! Handles cursor configuration using model-driven dynamic UI.

use crate::config::{Settings, SettingsCategory};
use crate::constants::{CURSOR_SIZE_MAX, CURSOR_SIZE_MIN, HIDE_INACTIVE_MAX, HIDE_INACTIVE_MIN};
use crate::CursorSettingModel;
use crate::MainWindow;
use log::{debug, error};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::SaveManager;

// Generate helper functions for CursorSettingModel
crate::impl_setting_builders!(CursorSettingModel);

// ============================================================================
// MODEL POPULATION FUNCTIONS
// ============================================================================

/// Populate cursor theme section settings model
fn populate_theme_settings(settings: &Settings) -> ModelRc<CursorSettingModel> {
    let cursor = &settings.cursor;

    let models = vec![
        make_text(
            "cursor_theme",
            "Theme name",
            "XCursor theme (leave empty for system default)",
            &cursor.theme,
            "System default",
            true,
        ),
        make_slider_int(
            "cursor_size",
            "Cursor size",
            "Size of the cursor in pixels",
            cursor.size,
            CURSOR_SIZE_MIN as f32,
            CURSOR_SIZE_MAX as f32,
            "px",
            true,
        ),
    ];

    ModelRc::new(VecModel::from(models))
}

/// Populate cursor visibility section settings model
fn populate_visibility_settings(settings: &Settings) -> ModelRc<CursorSettingModel> {
    let cursor = &settings.cursor;
    let hide_after_enabled = cursor.hide_after_inactive_ms.is_some();
    let hide_after_ms = cursor.hide_after_inactive_ms.unwrap_or(1000);

    let models = vec![
        make_toggle(
            "hide_when_typing",
            "Hide when typing",
            "Hide cursor while pressing keyboard keys",
            cursor.hide_when_typing,
            true,
        ),
        make_toggle(
            "hide_after_inactive_enabled",
            "Hide after inactivity",
            "Automatically hide cursor after a period of no movement",
            hide_after_enabled,
            true,
        ),
        make_slider_int(
            "hide_after_inactive_ms",
            "Inactivity timeout",
            "Time before cursor hides",
            hide_after_ms,
            HIDE_INACTIVE_MIN as f32,
            HIDE_INACTIVE_MAX as f32,
            "ms",
            hide_after_enabled,
        ),
    ];

    ModelRc::new(VecModel::from(models))
}

/// Sync all cursor UI models from settings
fn sync_cursor_models(ui: &MainWindow, settings: &Settings) {
    ui.set_cursor_theme_settings(populate_theme_settings(settings));
    ui.set_cursor_visibility_settings(populate_visibility_settings(settings));
}

// ============================================================================
// PUBLIC FUNCTIONS FOR SYNC
// ============================================================================

/// Public function to sync cursor models (called from sync.rs)
pub fn sync_all_models(ui: &MainWindow, settings: &Settings) {
    sync_cursor_models(ui, settings);
}

// ============================================================================
// CALLBACK SETUP
// ============================================================================

/// Set up dynamic cursor callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Generic toggle callback
    {
        let settings = Arc::clone(&settings);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_cursor_setting_toggle_changed(move |id, value| {
            let id_str = id.as_str();

            // Perform settings update and collect data for UI update
            let ui_update = match settings.lock() {
                Ok(mut s) => {
                    let mut needs_model_refresh = false;

                    match id_str {
                        "hide_when_typing" => {
                            s.cursor.hide_when_typing = value;
                        }
                        "hide_after_inactive_enabled" => {
                            if value {
                                if s.cursor.hide_after_inactive_ms.is_none() {
                                    s.cursor.hide_after_inactive_ms = Some(1000);
                                }
                            } else {
                                s.cursor.hide_after_inactive_ms = None;
                            }
                            needs_model_refresh = true;
                        }
                        _ => {
                            debug!("Unknown cursor toggle setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Cursor toggle {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Cursor);
                    save_manager.request_save();

                    // Clone data for UI update if needed
                    if needs_model_refresh {
                        Some(s.clone())
                    } else {
                        None
                    }
                }
                Err(e) => {
                    error!("Settings lock error: {}", e);
                    return;
                }
            };

            // UI updates happen after lock is released
            if let Some(settings_clone) = ui_update {
                if let Some(ui) = ui_weak.upgrade() {
                    sync_cursor_models(&ui, &settings_clone);
                }
            }
        });
    }

    // Generic slider int callback
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        ui.on_cursor_setting_slider_int_changed(move |id, value| {
            let id_str = id.as_str();
            match settings.lock() {
                Ok(mut s) => {
                    match id_str {
                        "cursor_size" => {
                            s.cursor.size = value.clamp(CURSOR_SIZE_MIN, CURSOR_SIZE_MAX);
                        }
                        "hide_after_inactive_ms" => {
                            s.cursor.hide_after_inactive_ms =
                                Some(value.clamp(HIDE_INACTIVE_MIN, HIDE_INACTIVE_MAX));
                        }
                        _ => {
                            debug!("Unknown cursor slider int setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Cursor slider int {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Cursor);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Generic text callback
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        ui.on_cursor_setting_text_changed(move |id, value| {
            let id_str = id.as_str();
            let value_str = value.to_string();

            match settings.lock() {
                Ok(mut s) => {
                    match id_str {
                        "cursor_theme" => {
                            s.cursor.theme = value_str.clone();
                        }
                        _ => {
                            debug!("Unknown cursor text setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Cursor text {} = {}", id_str, value_str);
                    save_manager.mark_dirty(SettingsCategory::Cursor);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }
}
