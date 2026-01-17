//! Dynamic overview UI callbacks
//!
//! Handles overview settings using model-driven dynamic UI.

use crate::config::models::WorkspaceShadow;
use crate::config::{Settings, SettingsCategory};
use crate::constants::{OVERVIEW_ZOOM_MAX, OVERVIEW_ZOOM_MIN};
use crate::types::Color;
use crate::{MainWindow, OverviewSettingModel};
use log::{debug, error};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::converters::color_to_slint_color;
use super::super::macros::SaveManager;

// ============================================================================
// HELPER FUNCTIONS FOR CREATING SETTING MODELS
// ============================================================================

fn make_toggle(
    id: &str,
    label: &str,
    desc: &str,
    value: bool,
    visible: bool,
) -> OverviewSettingModel {
    OverviewSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 0,
        bool_value: value,
        visible,
        ..Default::default()
    }
}

fn make_slider_int(
    id: &str,
    label: &str,
    desc: &str,
    value: i32,
    min: f32,
    max: f32,
    suffix: &str,
    visible: bool,
) -> OverviewSettingModel {
    OverviewSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 1,
        int_value: value,
        min_value: min,
        max_value: max,
        suffix: suffix.into(),
        use_float: false,
        visible,
        ..Default::default()
    }
}

fn make_slider_float(
    id: &str,
    label: &str,
    desc: &str,
    value: f32,
    min: f32,
    max: f32,
    suffix: &str,
    visible: bool,
) -> OverviewSettingModel {
    OverviewSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 1,
        float_value: value,
        min_value: min,
        max_value: max,
        suffix: suffix.into(),
        use_float: true,
        visible,
        ..Default::default()
    }
}

fn make_color(
    id: &str,
    label: &str,
    desc: &str,
    color: &Color,
    visible: bool,
) -> OverviewSettingModel {
    OverviewSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 4,
        text_value: color.to_hex().into(),
        placeholder: "#RRGGBBAA".into(),
        color_value: color_to_slint_color(color),
        visible,
        ..Default::default()
    }
}

// ============================================================================
// SECTION POPULATION FUNCTIONS
// ============================================================================

/// Populate appearance settings model (zoom)
fn populate_appearance_settings(settings: &Settings) -> ModelRc<OverviewSettingModel> {
    let zoom = settings.overview.zoom as f32;

    let models = vec![make_slider_float(
        "zoom",
        "Zoom level",
        "Scale of windows in the overview (lower = more zoomed out)",
        zoom,
        OVERVIEW_ZOOM_MIN as f32,
        OVERVIEW_ZOOM_MAX as f32,
        "%",
        true,
    )];

    ModelRc::new(VecModel::from(models))
}

/// Populate backdrop settings model
fn populate_backdrop_settings(settings: &Settings) -> ModelRc<OverviewSettingModel> {
    let backdrop_enabled = settings.overview.backdrop_color.is_some();
    let backdrop_color = settings
        .overview
        .backdrop_color
        .as_ref()
        .cloned()
        .unwrap_or_else(|| Color {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        });

    let models = vec![
        make_toggle(
            "backdrop_enabled",
            "Custom backdrop color",
            "Use a custom color behind the overview instead of default",
            backdrop_enabled,
            true,
        ),
        make_color(
            "backdrop_color",
            "Backdrop color",
            "Color shown behind windows in overview",
            &backdrop_color,
            backdrop_enabled,
        ),
    ];

    ModelRc::new(VecModel::from(models))
}

/// Populate workspace shadow settings model
fn populate_shadow_settings(settings: &Settings) -> ModelRc<OverviewSettingModel> {
    let shadow_enabled = settings.overview.workspace_shadow.is_some();
    let shadow = settings
        .overview
        .workspace_shadow
        .as_ref()
        .cloned()
        .unwrap_or_default();

    let show_details = shadow_enabled;

    let models = vec![
        make_toggle(
            "shadow_enabled",
            "Enable workspace shadow",
            "Show a shadow around workspaces in the overview",
            shadow_enabled,
            true,
        ),
        make_slider_int(
            "shadow_softness",
            "Softness",
            "How blurred the shadow edge is",
            shadow.softness,
            0.0,
            100.0,
            "",
            show_details,
        ),
        make_slider_int(
            "shadow_spread",
            "Spread",
            "How far the shadow extends",
            shadow.spread,
            0.0,
            50.0,
            "",
            show_details,
        ),
        make_slider_int(
            "shadow_offset_x",
            "Offset X",
            "Horizontal shadow offset",
            shadow.offset_x,
            -50.0,
            50.0,
            "",
            show_details,
        ),
        make_slider_int(
            "shadow_offset_y",
            "Offset Y",
            "Vertical shadow offset",
            shadow.offset_y,
            -50.0,
            50.0,
            "",
            show_details,
        ),
        make_color(
            "shadow_color",
            "Shadow color",
            "Color of the workspace shadow (with alpha)",
            &shadow.color,
            show_details,
        ),
    ];

    ModelRc::new(VecModel::from(models))
}

/// Sync all UI models from settings
fn sync_all_models(ui: &MainWindow, settings: &Settings) {
    ui.set_overview_appearance_settings(populate_appearance_settings(settings));
    ui.set_overview_backdrop_settings(populate_backdrop_settings(settings));
    ui.set_overview_shadow_settings(populate_shadow_settings(settings));
}

// ============================================================================
// CALLBACK SETUP
// ============================================================================

