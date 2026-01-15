//! Recent windows (Alt-Tab) UI callbacks (v25.05+)
//!
//! Handles recent windows switcher settings including timing, highlight, and preview options.

use crate::config::{Settings, SettingsCategory};
use crate::MainWindow;
use log::{debug, error};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::converters::slint_color_to_color;
use super::super::macros::SaveManager;

/// Set up recent windows-related callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Enable/disable toggle
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_recent_windows_enabled_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                s.recent_windows.off = !enabled;
                debug!("Recent windows enabled: {}", enabled);
                save_manager.mark_dirty(SettingsCategory::RecentWindows);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Debounce delay
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_recent_windows_debounce_ms_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                s.recent_windows.debounce_ms = val.max(0);
                debug!("Recent windows debounce: {}ms", val);
                save_manager.mark_dirty(SettingsCategory::RecentWindows);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Open delay
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_recent_windows_open_delay_ms_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                s.recent_windows.open_delay_ms = val.max(0);
                debug!("Recent windows open delay: {}ms", val);
                save_manager.mark_dirty(SettingsCategory::RecentWindows);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Highlight active color
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_recent_windows_highlight_active_color_changed(move |color| match settings.lock() {
            Ok(mut s) => {
                s.recent_windows.highlight.active_color = slint_color_to_color(color);
                debug!("Recent windows highlight active color changed");
                save_manager.mark_dirty(SettingsCategory::RecentWindows);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Highlight urgent color
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_recent_windows_highlight_urgent_color_changed(move |color| match settings.lock() {
            Ok(mut s) => {
                s.recent_windows.highlight.urgent_color = slint_color_to_color(color);
                debug!("Recent windows highlight urgent color changed");
                save_manager.mark_dirty(SettingsCategory::RecentWindows);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Highlight padding
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_recent_windows_highlight_padding_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                s.recent_windows.highlight.padding = val.max(0);
                debug!("Recent windows highlight padding: {}px", val);
                save_manager.mark_dirty(SettingsCategory::RecentWindows);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Highlight corner radius
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_recent_windows_highlight_corner_radius_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                s.recent_windows.highlight.corner_radius = val.max(0);
                debug!("Recent windows highlight corner radius: {}px", val);
                save_manager.mark_dirty(SettingsCategory::RecentWindows);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Previews max height
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_recent_windows_previews_max_height_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                s.recent_windows.previews.max_height = val.max(50);
                debug!("Recent windows previews max height: {}px", val);
                save_manager.mark_dirty(SettingsCategory::RecentWindows);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Previews max scale
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_recent_windows_previews_max_scale_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                s.recent_windows.previews.max_scale = (val as f64).clamp(0.1, 1.0);
                debug!("Recent windows previews max scale: {:.0}%", val * 100.0);
                save_manager.mark_dirty(SettingsCategory::RecentWindows);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }
}
