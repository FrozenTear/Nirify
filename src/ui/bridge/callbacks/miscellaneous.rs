//! Dynamic Miscellaneous UI callbacks
//!
//! Handles miscellaneous settings using model-driven dynamic UI.

use crate::config::models::XWaylandSatelliteConfig;
use crate::config::{Settings, SettingsCategory};
use crate::{MainWindow, MiscSettingModel};
use log::{debug, error};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::SaveManager;

// Generate helper functions for MiscSettingModel
crate::impl_setting_builders!(MiscSettingModel);

// ============================================================================
// SECTION ENUM FOR SELECTIVE SYNC
// ============================================================================

/// Identifies which section of miscellaneous settings to refresh
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MiscSection {
    Decorations,
    Screenshots,
    ScreenshotPath,
    Clipboard,
    HotkeyOverlay,
    ConfigNotification,
    Startup,
    XWayland,
    XWaylandPath,
    Developer,
    All,
}

// ============================================================================
// Functions to populate section models from Settings struct
// ============================================================================

/// Populate window decorations section settings
fn populate_decorations_settings(settings: &Settings) -> ModelRc<MiscSettingModel> {
    let misc = &settings.miscellaneous;
    let items = vec![make_toggle(
        "prefer_no_csd",
        "Prefer server-side decorations",
        "Niri draws window borders instead of letting apps draw their own title bars",
        misc.prefer_no_csd,
        true,
    )];
    ModelRc::new(VecModel::from(items))
}

/// Populate screenshots section settings (empty array, uses full-width text)
fn populate_screenshots_settings() -> ModelRc<MiscSettingModel> {
    // No regular settings in this section, just the full-width text input
    ModelRc::new(VecModel::from(vec![]))
}

/// Populate screenshot path setting (full-width text input)
fn populate_screenshot_path_setting(settings: &Settings) -> MiscSettingModel {
    let misc = &settings.miscellaneous;
    make_text(
        "screenshot_path",
        "Save path",
        "Where to save screenshots (supports strftime format)",
        &misc.screenshot_path,
        "~/Pictures/Screenshots/...",
        true,
    )
}

/// Populate clipboard section settings
fn populate_clipboard_settings(settings: &Settings) -> ModelRc<MiscSettingModel> {
    let misc = &settings.miscellaneous;
    let items = vec![make_toggle(
        "disable_primary_clipboard",
        "Disable primary clipboard",
        "Disable middle-click paste functionality",
        misc.disable_primary_clipboard,
        true,
    )];
    ModelRc::new(VecModel::from(items))
}

/// Populate hotkey overlay section settings
fn populate_hotkey_overlay_settings(settings: &Settings) -> ModelRc<MiscSettingModel> {
    let misc = &settings.miscellaneous;
    let items = vec![
        make_toggle(
            "hotkey_overlay_skip",
            "Skip at startup",
            "Don't show hotkey help when niri starts",
            misc.hotkey_overlay_skip_at_startup,
            true,
        ),
        make_toggle(
            "hotkey_overlay_hide_not_bound",
            "Hide unbound actions",
            "Only show actions that have keybindings assigned",
            misc.hotkey_overlay_hide_not_bound,
            true,
        ),
    ];
    ModelRc::new(VecModel::from(items))
}

/// Populate config notification section settings
fn populate_config_notification_settings(settings: &Settings) -> ModelRc<MiscSettingModel> {
    let misc = &settings.miscellaneous;
    let items = vec![make_toggle(
        "config_notification_disable_failed",
        "Disable failed config notifications",
        "Suppress notifications when config reload fails",
        misc.config_notification_disable_failed,
        true,
    )];
    ModelRc::new(VecModel::from(items))
}

/// Populate startup section settings
fn populate_startup_settings(settings: &Settings) -> ModelRc<MiscSettingModel> {
    let misc = &settings.miscellaneous;
    let items = vec![make_toggle(
        "spawn_sh_at_startup",
        "Use shell for spawn commands",
        "Execute spawn-at-startup commands through a shell",
        misc.spawn_sh_at_startup,
        true,
    )];
    ModelRc::new(VecModel::from(items))
}

/// Populate XWayland section settings
fn populate_xwayland_settings(settings: &Settings) -> ModelRc<MiscSettingModel> {
    let misc = &settings.miscellaneous;
    let xwayland_index = match &misc.xwayland_satellite {
        XWaylandSatelliteConfig::Default => 0,
        XWaylandSatelliteConfig::Off => 1,
        XWaylandSatelliteConfig::CustomPath(_) => 2,
    };

    let items = vec![make_combo(
        "xwayland_satellite_mode",
        "XWayland satellite",
        "XWayland compatibility layer for X11 apps",
        xwayland_index,
        &["Default", "Disabled", "Custom path"],
        true,
    )];
    ModelRc::new(VecModel::from(items))
}

