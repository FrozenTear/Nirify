//! Overview-related UI callbacks
//!
//! Handles overview zoom, backdrop color, and workspace shadow (Phase 8).

use crate::config::{Settings, SettingsCategory, WorkspaceShadow};
use crate::constants::*;
use crate::MainWindow;
use log::{debug, error};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::converters::slint_color_to_color;
use super::super::macros::SaveManager;

/// Set up overview-related callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Overview zoom
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_overview_zoom_changed(move |zoom| {
            let clamped = (zoom as f64).clamp(OVERVIEW_ZOOM_MIN, OVERVIEW_ZOOM_MAX);
            match settings.lock() {
                Ok(mut s) => {
                    s.overview.zoom = clamped;
                    debug!("Overview zoom: {:.0}%", clamped * 100.0);
                    save_manager.mark_dirty(SettingsCategory::Overview);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Overview backdrop toggle
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_overview_backdrop_toggled(move |enabled| {
            match settings.lock() {
                Ok(mut s) => {
                    if enabled {
                        // Set a default color if enabling and none set
                        if s.overview.backdrop_color.is_none() {
                            s.overview.backdrop_color = Some(crate::types::Color {
                                r: 0,
                                g: 0,
                                b: 0,
                                a: 255,
                            });
                        }
                    } else {
                        s.overview.backdrop_color = None;
                    }
                    debug!("Overview backdrop enabled: {}", enabled);
                    save_manager.mark_dirty(SettingsCategory::Overview);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Overview backdrop color
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_overview_backdrop_color_changed(move |color| match settings.lock() {
            Ok(mut s) => {
                s.overview.backdrop_color = Some(slint_color_to_color(color));
                debug!("Overview backdrop color changed");
                save_manager.mark_dirty(SettingsCategory::Overview);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Phase 8: Workspace shadow enabled
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_overview_workspace_shadow_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                if enabled {
                    if s.overview.workspace_shadow.is_none() {
                        s.overview.workspace_shadow = Some(WorkspaceShadow::default());
                    }
                } else {
                    s.overview.workspace_shadow = None;
                }
                debug!("Overview workspace shadow enabled: {}", enabled);
                save_manager.mark_dirty(SettingsCategory::Overview);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Phase 8: Workspace shadow softness
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_overview_workspace_shadow_softness_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                if let Some(ref mut ws) = s.overview.workspace_shadow {
                    ws.softness = val as i32;
                    debug!("Overview workspace shadow softness: {}", val);
                    save_manager.mark_dirty(SettingsCategory::Overview);
                    save_manager.request_save();
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Phase 8: Workspace shadow spread
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_overview_workspace_shadow_spread_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                if let Some(ref mut ws) = s.overview.workspace_shadow {
                    ws.spread = val as i32;
                    debug!("Overview workspace shadow spread: {}", val);
                    save_manager.mark_dirty(SettingsCategory::Overview);
                    save_manager.request_save();
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Phase 8: Workspace shadow offset X
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_overview_workspace_shadow_offset_x_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                if let Some(ref mut ws) = s.overview.workspace_shadow {
                    ws.offset_x = val as i32;
                    debug!("Overview workspace shadow offset X: {}", val);
                    save_manager.mark_dirty(SettingsCategory::Overview);
                    save_manager.request_save();
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Phase 8: Workspace shadow offset Y
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_overview_workspace_shadow_offset_y_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                if let Some(ref mut ws) = s.overview.workspace_shadow {
                    ws.offset_y = val as i32;
                    debug!("Overview workspace shadow offset Y: {}", val);
                    save_manager.mark_dirty(SettingsCategory::Overview);
                    save_manager.request_save();
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Phase 8: Workspace shadow color
    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_overview_workspace_shadow_color_changed(move |color| match settings.lock() {
            Ok(mut s) => {
                if let Some(ref mut ws) = s.overview.workspace_shadow {
                    ws.color = slint_color_to_color(color);
                    debug!("Overview workspace shadow color changed");
                    save_manager.mark_dirty(SettingsCategory::Overview);
                    save_manager.request_save();
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }
}
