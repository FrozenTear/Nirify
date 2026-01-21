//! Dynamic appearance UI callbacks
//!
//! Handles appearance settings using model-driven dynamic UI.

use crate::config::models::AppearanceSettings;
use crate::config::{Settings, SettingsCategory};
use crate::constants::{
    BORDER_THICKNESS_MAX, BORDER_THICKNESS_MIN, CORNER_RADIUS_MAX, CORNER_RADIUS_MIN,
    FOCUS_RING_WIDTH_MAX, FOCUS_RING_WIDTH_MIN, GAP_SIZE_MAX, GAP_SIZE_MIN,
};
use crate::types::{Color, ColorOrGradient};
use crate::{AppearanceSettingModel, MainWindow};
use log::{debug, error};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::converters::{color_to_slint_color, slint_color_to_color};
use super::super::macros::SaveManager;

// Generate helper functions for AppearanceSettingModel
crate::impl_setting_builders!(AppearanceSettingModel);

// ============================================================================
// SECTION ENUM FOR SELECTIVE SYNC
// ============================================================================

/// Identifies which section of appearance settings to refresh
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppearanceSection {
    FocusRing,
    Border,
    Background,
    Gaps,
    Corners,
    All,
}

// ============================================================================
// CUSTOM HELPER FOR COLOR (appearance uses color_to_slint_color)
// ============================================================================

fn make_color_with_slint(
    id: &str,
    label: &str,
    desc: &str,
    color: &Color,
    visible: bool,
) -> AppearanceSettingModel {
    AppearanceSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 4,
        text_value: color.to_hex().into(),
        color_value: color_to_slint_color(color),
        placeholder: "#RRGGBB".into(),
        visible,
        ..Default::default()
    }
}

fn make_color_from_gradient(
    id: &str,
    label: &str,
    desc: &str,
    color_or_gradient: &ColorOrGradient,
    visible: bool,
) -> AppearanceSettingModel {
    let color = color_or_gradient.primary_color();
    make_color_with_slint(id, label, desc, color, visible)
}

// ============================================================================
// MODEL POPULATION FUNCTIONS
// ============================================================================