/// Populate XWayland path setting (full-width text input, conditional visibility)
fn populate_xwayland_path_setting(settings: &Settings) -> MiscSettingModel {
    let misc = &settings.miscellaneous;
    let (path, visible) = match &misc.xwayland_satellite {
        XWaylandSatelliteConfig::CustomPath(p) => (p.as_str(), true),
        _ => ("", false),
    };
    make_text(
        "xwayland_satellite_path",
        "Custom binary path",
        "",
        path,
        "/usr/bin/xwayland-satellite",
        visible,
    )
}

/// Populate developer options section settings
/// Note: show_debug_options is UI-only, not persisted to config
fn populate_developer_settings(show_debug: bool) -> ModelRc<MiscSettingModel> {
    let items = vec![make_toggle(
        "show_debug_options",
        "Show debug options",
        "Show the Debug page in the System navigation",
        show_debug,
        true,
    )];
    ModelRc::new(VecModel::from(items))
}

// ============================================================================
// Sync all UI models from Settings
// ============================================================================

/// Sync a specific section of miscellaneous UI models from settings
///
/// This function allows selective refresh of UI models, avoiding the overhead
/// of refreshing all sections when only one has changed.
pub fn sync_models(ui: &MainWindow, settings: &Settings, show_debug: bool, section: MiscSection) {
    match section {
        MiscSection::Decorations => {
            ui.set_misc_decorations_settings(populate_decorations_settings(settings));
        }
        MiscSection::Screenshots => {
            ui.set_misc_screenshots_settings(populate_screenshots_settings());
        }
        MiscSection::ScreenshotPath => {
            ui.set_misc_screenshot_path_setting(populate_screenshot_path_setting(settings));
        }
        MiscSection::Clipboard => {
            ui.set_misc_clipboard_settings(populate_clipboard_settings(settings));
        }
        MiscSection::HotkeyOverlay => {
            ui.set_misc_hotkey_overlay_settings(populate_hotkey_overlay_settings(settings));
        }
        MiscSection::ConfigNotification => {
            ui.set_misc_config_notification_settings(populate_config_notification_settings(
                settings,
            ));
        }
        MiscSection::Startup => {
            ui.set_misc_startup_settings(populate_startup_settings(settings));
        }
        MiscSection::XWayland => {
            ui.set_misc_xwayland_settings(populate_xwayland_settings(settings));
        }
        MiscSection::XWaylandPath => {
            ui.set_misc_xwayland_path_setting(populate_xwayland_path_setting(settings));
        }
        MiscSection::Developer => {
            ui.set_misc_developer_settings(populate_developer_settings(show_debug));
        }
        MiscSection::All => {
            sync_all_models(ui, settings, show_debug);
        }
    }
}

/// Sync all miscellaneous settings UI models from Settings struct
fn sync_all_models(ui: &MainWindow, settings: &Settings, show_debug: bool) {
    ui.set_misc_decorations_settings(populate_decorations_settings(settings));
    ui.set_misc_screenshots_settings(populate_screenshots_settings());
    ui.set_misc_screenshot_path_setting(populate_screenshot_path_setting(settings));
    ui.set_misc_clipboard_settings(populate_clipboard_settings(settings));
    ui.set_misc_hotkey_overlay_settings(populate_hotkey_overlay_settings(settings));
    ui.set_misc_config_notification_settings(populate_config_notification_settings(settings));
    ui.set_misc_startup_settings(populate_startup_settings(settings));
    ui.set_misc_xwayland_settings(populate_xwayland_settings(settings));
    ui.set_misc_xwayland_path_setting(populate_xwayland_path_setting(settings));
    ui.set_misc_developer_settings(populate_developer_settings(show_debug));
}

// ============================================================================
// Setup function - Register all callbacks
// ============================================================================

