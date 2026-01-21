//! Dynamic Window Rules Callbacks
//!
//! Model-driven approach replacing 50+ specific callbacks with generic ones.
//!
//! This module is split into:
//! - `mod.rs` - Setup function and callback handlers
//! - `populate.rs` - UI model population functions
//! - `helpers.rs` - Helper functions and conversion utilities

mod helpers;
mod populate;

use crate::config::models::{
    DefaultColumnDisplay, FloatingPosition, OpenBehavior, PositionRelativeTo, ShadowSettings,
    TabIndicatorPosition, TabIndicatorSettings, WindowRule, WindowRuleMatch,
};
use crate::config::{Settings, SettingsCategory};
use crate::constants::{MAX_MATCHES_PER_RULE, MAX_WINDOW_RULES};
use crate::types::Color;
use crate::MainWindow;
use log::{debug, warn};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::SaveManager;
use helpers::index_to_option_bool;
use populate::{
    build_rule_list_model, populate_opening_settings, populate_rule_models,
    populate_shadow_settings, populate_tab_settings, populate_visual_settings,
};

// Re-export public functions for sync module
pub use populate::{build_matches_list_model as build_matches_model, build_rule_list_model as build_rules_list_model};

/// Helper function for sync module compatibility
pub fn get_open_behavior_index(behavior: OpenBehavior) -> i32 {
    behavior.to_index()
}

