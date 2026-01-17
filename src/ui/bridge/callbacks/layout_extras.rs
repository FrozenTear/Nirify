//! Dynamic layout extras UI callbacks
//!
//! Handles shadow settings, tab indicator, and insert hint using model-driven dynamic UI.

use crate::config::models::{LayoutExtrasSettings, TabIndicatorPosition};
use crate::config::{Settings, SettingsCategory};
use crate::types::{Color, ColorOrGradient};
use crate::{LayoutSettingModel, MainWindow};
use log::{debug, error};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

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
) -> LayoutSettingModel {
    LayoutSettingModel {
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
) -> LayoutSettingModel {
    LayoutSettingModel {
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
) -> LayoutSettingModel {
    LayoutSettingModel {
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

fn make_combo(
    id: &str,
    label: &str,
    desc: &str,
    index: i32,
    options: &[&str],
    visible: bool,
) -> LayoutSettingModel {
    let opts: Vec<SharedString> = options.iter().map(|s| (*s).into()).collect();
    LayoutSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 2,
        combo_index: index,
        combo_options: ModelRc::new(VecModel::from(opts)),
        visible,
        ..Default::default()
    }
}

fn make_color(id: &str, label: &str, desc: &str, hex: &str, visible: bool) -> LayoutSettingModel {
    LayoutSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 4,
        text_value: hex.into(),
        placeholder: "#RRGGBBAA".into(),
        visible,
        ..Default::default()
    }
}

// ============================================================================
// MODEL POPULATION FUNCTIONS
// ============================================================================

/// Populate shadow settings model
pub fn populate_shadow_settings(extras: &LayoutExtrasSettings) -> ModelRc<LayoutSettingModel> {
    let shadow = &extras.shadow;
    let show_details = shadow.enabled;

    let settings = vec![
        make_toggle(
            "shadow_enabled",
            "Enable shadows",
            "Draw shadows behind windows",
            shadow.enabled,
            true,
        ),
        make_slider_int(
            "shadow_softness",
            "Softness",
            "Blur amount for shadow edges",
            shadow.softness,
            0.0,
            100.0,
            "px",
            show_details,
        ),
        make_slider_int(
            "shadow_spread",
            "Spread",
            "How far shadow extends from window",
            shadow.spread,
            0.0,
            50.0,
            "px",
            show_details,
        ),
        make_slider_int(
            "shadow_offset_x",
            "Offset X",
            "Horizontal shadow offset",
            shadow.offset_x,
            -50.0,
            50.0,
            "px",
            show_details,
        ),
        make_slider_int(
            "shadow_offset_y",
            "Offset Y",
            "Vertical shadow offset",
            shadow.offset_y,
            -50.0,
            50.0,
            "px",
            show_details,
        ),
        make_toggle(
            "shadow_draw_behind",
            "Draw behind window",
            "Draw shadow behind transparent windows",
            shadow.draw_behind_window,
            show_details,
        ),
        make_color(
            "shadow_color",
            "Shadow color",
            "Color for focused window shadow",
            &shadow.color.to_hex(),
            show_details,
        ),
        make_color(
            "shadow_inactive_color",
            "Inactive shadow",
            "Color for unfocused window shadow",
            &shadow.inactive_color.to_hex(),
            show_details,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

/// Populate tab indicator settings model
pub fn populate_tab_indicator_settings(
    extras: &LayoutExtrasSettings,
) -> ModelRc<LayoutSettingModel> {
    let tab = &extras.tab_indicator;
    let show_details = tab.enabled;

    let position_index = match tab.position {
        TabIndicatorPosition::Left => 0,
        TabIndicatorPosition::Right => 1,
        TabIndicatorPosition::Top => 2,
        TabIndicatorPosition::Bottom => 3,
    };

    let settings = vec![
        make_toggle(
            "tab_enabled",
            "Enable tab indicator",
            "Show indicator for tabbed windows",
            tab.enabled,
            true,
        ),
        make_toggle(
            "tab_hide_single",
            "Hide when single tab",
            "Only show when multiple tabs exist",
            tab.hide_when_single_tab,
            show_details,
        ),
        make_toggle(
            "tab_within_column",
            "Place within column",
            "Draw indicator inside the window column",
            tab.place_within_column,
            show_details,
        ),
        make_combo(
            "tab_position",
            "Position",
            "Which side to show the indicator",
            position_index,
            &["Left", "Right", "Top", "Bottom"],
            show_details,
        ),
        make_slider_int(
            "tab_width",
            "Width",
            "Thickness of the indicator bar",
            tab.width,
            1.0,
            20.0,
            "px",
            show_details,
        ),
        make_slider_int(
            "tab_gap",
            "Gap",
            "Space between indicator and window",
            tab.gap,
            0.0,
            20.0,
            "px",
            show_details,
        ),
        make_slider_int(
            "tab_corner_radius",
            "Corner radius",
            "Rounded corners for the indicator",
            tab.corner_radius,
            0.0,
            20.0,
            "px",
            show_details,
        ),
        make_slider_int(
            "tab_gaps_between",
            "Gaps between tabs",
            "Space between individual tab indicators",
            tab.gaps_between_tabs,
            0.0,
            20.0,
            "px",
            show_details,
        ),
        make_slider_float(
            "tab_length_proportion",
            "Length proportion",
            "How much of the available length to use",
            tab.length_proportion,
            0.1,
            1.0,
            "%",
            show_details,
        ),
        make_color(
            "tab_active_color",
            "Active color",
            "Color for the active tab indicator",
            &tab.active.to_hex(),
            show_details,
        ),
        make_color(
            "tab_inactive_color",
            "Inactive color",
            "Color for inactive tab indicators",
            &tab.inactive.to_hex(),
            show_details,
        ),
        make_color(
            "tab_urgent_color",
            "Urgent color",
            "Color for tab indicator when window is urgent",
            &tab.urgent.to_hex(),
            show_details,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

/// Populate insert hint settings model
pub fn populate_insert_hint_settings(extras: &LayoutExtrasSettings) -> ModelRc<LayoutSettingModel> {
    let hint = &extras.insert_hint;
    let show_details = hint.enabled;

    let settings = vec![
        make_toggle(
            "hint_enabled",
            "Enable insert hint",
            "Show visual feedback when positioning windows",
            hint.enabled,
            true,
        ),
        make_color(
            "hint_color",
            "Hint color",
            "Color for the insert hint overlay",
            &hint.color.to_hex(),
            show_details,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

/// Sync all UI models from settings
fn sync_all_models(ui: &MainWindow, extras: &LayoutExtrasSettings) {
    ui.set_layout_extras_shadow_settings(populate_shadow_settings(extras));
    ui.set_layout_extras_tab_indicator_settings(populate_tab_indicator_settings(extras));
    ui.set_layout_extras_insert_hint_settings(populate_insert_hint_settings(extras));
}

// ============================================================================
// CALLBACK SETUP
// ============================================================================

/// Set up dynamic layout extras callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Generic toggle callback
    {
        let settings = Arc::clone(&settings);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_layout_extras_setting_toggle_changed(move |id, value| {
            let id_str = id.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    let extras = &mut s.layout_extras;
                    let mut needs_model_refresh = false;

                    match id_str.as_str() {
                        // Shadow settings
                        "shadow_enabled" => {
                            extras.shadow.enabled = value;
                            needs_model_refresh = true;
                        }
                        "shadow_draw_behind" => {
                            extras.shadow.draw_behind_window = value;
                        }

                        // Tab indicator settings
                        "tab_enabled" => {
                            extras.tab_indicator.enabled = value;
                            needs_model_refresh = true;
                        }
                        "tab_hide_single" => {
                            extras.tab_indicator.hide_when_single_tab = value;
                        }
                        "tab_within_column" => {
                            extras.tab_indicator.place_within_column = value;
                        }

                        // Insert hint settings
                        "hint_enabled" => {
                            extras.insert_hint.enabled = value;
                            needs_model_refresh = true;
                        }

                        _ => {
                            debug!("Unknown layout extras toggle setting: {}", id_str);
                            return;
                        }
                    }

                    // Refresh models if visibility changed
                    if needs_model_refresh {
                        if let Some(ui) = ui_weak.upgrade() {
                            sync_all_models(&ui, extras);
                        }
                    }

                    debug!("Layout extras toggle {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::LayoutExtras);
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
        ui.on_layout_extras_setting_slider_int_changed(move |id, value| {
            let id_str = id.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    let extras = &mut s.layout_extras;

                    match id_str.as_str() {
                        // Shadow settings
                        "shadow_softness" => {
                            extras.shadow.softness = value.clamp(0, 100);
                        }
                        "shadow_spread" => {
                            extras.shadow.spread = value.clamp(0, 50);
                        }
                        "shadow_offset_x" => {
                            extras.shadow.offset_x = value.clamp(-50, 50);
                        }
                        "shadow_offset_y" => {
                            extras.shadow.offset_y = value.clamp(-50, 50);
                        }

                        // Tab indicator settings
                        "tab_width" => {
                            extras.tab_indicator.width = value.clamp(1, 20);
                        }
                        "tab_gap" => {
                            extras.tab_indicator.gap = value.clamp(0, 20);
                        }
                        "tab_corner_radius" => {
                            extras.tab_indicator.corner_radius = value.clamp(0, 20);
                        }
                        "tab_gaps_between" => {
                            extras.tab_indicator.gaps_between_tabs = value.clamp(0, 20);
                        }

                        _ => {
                            debug!("Unknown layout extras slider int setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Layout extras slider int {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::LayoutExtras);
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
        ui.on_layout_extras_setting_slider_float_changed(move |id, value| {
            let id_str = id.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    let extras = &mut s.layout_extras;

                    match id_str.as_str() {
                        "tab_length_proportion" => {
                            extras.tab_indicator.length_proportion = value.clamp(0.1, 1.0);
                        }

                        _ => {
                            debug!("Unknown layout extras slider float setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Layout extras slider float {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::LayoutExtras);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Generic combo callback
    {
        let settings = Arc::clone(&settings);
        let save_manager = Rc::clone(&save_manager);
        ui.on_layout_extras_setting_combo_changed(move |id, index| {
            let id_str = id.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    let extras = &mut s.layout_extras;

                    match id_str.as_str() {
                        "tab_position" => {
                            extras.tab_indicator.position = match index {
                                0 => TabIndicatorPosition::Left,
                                1 => TabIndicatorPosition::Right,
                                2 => TabIndicatorPosition::Top,
                                3 => TabIndicatorPosition::Bottom,
                                _ => TabIndicatorPosition::Left,
                            };
                        }

                        _ => {
                            debug!("Unknown layout extras combo setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Layout extras combo {} = {}", id_str, index);
                    save_manager.mark_dirty(SettingsCategory::LayoutExtras);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Generic color callback (handles color hex input)
    {
        let settings = Arc::clone(&settings);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_layout_extras_setting_color_changed(move |id, hex_value| {
            let id_str = id.to_string();
            let hex_str = hex_value.to_string();

            // Parse the hex color
            let Some(color) = Color::from_hex(&hex_str) else {
                debug!("Invalid color hex value: {}", hex_str);
                return;
            };

            match settings.lock() {
                Ok(mut s) => {
                    let extras = &mut s.layout_extras;

                    match id_str.as_str() {
                        // Shadow colors
                        "shadow_color" => {
                            extras.shadow.color = color;
                        }
                        "shadow_inactive_color" => {
                            extras.shadow.inactive_color = color;
                        }

                        // Tab indicator colors (these are ColorOrGradient, so we set as solid color)
                        "tab_active_color" => {
                            extras.tab_indicator.active = ColorOrGradient::Color(color);
                        }
                        "tab_inactive_color" => {
                            extras.tab_indicator.inactive = ColorOrGradient::Color(color);
                        }
                        "tab_urgent_color" => {
                            extras.tab_indicator.urgent = ColorOrGradient::Color(color);
                        }

                        // Insert hint color
                        "hint_color" => {
                            extras.insert_hint.color = ColorOrGradient::Color(color);
                        }

                        _ => {
                            debug!("Unknown layout extras color setting: {}", id_str);
                            return;
                        }
                    }

                    // Update models to reflect color change
                    if let Some(ui) = ui_weak.upgrade() {
                        sync_all_models(&ui, extras);
                    }

                    debug!("Layout extras color {} = {}", id_str, hex_str);
                    save_manager.mark_dirty(SettingsCategory::LayoutExtras);
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

/// Public function to sync all layout extras models from settings
pub fn sync_layout_extras_models(ui: &MainWindow, extras: &LayoutExtrasSettings) {
    sync_all_models(ui, extras);
}