/// Populate focus ring settings model
fn populate_focus_ring_settings(
    appearance: &AppearanceSettings,
) -> ModelRc<AppearanceSettingModel> {
    let enabled = appearance.focus_ring_enabled;

    let settings = vec![
        make_toggle(
            "focus_ring_enabled",
            "Enable focus ring",
            "Show a colored ring around the focused window",
            enabled,
            true,
        ),
        make_slider_float(
            "focus_ring_width",
            "Ring width",
            "Thickness of the focus ring in pixels",
            appearance.focus_ring_width,
            FOCUS_RING_WIDTH_MIN,
            FOCUS_RING_WIDTH_MAX,
            "px",
            enabled,
        ),
        make_color_from_gradient(
            "focus_ring_active",
            "Active color",
            "Color when window is focused",
            &appearance.focus_ring_active,
            enabled,
        ),
        make_color_from_gradient(
            "focus_ring_inactive",
            "Inactive color",
            "Color when window is not focused",
            &appearance.focus_ring_inactive,
            enabled,
        ),
        make_color_from_gradient(
            "focus_ring_urgent",
            "Urgent color",
            "Color when window needs attention",
            &appearance.focus_ring_urgent,
            enabled,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

/// Populate border settings model
fn populate_border_settings(appearance: &AppearanceSettings) -> ModelRc<AppearanceSettingModel> {
    let enabled = appearance.border_enabled;

    let settings = vec![
        make_toggle(
            "border_enabled",
            "Enable window border",
            "Show a border around windows (inside the focus ring)",
            enabled,
            true,
        ),
        make_slider_float(
            "border_thickness",
            "Border width",
            "Thickness of the window border in pixels",
            appearance.border_thickness,
            BORDER_THICKNESS_MIN,
            BORDER_THICKNESS_MAX,
            "px",
            enabled,
        ),
        make_color_from_gradient(
            "border_active",
            "Active color",
            "Border color when window is focused",
            &appearance.border_active,
            enabled,
        ),
        make_color_from_gradient(
            "border_inactive",
            "Inactive color",
            "Border color when window is not focused",
            &appearance.border_inactive,
            enabled,
        ),
        make_color_from_gradient(
            "border_urgent",
            "Urgent color",
            "Border color when window needs attention",
            &appearance.border_urgent,
            enabled,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

/// Populate background settings model
fn populate_background_settings(
    appearance: &AppearanceSettings,
) -> ModelRc<AppearanceSettingModel> {
    let color = appearance.background_color.unwrap_or(Color {
        r: 0x1e,
        g: 0x1e,
        b: 0x2e,
        a: 0xff,
    });

    let settings = vec![make_color_with_slint(
        "background_color",
        "Window background",
        "Default background color for windows",
        &color,
        true,
    )];
    ModelRc::new(VecModel::from(settings))
}

/// Populate gaps settings model
fn populate_gaps_settings(appearance: &AppearanceSettings) -> ModelRc<AppearanceSettingModel> {
    let settings = vec![make_slider_float(
        "gaps",
        "Gaps",
        "Space between windows and screen edges",
        appearance.gaps,
        GAP_SIZE_MIN,
        GAP_SIZE_MAX,
        "px",
        true,
    )];
    ModelRc::new(VecModel::from(settings))
}

/// Populate corners settings model
fn populate_corners_settings(appearance: &AppearanceSettings) -> ModelRc<AppearanceSettingModel> {
    let settings = vec![make_slider_float(
        "corner_radius",
        "Corner radius",
        "Roundness of window corners",
        appearance.corner_radius,
        CORNER_RADIUS_MIN,
        CORNER_RADIUS_MAX,
        "px",
        true,
    )];
    ModelRc::new(VecModel::from(settings))
}

/// Sync a specific section of appearance UI models from settings
///
/// This function allows selective refresh of UI models, avoiding the overhead
/// of refreshing all sections when only one has changed.
pub fn sync_models(ui: &MainWindow, appearance: &AppearanceSettings, section: AppearanceSection) {
    match section {
        AppearanceSection::FocusRing => {
            ui.set_appearance_focus_ring_settings(populate_focus_ring_settings(appearance));
        }
        AppearanceSection::Border => {
            ui.set_appearance_border_settings(populate_border_settings(appearance));
        }
        AppearanceSection::Background => {
            ui.set_appearance_background_settings(populate_background_settings(appearance));
        }
        AppearanceSection::Gaps => {
            ui.set_appearance_gaps_settings(populate_gaps_settings(appearance));
        }
        AppearanceSection::Corners => {
            ui.set_appearance_corners_settings(populate_corners_settings(appearance));
        }
        AppearanceSection::All => {
            sync_all_models(ui, appearance);
        }
    }
}

/// Sync all UI models from settings
pub fn sync_all_models(ui: &MainWindow, appearance: &AppearanceSettings) {
    ui.set_appearance_focus_ring_settings(populate_focus_ring_settings(appearance));
    ui.set_appearance_border_settings(populate_border_settings(appearance));
    ui.set_appearance_background_settings(populate_background_settings(appearance));
    ui.set_appearance_gaps_settings(populate_gaps_settings(appearance));
    ui.set_appearance_corners_settings(populate_corners_settings(appearance));
}

// ============================================================================
// CALLBACK SETUP
// ============================================================================

/// Set up dynamic appearance callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Toggle callback
    {
        let settings = Arc::clone(&settings);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_appearance_setting_toggle_changed(move |id, value| {
            let id_str = id.as_str();

            // Clone data needed for UI update, then release lock before UI operations
            let refresh_info = match settings.lock() {
                Ok(mut s) => {
                    let section = match id_str {
                        "focus_ring_enabled" => {
                            s.appearance.focus_ring_enabled = value;
                            Some(AppearanceSection::FocusRing)
                        }
                        "border_enabled" => {
                            s.appearance.border_enabled = value;
                            Some(AppearanceSection::Border)
                        }
                        _ => {
                            debug!("Unknown appearance toggle setting: {}", id_str);
                            return;
                        }
                    };

                    debug!("Appearance toggle {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Appearance);
                    save_manager.request_save();

                    // Clone data for UI update if needed
                    section.map(|sec| (s.appearance.clone(), sec))
                }
                Err(e) => {
                    error!("Settings lock error: {}", e);
                    return;
                }
            };

            // UI updates happen after lock is released
            if let Some((appearance, section)) = refresh_info {
                if let Some(ui) = ui_weak.upgrade() {
                    sync_models(&ui, &appearance, section);
                }
            }
        });
    }

    // Slider float callback
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        ui.on_appearance_setting_slider_float_changed(move |id, value| {
            let id_str = id.as_str();
            match settings.lock() {
                Ok(mut s) => {
                    match id_str {
                        "focus_ring_width" => {
                            s.appearance.focus_ring_width =
                                value.clamp(FOCUS_RING_WIDTH_MIN, FOCUS_RING_WIDTH_MAX);
                        }
                        "border_thickness" => {
                            s.appearance.border_thickness =
                                value.clamp(BORDER_THICKNESS_MIN, BORDER_THICKNESS_MAX);
                        }
                        "gaps" => {
                            s.appearance.gaps = value.clamp(GAP_SIZE_MIN, GAP_SIZE_MAX);
                        }
                        "corner_radius" => {
                            s.appearance.corner_radius =
                                value.clamp(CORNER_RADIUS_MIN, CORNER_RADIUS_MAX);
                        }
                        _ => {
                            debug!("Unknown appearance slider float setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Appearance slider float {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Appearance);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Text callback (handles color hex input)
    {
        let settings = Arc::clone(&settings);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_appearance_setting_text_changed(move |id, value| {
            let id_str = id.as_str();
            let value_str = value.to_string();

            // Try to parse as color for color fields
            let color_result = Color::from_hex(&value_str);

            // Clone data needed for UI update, then release lock before UI operations
            let refresh_info = match settings.lock() {
                Ok(mut s) => {
                    let section = match id_str {
                        "focus_ring_active" => {
                            if let Some(color) = color_result {
                                s.appearance.focus_ring_active.set_color(color);
                                Some(AppearanceSection::FocusRing)
                            } else {
                                None
                            }
                        }
                        "focus_ring_inactive" => {
                            if let Some(color) = color_result {
                                s.appearance.focus_ring_inactive.set_color(color);
                                Some(AppearanceSection::FocusRing)
                            } else {
                                None
                            }
                        }
                        "focus_ring_urgent" => {
                            if let Some(color) = color_result {
                                s.appearance.focus_ring_urgent.set_color(color);
                                Some(AppearanceSection::FocusRing)
                            } else {
                                None
                            }
                        }
                        "border_active" => {
                            if let Some(color) = color_result {
                                s.appearance.border_active.set_color(color);
                                Some(AppearanceSection::Border)
                            } else {
                                None
                            }
                        }
                        "border_inactive" => {
                            if let Some(color) = color_result {
                                s.appearance.border_inactive.set_color(color);
                                Some(AppearanceSection::Border)
                            } else {
                                None
                            }
                        }
                        "border_urgent" => {
                            if let Some(color) = color_result {
                                s.appearance.border_urgent.set_color(color);
                                Some(AppearanceSection::Border)
                            } else {
                                None
                            }
                        }
                        "background_color" => {
                            if let Some(color) = color_result {
                                s.appearance.background_color = Some(color);
                                Some(AppearanceSection::Background)
                            } else {
                                None
                            }
                        }
                        _ => {
                            debug!("Unknown appearance text setting: {}", id_str);
                            return;
                        }
                    };

                    // Only save if we had a valid color
                    if color_result.is_some() {
                        debug!("Appearance text {} = {}", id_str, value_str);
                        save_manager.mark_dirty(SettingsCategory::Appearance);
                        save_manager.request_save();
                    }

                    // Clone data for UI update if needed
                    section.map(|sec| (s.appearance.clone(), sec))
                }
                Err(e) => {
                    error!("Settings lock error: {}", e);
                    return;
                }
            };

            // UI updates happen after lock is released
            if let Some((appearance, section)) = refresh_info {
                if let Some(ui) = ui_weak.upgrade() {
                    sync_models(&ui, &appearance, section);
                }
            }
        });
    }

    // Color callback (from swatch picker)
    {
        let settings = Arc::clone(&settings);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_appearance_setting_color_changed(move |id, color| {
            let id_str = id.as_str();
            let rust_color = slint_color_to_color(color);

            // Clone data needed for UI update, then release lock before UI operations
            let refresh_info = match settings.lock() {
                Ok(mut s) => {
                    let section = match id_str {
                        "focus_ring_active" => {
                            s.appearance.focus_ring_active.set_color(rust_color);
                            AppearanceSection::FocusRing
                        }
                        "focus_ring_inactive" => {
                            s.appearance.focus_ring_inactive.set_color(rust_color);
                            AppearanceSection::FocusRing
                        }
                        "focus_ring_urgent" => {
                            s.appearance.focus_ring_urgent.set_color(rust_color);
                            AppearanceSection::FocusRing
                        }
                        "border_active" => {
                            s.appearance.border_active.set_color(rust_color);
                            AppearanceSection::Border
                        }
                        "border_inactive" => {
                            s.appearance.border_inactive.set_color(rust_color);
                            AppearanceSection::Border
                        }
                        "border_urgent" => {
                            s.appearance.border_urgent.set_color(rust_color);
                            AppearanceSection::Border
                        }
                        "background_color" => {
                            s.appearance.background_color = Some(rust_color);
                            AppearanceSection::Background
                        }
                        _ => {
                            debug!("Unknown appearance color setting: {}", id_str);
                            return;
                        }
                    };

                    debug!("Appearance color {} changed", id_str);
                    save_manager.mark_dirty(SettingsCategory::Appearance);
                    save_manager.request_save();

                    // Clone data for UI update
                    (s.appearance.clone(), section)
                }
                Err(e) => {
                    error!("Settings lock error: {}", e);
                    return;
                }
            };

            // UI updates happen after lock is released
            if let Some(ui) = ui_weak.upgrade() {
                sync_models(&ui, &refresh_info.0, refresh_info.1);
            }
        });
    }
}