/// Set up dynamic miscellaneous callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Track show_debug_options state (UI-only, not persisted)
    let show_debug = Rc::new(std::cell::Cell::new(false));

    // Generic toggle callback
    {
        let settings = Arc::clone(&settings);
        let show_debug = Rc::clone(&show_debug);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_misc_setting_toggle_changed(move |id, value| {
            let id_str = id.as_str();

            // Handle show_debug_options separately (UI-only, controls Debug page visibility)
            if id_str == "show_debug_options" {
                show_debug.set(value);
                debug!("Show debug options: {}", value);

                // Clone settings for UI update, then release lock
                let settings_clone = settings.lock().ok().map(|s| s.clone());

                // UI updates happen after lock is released
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_show_debug_options(value);
                    if let Some(s) = settings_clone {
                        // Only refresh the developer section, not all sections
                        sync_models(&ui, &s, value, MiscSection::Developer);
                    }
                }
                return;
            }

            // For other toggles, no UI refresh needed (they don't affect visibility of other settings)
            match settings.lock() {
                Ok(mut s) => {
                    let changed = match id_str {
                        "prefer_no_csd" => {
                            s.miscellaneous.prefer_no_csd = value;
                            true
                        }
                        "disable_primary_clipboard" => {
                            s.miscellaneous.disable_primary_clipboard = value;
                            true
                        }
                        "hotkey_overlay_skip" => {
                            s.miscellaneous.hotkey_overlay_skip_at_startup = value;
                            true
                        }
                        "hotkey_overlay_hide_not_bound" => {
                            s.miscellaneous.hotkey_overlay_hide_not_bound = value;
                            true
                        }
                        "config_notification_disable_failed" => {
                            s.miscellaneous.config_notification_disable_failed = value;
                            true
                        }
                        "spawn_sh_at_startup" => {
                            s.miscellaneous.spawn_sh_at_startup = value;
                            true
                        }
                        _ => {
                            debug!("Unknown misc toggle setting: {}", id_str);
                            false
                        }
                    };

                    if changed {
                        debug!("Misc toggle {} = {}", id_str, value);
                        save_manager.mark_dirty(SettingsCategory::Miscellaneous);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Generic combo callback
    {
        let settings = Arc::clone(&settings);
        let show_debug = Rc::clone(&show_debug);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_misc_setting_combo_changed(move |id, index| {
            let id_str = id.as_str();

            // Clone data needed for UI update, then release lock before UI operations
            let refresh_info = match settings.lock() {
                Ok(mut s) => {
                    let section = match id_str {
                        "xwayland_satellite_mode" => {
                            s.miscellaneous.xwayland_satellite = match index {
                                1 => XWaylandSatelliteConfig::Off,
                                2 => XWaylandSatelliteConfig::CustomPath(String::new()),
                                _ => XWaylandSatelliteConfig::Default,
                            };
                            // Need to refresh XWaylandPath since its visibility depends on mode
                            Some(MiscSection::XWaylandPath)
                        }
                        _ => {
                            debug!("Unknown misc combo setting: {}", id_str);
                            None
                        }
                    };

                    if section.is_some() {
                        debug!("Misc combo {} = {}", id_str, index);
                        save_manager.mark_dirty(SettingsCategory::Miscellaneous);
                        save_manager.request_save();
                    }

                    // Clone data for UI update if needed
                    section.map(|sec| (s.clone(), sec))
                }
                Err(e) => {
                    error!("Settings lock error: {}", e);
                    return;
                }
            };

            // UI updates happen after lock is released
            if let Some((s, section)) = refresh_info {
                if let Some(ui) = ui_weak.upgrade() {
                    sync_models(&ui, &s, show_debug.get(), section);
                }
            }
        });
    }

    // Generic text callback
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        ui.on_misc_setting_text_changed(move |id, value| {
            let id_str = id.as_str();
            let value_str = value.to_string();

            match settings.lock() {
                Ok(mut s) => {
                    let mut changed = true;

                    match id_str {
                        "screenshot_path" => {
                            s.miscellaneous.screenshot_path = value_str.clone();
                        }
                        "xwayland_satellite_path" => {
                            s.miscellaneous.xwayland_satellite =
                                XWaylandSatelliteConfig::CustomPath(value_str.clone());
                        }
                        _ => {
                            debug!("Unknown misc text setting: {}", id_str);
                            changed = false;
                        }
                    }

                    if changed {
                        debug!("Misc text {} = {}", id_str, value_str);
                        save_manager.mark_dirty(SettingsCategory::Miscellaneous);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }
}

// ============================================================================
// Public API for sync.rs integration
// ============================================================================

/// Sync all miscellaneous dynamic UI models from Settings
/// Called from sync.rs during initial load and when settings change externally
/// Sync all miscellaneous dynamic UI models, reading show_debug from MainWindow
pub fn sync_misc_dynamic_models_auto(ui: &MainWindow, settings: &Settings) {
    let show_debug = ui.get_show_debug_options();
    sync_all_models(ui, settings, show_debug);
}
