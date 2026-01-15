//! Miscellaneous UI callbacks
//!
//! Handles CSD preferences, screenshot path, clipboard, hotkey overlay, and XWayland settings.

use crate::config::category_section::Miscellaneous;
use crate::config::models::XWaylandSatelliteConfig;
use crate::MainWindow;
use log::{debug, error};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::{register_bool_callbacks, register_string_callbacks, SaveManager};
use crate::config::{Settings, SettingsCategory};

/// Set up miscellaneous callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Boolean callbacks
    register_bool_callbacks!(
        ui,
        settings,
        save_manager,
        Miscellaneous,
        [
            (on_prefer_no_csd_toggled, prefer_no_csd, "Prefer no CSD"),
            (
                on_disable_primary_clipboard_toggled,
                disable_primary_clipboard,
                "Disable primary clipboard"
            ),
            (
                on_hotkey_overlay_skip_toggled,
                hotkey_overlay_skip_at_startup,
                "Hotkey overlay skip at startup"
            ),
            (
                on_hotkey_overlay_hide_not_bound_toggled,
                hotkey_overlay_hide_not_bound,
                "Hotkey overlay hide not bound"
            ),
            (
                on_config_notification_disable_failed_toggled,
                config_notification_disable_failed,
                "Config notification disable failed"
            ),
            (
                on_spawn_sh_at_startup_toggled,
                spawn_sh_at_startup,
                "Spawn sh at startup"
            ),
        ]
    );

    // String callbacks
    register_string_callbacks!(
        ui,
        settings,
        save_manager,
        Miscellaneous,
        [(
            on_screenshot_path_changed,
            screenshot_path,
            "Screenshot path"
        ),]
    );

    // XWayland satellite mode - custom enum handling
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_xwayland_satellite_mode_changed(move |idx| match settings.lock() {
            Ok(mut s) => {
                s.miscellaneous.xwayland_satellite = match idx {
                    1 => XWaylandSatelliteConfig::Off,
                    2 => XWaylandSatelliteConfig::CustomPath(String::new()),
                    _ => XWaylandSatelliteConfig::Default,
                };
                debug!(
                    "XWayland satellite mode: {:?}",
                    s.miscellaneous.xwayland_satellite
                );
                save_manager.mark_dirty(SettingsCategory::Miscellaneous);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // XWayland satellite custom path
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_xwayland_satellite_path_changed(move |path| match settings.lock() {
            Ok(mut s) => {
                s.miscellaneous.xwayland_satellite =
                    XWaylandSatelliteConfig::CustomPath(path.to_string());
                debug!("XWayland satellite path: {}", path);
                save_manager.mark_dirty(SettingsCategory::Miscellaneous);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Show debug options toggle - UI only (not persisted to config)
    ui.on_show_debug_options_toggled(move |enabled| {
        debug!("Show debug options: {}", enabled);
    });
}