/// Set up dynamic window rules callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    let selected_rule_idx = Rc::new(Cell::new(-1i32));
    let selected_match_idx = Rc::new(Cell::new(0i32));

    // Initialize rule list
    if let Ok(s) = settings.lock() {
        ui.set_window_rules_list_dynamic(build_rule_list_model(&s.window_rules.rules));
    }

    // Add rule
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_add_dynamic(move || {
            if let Ok(mut s) = settings.lock() {
                if s.window_rules.rules.len() >= MAX_WINDOW_RULES {
                    warn!("Maximum window rules limit reached");
                    return;
                }

                let new_id = s.window_rules.next_id;
                s.window_rules.next_id += 1;

                let rule = WindowRule {
                    id: new_id,
                    name: format!("Rule {}", new_id + 1),
                    ..Default::default()
                };
                s.window_rules.rules.push(rule);

                let new_idx = (s.window_rules.rules.len() - 1) as i32;
                selected_rule_idx.set(new_idx);
                selected_match_idx.set(0);

                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_window_rules_list_dynamic(build_rule_list_model(&s.window_rules.rules));
                    ui.set_selected_window_rule_index_dynamic(new_idx);

                    if let Some(rule) = s.window_rules.rules.get(new_idx as usize) {
                        populate_rule_models(&ui, rule, 0);
                    }
                }

                debug!("Added window rule {}", new_id);
                save_manager.mark_dirty(SettingsCategory::WindowRules);
                save_manager.request_save();
            }
        });
    }

    // Remove rule
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_remove_dynamic(move |index| {
            let idx = index as usize;
            if let Ok(mut s) = settings.lock() {
                if idx < s.window_rules.rules.len() {
                    s.window_rules.rules.remove(idx);

                    let new_sel = if s.window_rules.rules.is_empty() {
                        -1
                    } else if idx >= s.window_rules.rules.len() {
                        (s.window_rules.rules.len() - 1) as i32
                    } else {
                        idx as i32
                    };

                    selected_rule_idx.set(new_sel);
                    selected_match_idx.set(0);

                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_window_rules_list_dynamic(build_rule_list_model(
                            &s.window_rules.rules,
                        ));
                        ui.set_selected_window_rule_index_dynamic(new_sel);

                        if new_sel >= 0 {
                            if let Some(rule) = s.window_rules.rules.get(new_sel as usize) {
                                populate_rule_models(&ui, rule, 0);
                            }
                        }
                    }

                    save_manager.mark_dirty(SettingsCategory::WindowRules);
                    save_manager.request_save();
                }
            }
        });
    }

    // Select rule
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        ui.on_window_rule_select_dynamic(move |index| {
            selected_rule_idx.set(index);
            selected_match_idx.set(0);

            if let Ok(s) = settings.lock() {
                if let Some(ui) = ui_weak.upgrade() {
                    if let Some(rule) = s.window_rules.rules.get(index as usize) {
                        populate_rule_models(&ui, rule, 0);
                    }
                }
            }
        });
    }

    // Reorder rule
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_reorder_dynamic(move |from, to| {
            if let Ok(mut s) = settings.lock() {
                let from_idx = from as usize;
                let to_idx = to as usize;
                if from_idx < s.window_rules.rules.len() && to_idx <= s.window_rules.rules.len() {
                    let rule = s.window_rules.rules.remove(from_idx);
                    let insert_idx = if to_idx > from_idx {
                        to_idx - 1
                    } else {
                        to_idx
                    };
                    s.window_rules.rules.insert(insert_idx, rule);

                    selected_rule_idx.set(insert_idx as i32);

                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_window_rules_list_dynamic(build_rule_list_model(
                            &s.window_rules.rules,
                        ));
                        ui.set_selected_window_rule_index_dynamic(insert_idx as i32);
                    }

                    save_manager.mark_dirty(SettingsCategory::WindowRules);
                    save_manager.request_save();
                }
            }
        });
    }

    // Add match
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_add_match_dynamic(move || {
            let rule_idx = selected_rule_idx.get();
            if rule_idx < 0 {
                return;
            }

            if let Ok(mut s) = settings.lock() {
                if let Some(rule) = s.window_rules.rules.get_mut(rule_idx as usize) {
                    if rule.matches.len() >= MAX_MATCHES_PER_RULE {
                        warn!("Maximum matches per rule limit reached");
                        return;
                    }

                    rule.matches.push(WindowRuleMatch::default());
                    let new_match_idx = (rule.matches.len() - 1) as i32;
                    selected_match_idx.set(new_match_idx);

                    if let Some(ui) = ui_weak.upgrade() {
                        populate_rule_models(&ui, rule, new_match_idx);
                    }

                    save_manager.mark_dirty(SettingsCategory::WindowRules);
                    save_manager.request_save();
                }
            }
        });
    }

    // Remove match
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_remove_match_dynamic(move |match_index| {
            let rule_idx = selected_rule_idx.get();
            if rule_idx < 0 {
                return;
            }

            if let Ok(mut s) = settings.lock() {
                if let Some(rule) = s.window_rules.rules.get_mut(rule_idx as usize) {
                    let idx = match_index as usize;
                    if idx < rule.matches.len() && rule.matches.len() > 1 {
                        rule.matches.remove(idx);
                        let new_match_idx = if idx >= rule.matches.len() {
                            (rule.matches.len() - 1) as i32
                        } else {
                            idx as i32
                        };
                        selected_match_idx.set(new_match_idx);

                        if let Some(ui) = ui_weak.upgrade() {
                            populate_rule_models(&ui, rule, new_match_idx);
                        }

                        save_manager.mark_dirty(SettingsCategory::WindowRules);
                        save_manager.request_save();
                    }
                }
            }
        });
    }

    // Select match
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        ui.on_window_rule_select_match_dynamic(move |match_index| {
            selected_match_idx.set(match_index);

            let rule_idx = selected_rule_idx.get();
            if rule_idx < 0 {
                return;
            }

            if let Ok(s) = settings.lock() {
                if let Some(ui) = ui_weak.upgrade() {
                    if let Some(rule) = s.window_rules.rules.get(rule_idx as usize) {
                        populate::populate_match_settings(&ui, rule, match_index);
                    }
                }
            }
        });
    }

    // Generic toggle callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_setting_toggle_changed(move |id, value| {
            let rule_idx = selected_rule_idx.get();
            if rule_idx < 0 {
                return;
            }

            if let Ok(mut s) = settings.lock() {
                if let Some(rule) = s.window_rules.rules.get_mut(rule_idx as usize) {
                    let id_str = id.as_str();
                    let match_idx = selected_match_idx.get() as usize;

                    match id_str {
                        // Match criteria
                        "match_is_floating" => {
                            if let Some(m) = rule.matches.get_mut(match_idx) {
                                m.is_floating = if value { Some(true) } else { None };
                            }
                        }
                        // Visual properties
                        "has_opacity" => {
                            rule.opacity = if value { Some(1.0) } else { None };
                            if let Some(ui) = ui_weak.upgrade() {
                                populate_visual_settings(&ui, rule);
                            }
                        }
                        "block_screencast" => rule.block_out_from_screencast = value,
                        "has_corner_radius" => {
                            rule.corner_radius = if value { Some(12) } else { None };
                            if let Some(ui) = ui_weak.upgrade() {
                                populate_visual_settings(&ui, rule);
                            }
                        }
                        "clip_to_geometry" => rule.clip_to_geometry = Some(value),
                        "baba_is_float" => rule.baba_is_float = Some(value),
                        "has_floating_position" => {
                            rule.default_floating_position = if value {
                                Some(FloatingPosition::default())
                            } else {
                                None
                            };
                            if let Some(ui) = ui_weak.upgrade() {
                                populate_opening_settings(&ui, rule);
                            }
                        }
                        // Shadow
                        "shadow_draw_behind" => {
                            if let Some(ref mut shadow) = rule.shadow {
                                shadow.draw_behind_window = value;
                            }
                        }
                        // Tab indicator
                        "tab_hide_when_single" => {
                            if let Some(ref mut ti) = rule.tab_indicator {
                                ti.hide_when_single_tab = value;
                            }
                        }
                        "tab_place_within_column" => {
                            if let Some(ref mut ti) = rule.tab_indicator {
                                ti.place_within_column = value;
                            }
                        }
                        _ => debug!("Unknown toggle ID: {}", id_str),
                    }

                    save_manager.mark_dirty(SettingsCategory::WindowRules);
                    save_manager.request_save();
                }
            }
        });
    }

    // Generic slider int callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_setting_slider_int_changed(move |id, value| {
            let rule_idx = selected_rule_idx.get();
            if rule_idx < 0 {
                return;
            }

            if let Ok(mut s) = settings.lock() {
                if let Some(rule) = s.window_rules.rules.get_mut(rule_idx as usize) {
                    let id_str = id.as_str();

                    match id_str {
                        "corner_radius" => rule.corner_radius = Some(value),
                        "floating_x" => {
                            if let Some(ref mut pos) = rule.default_floating_position {
                                pos.x = value;
                            }
                        }
                        "floating_y" => {
                            if let Some(ref mut pos) = rule.default_floating_position {
                                pos.y = value;
                            }
                        }
                        // Shadow
                        "shadow_softness" => {
                            if let Some(ref mut shadow) = rule.shadow {
                                shadow.softness = value;
                            }
                        }
                        "shadow_spread" => {
                            if let Some(ref mut shadow) = rule.shadow {
                                shadow.spread = value;
                            }
                        }
                        "shadow_offset_x" => {
                            if let Some(ref mut shadow) = rule.shadow {
                                shadow.offset_x = value;
                            }
                        }
                        "shadow_offset_y" => {
                            if let Some(ref mut shadow) = rule.shadow {
                                shadow.offset_y = value;
                            }
                        }
                        // Tab indicator
                        "tab_gap" => {
                            if let Some(ref mut ti) = rule.tab_indicator {
                                ti.gap = value;
                            }
                        }
                        "tab_width" => {
                            if let Some(ref mut ti) = rule.tab_indicator {
                                ti.width = value;
                            }
                        }
                        "tab_gaps_between" => {
                            if let Some(ref mut ti) = rule.tab_indicator {
                                ti.gaps_between_tabs = value;
                            }
                        }
                        "tab_corner_radius" => {
                            if let Some(ref mut ti) = rule.tab_indicator {
                                ti.corner_radius = value;
                            }
                        }
                        _ => debug!("Unknown slider int ID: {}", id_str),
                    }

                    save_manager.mark_dirty(SettingsCategory::WindowRules);
                    save_manager.request_save();
                }
            }
        });
    }

    // Generic slider float callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_setting_slider_float_changed(move |id, value| {
            let rule_idx = selected_rule_idx.get();
            if rule_idx < 0 {
                return;
            }

            if let Ok(mut s) = settings.lock() {
                if let Some(rule) = s.window_rules.rules.get_mut(rule_idx as usize) {
                    let id_str = id.as_str();

                    match id_str {
                        "opacity" => rule.opacity = Some(value),
                        "tab_length" => {
                            if let Some(ref mut ti) = rule.tab_indicator {
                                ti.length_proportion = value;
                            }
                        }
                        _ => debug!("Unknown slider float ID: {}", id_str),
                    }

                    save_manager.mark_dirty(SettingsCategory::WindowRules);
                    save_manager.request_save();
                }
            }
        });
    }

    // Generic combo callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_setting_combo_changed(move |id, index| {
            let rule_idx = selected_rule_idx.get();
            if rule_idx < 0 {
                return;
            }

            if let Ok(mut s) = settings.lock() {
                if let Some(rule) = s.window_rules.rules.get_mut(rule_idx as usize) {
                    let id_str = id.as_str();
                    let match_idx = selected_match_idx.get() as usize;

                    match id_str {
                        // Match criteria (Any=0, Yes=1, No=2)
                        "match_is_floating" => {
                            if let Some(m) = rule.matches.get_mut(match_idx) {
                                m.is_floating = index_to_option_bool(index);
                            }
                        }
                        "match_is_active" => {
                            if let Some(m) = rule.matches.get_mut(match_idx) {
                                m.is_active = index_to_option_bool(index);
                            }
                        }
                        "match_is_focused" => {
                            if let Some(m) = rule.matches.get_mut(match_idx) {
                                m.is_focused = index_to_option_bool(index);
                            }
                        }
                        "match_is_active_in_column" => {
                            if let Some(m) = rule.matches.get_mut(match_idx) {
                                m.is_active_in_column = index_to_option_bool(index);
                            }
                        }
                        "match_is_window_cast_target" => {
                            if let Some(m) = rule.matches.get_mut(match_idx) {
                                m.is_window_cast_target = index_to_option_bool(index);
                            }
                        }
                        "match_is_urgent" => {
                            if let Some(m) = rule.matches.get_mut(match_idx) {
                                m.is_urgent = index_to_option_bool(index);
                            }
                        }
                        "match_at_startup" => {
                            if let Some(m) = rule.matches.get_mut(match_idx) {
                                m.at_startup = index_to_option_bool(index);
                            }
                        }
                        // Opening behavior
                        "open_behavior" => {
                            rule.open_behavior = match index {
                                1 => OpenBehavior::Maximized,
                                2 => OpenBehavior::Fullscreen,
                                3 => OpenBehavior::Floating,
                                _ => OpenBehavior::Normal,
                            };
                            if let Some(ui) = ui_weak.upgrade() {
                                populate_opening_settings(&ui, rule);
                            }
                        }
                        "floating_relative_to" => {
                            if let Some(ref mut pos) = rule.default_floating_position {
                                pos.relative_to = match index {
                                    1 => PositionRelativeTo::TopRight,
                                    2 => PositionRelativeTo::BottomLeft,
                                    3 => PositionRelativeTo::BottomRight,
                                    4 => PositionRelativeTo::Top,
                                    5 => PositionRelativeTo::Bottom,
                                    6 => PositionRelativeTo::Left,
                                    7 => PositionRelativeTo::Right,
                                    8 => PositionRelativeTo::Center,
                                    _ => PositionRelativeTo::TopLeft,
                                };
                            }
                        }
                        // Visual
                        "vrr" => {
                            rule.variable_refresh_rate = match index {
                                1 => Some(true),
                                2 => Some(false),
                                _ => None,
                            };
                        }
                        "column_display" => {
                            rule.default_column_display = match index {
                                1 => Some(DefaultColumnDisplay::Tabbed),
                                _ => None,
                            };
                        }
                        "tiled_state" => {
                            rule.tiled_state = match index {
                                1 => Some(true),
                                2 => Some(false),
                                _ => None,
                            };
                        }
                        // Shadow mode
                        "shadow_mode" => {
                            rule.shadow = match index {
                                1 => Some(ShadowSettings {
                                    enabled: true,
                                    ..ShadowSettings::default()
                                }),
                                2 => Some(ShadowSettings {
                                    enabled: false,
                                    ..ShadowSettings::default()
                                }),
                                _ => None,
                            };
                            if let Some(ui) = ui_weak.upgrade() {
                                populate_shadow_settings(&ui, rule);
                            }
                        }
                        // Tab indicator mode
                        "tab_mode" => {
                            rule.tab_indicator = match index {
                                1 => Some(TabIndicatorSettings {
                                    enabled: true,
                                    ..TabIndicatorSettings::default()
                                }),
                                2 => Some(TabIndicatorSettings {
                                    enabled: false,
                                    ..TabIndicatorSettings::default()
                                }),
                                _ => None,
                            };
                            if let Some(ui) = ui_weak.upgrade() {
                                populate_tab_settings(&ui, rule);
                            }
                        }
                        "tab_position" => {
                            if let Some(ref mut ti) = rule.tab_indicator {
                                ti.position = match index {
                                    1 => TabIndicatorPosition::Right,
                                    2 => TabIndicatorPosition::Top,
                                    3 => TabIndicatorPosition::Bottom,
                                    _ => TabIndicatorPosition::Left,
                                };
                            }
                        }
                        _ => debug!("Unknown combo ID: {}", id_str),
                    }

                    save_manager.mark_dirty(SettingsCategory::WindowRules);
                    save_manager.request_save();
                }
            }
        });
    }

    // Generic text callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_setting_text_changed(move |id, value| {
            let rule_idx = selected_rule_idx.get();
            if rule_idx < 0 {
                return;
            }

            if let Ok(mut s) = settings.lock() {
                if let Some(rule) = s.window_rules.rules.get_mut(rule_idx as usize) {
                    let id_str = id.as_str();
                    let mut value_str = value.to_string();
                    let match_idx = selected_match_idx.get() as usize;

                    // Validate length based on field type
                    let max_len = match id_str {
                        // Pattern fields use shorter limit
                        "match_app_id" | "match_title" => crate::constants::MAX_PATTERN_LENGTH,
                        // Other text fields use general limit
                        _ => crate::constants::MAX_STRING_LENGTH,
                    };

                    if value_str.len() > max_len {
                        warn!("Text input '{}' exceeds maximum length, truncating", id_str);
                        value_str.truncate(max_len);
                    }

                    match id_str {
                        "rule_name" => {
                            rule.name = value_str;
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_window_rules_list_dynamic(build_rule_list_model(
                                    &s.window_rules.rules,
                                ));
                            }
                        }
                        "match_app_id" => {
                            if let Some(m) = rule.matches.get_mut(match_idx) {
                                m.app_id = if value_str.is_empty() {
                                    None
                                } else {
                                    Some(value_str)
                                };
                            }
                        }
                        "match_title" => {
                            if let Some(m) = rule.matches.get_mut(match_idx) {
                                m.title = if value_str.is_empty() {
                                    None
                                } else {
                                    Some(value_str)
                                };
                            }
                        }
                        "open_on_output" => {
                            rule.open_on_output = if value_str.is_empty() {
                                None
                            } else {
                                Some(value_str)
                            };
                        }
                        "open_on_workspace" => {
                            rule.open_on_workspace = if value_str.is_empty() {
                                None
                            } else {
                                Some(value_str)
                            };
                        }
                        // Colors
                        "shadow_color" => {
                            if let Some(ref mut shadow) = rule.shadow {
                                if let Some(c) = Color::from_hex(&value_str) {
                                    shadow.color = c;
                                }
                            }
                        }
                        "shadow_inactive_color" => {
                            if let Some(ref mut shadow) = rule.shadow {
                                if let Some(c) = Color::from_hex(&value_str) {
                                    shadow.inactive_color = c;
                                }
                            }
                        }
                        "tab_active_color" => {
                            if let Some(ref mut ti) = rule.tab_indicator {
                                if let Some(c) = Color::from_hex(&value_str) {
                                    ti.active = crate::types::ColorOrGradient::Color(c);
                                }
                            }
                        }
                        "tab_inactive_color" => {
                            if let Some(ref mut ti) = rule.tab_indicator {
                                if let Some(c) = Color::from_hex(&value_str) {
                                    ti.inactive = crate::types::ColorOrGradient::Color(c);
                                }
                            }
                        }
                        _ => debug!("Unknown text ID: {}", id_str),
                    }

                    save_manager.mark_dirty(SettingsCategory::WindowRules);
                    save_manager.request_save();
                }
            }
        });
    }

    // Refresh windows (for picker)
    {
        let ui_weak = ui.as_weak();
        ui.on_window_rule_refresh_windows_dynamic(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let windows: Vec<String> = crate::ipc::get_windows()
                    .ok()
                    .map(|w| w.into_iter().map(|w| w.app_id).collect())
                    .unwrap_or_default();
                let model: Vec<SharedString> =
                    windows.into_iter().map(SharedString::from).collect();
                ui.set_window_rules_running_windows_dynamic(ModelRc::new(VecModel::from(model)));
            }
        });
    }

    // Select running window
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_select_running_window_dynamic(move |window_index| {
            let rule_idx = selected_rule_idx.get();
            if rule_idx < 0 {
                return;
            }

            if let Some(ui) = ui_weak.upgrade() {
                let windows: Vec<String> = crate::ipc::get_windows()
                    .ok()
                    .map(|w| w.into_iter().map(|w| w.app_id).collect())
                    .unwrap_or_default();
                if let Some(app_id) = windows.get(window_index as usize) {
                    if let Ok(mut s) = settings.lock() {
                        if let Some(rule) = s.window_rules.rules.get_mut(rule_idx as usize) {
                            let match_idx = selected_match_idx.get() as usize;
                            if let Some(m) = rule.matches.get_mut(match_idx) {
                                m.app_id = Some(app_id.clone());
                                populate::populate_match_settings(&ui, rule, match_idx as i32);

                                save_manager.mark_dirty(SettingsCategory::WindowRules);
                                save_manager.request_save();
                            }
                        }
                    }
                }
            }
        });
    }
}