/// Set up dynamic overview callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Generic toggle callback
    {
        let settings = Arc::clone(&settings);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_overview_setting_toggle_changed(move |id, value| {
            let id_str = id.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    #[allow(unused_assignments)]
                    let mut needs_model_refresh = false;

                    match id_str.as_str() {
                        "backdrop_enabled" => {
                            if value {
                                // Set a default color if enabling and none set
                                if s.overview.backdrop_color.is_none() {
                                    s.overview.backdrop_color = Some(Color {
                                        r: 0,
                                        g: 0,
                                        b: 0,
                                        a: 255,
                                    });
                                }
                            } else {
                                s.overview.backdrop_color = None;
                            }
                            needs_model_refresh = true;
                        }
                        "shadow_enabled" => {
                            if value {
                                if s.overview.workspace_shadow.is_none() {
                                    s.overview.workspace_shadow = Some(WorkspaceShadow::default());
                                }
                            } else {
                                s.overview.workspace_shadow = None;
                            }
                            needs_model_refresh = true;
                        }
                        _ => {
                            debug!("Unknown overview toggle setting: {}", id_str);
                            return;
                        }
                    }

                    // Refresh models if visibility changed
                    if needs_model_refresh {
                        if let Some(ui) = ui_weak.upgrade() {
                            sync_all_models(&ui, &s);
                        }
                    }

                    debug!("Overview toggle {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Overview);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Generic slider int callback
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        ui.on_overview_setting_slider_int_changed(move |id, value| {
            let id_str = id.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    match id_str.as_str() {
                        "shadow_softness" => {
                            if let Some(ref mut ws) = s.overview.workspace_shadow {
                                ws.softness = value.clamp(0, 100);
                            }
                        }
                        "shadow_spread" => {
                            if let Some(ref mut ws) = s.overview.workspace_shadow {
                                ws.spread = value.clamp(0, 50);
                            }
                        }
                        "shadow_offset_x" => {
                            if let Some(ref mut ws) = s.overview.workspace_shadow {
                                ws.offset_x = value.clamp(-50, 50);
                            }
                        }
                        "shadow_offset_y" => {
                            if let Some(ref mut ws) = s.overview.workspace_shadow {
                                ws.offset_y = value.clamp(-50, 50);
                            }
                        }
                        _ => {
                            debug!("Unknown overview slider int setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Overview slider int {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Overview);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Generic slider float callback
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        ui.on_overview_setting_slider_float_changed(move |id, value| {
            let id_str = id.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    match id_str.as_str() {
                        "zoom" => {
                            let clamped =
                                (value as f64).clamp(OVERVIEW_ZOOM_MIN, OVERVIEW_ZOOM_MAX);
                            s.overview.zoom = clamped;
                            debug!("Overview zoom: {:.0}%", clamped * 100.0);
                        }
                        _ => {
                            debug!("Unknown overview slider float setting: {}", id_str);
                            return;
                        }
                    }

                    save_manager.mark_dirty(SettingsCategory::Overview);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Generic text callback (handles color hex input)
    {
        let settings = Arc::clone(&settings);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_overview_setting_text_changed(move |id, value| {
            let id_str = id.to_string();
            let value_str = value.to_string();

            match settings.lock() {
                Ok(mut s) => {
                    match id_str.as_str() {
                        "backdrop_color" => {
                            if let Some(color) = Color::from_hex(&value_str) {
                                s.overview.backdrop_color = Some(color);
                                // Update model to show new color preview
                                if let Some(ui) = ui_weak.upgrade() {
                                    ui.set_overview_backdrop_settings(populate_backdrop_settings(
                                        &s,
                                    ));
                                }
                            }
                        }
                        "shadow_color" => {
                            if let Some(ref mut ws) = s.overview.workspace_shadow {
                                if let Some(color) = Color::from_hex(&value_str) {
                                    ws.color = color;
                                    // Update model to show new color preview
                                    if let Some(ui) = ui_weak.upgrade() {
                                        ui.set_overview_shadow_settings(populate_shadow_settings(
                                            &s,
                                        ));
                                    }
                                }
                            }
                        }
                        _ => {
                            debug!("Unknown overview text setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Overview text {} = {}", id_str, value_str);
                    save_manager.mark_dirty(SettingsCategory::Overview);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Generic color callback (for color picker widget if used)
    {
        let settings = Arc::clone(&settings);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_overview_setting_color_changed(move |id, color| {
            let id_str = id.to_string();

            match settings.lock() {
                Ok(mut s) => {
                    let rust_color = Color {
                        r: color.red(),
                        g: color.green(),
                        b: color.blue(),
                        a: color.alpha(),
                    };

                    match id_str.as_str() {
                        "backdrop_color" => {
                            s.overview.backdrop_color = Some(rust_color);
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_overview_backdrop_settings(populate_backdrop_settings(&s));
                            }
                        }
                        "shadow_color" => {
                            if let Some(ref mut ws) = s.overview.workspace_shadow {
                                ws.color = rust_color;
                                if let Some(ui) = ui_weak.upgrade() {
                                    ui.set_overview_shadow_settings(populate_shadow_settings(&s));
                                }
                            }
                        }
                        _ => {
                            debug!("Unknown overview color setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Overview color {} changed", id_str);
                    save_manager.mark_dirty(SettingsCategory::Overview);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }
}

// ============================================================================
// PUBLIC SYNC FUNCTIONS
// ============================================================================

/// Public function to sync all overview models for the given settings
pub fn sync_overview_models(ui: &MainWindow, settings: &Settings) {
    sync_all_models(ui, settings);
}
