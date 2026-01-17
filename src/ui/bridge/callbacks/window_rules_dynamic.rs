//! Dynamic Window Rules Callbacks
//!
//! Model-driven approach replacing 50+ specific callbacks with generic ones.

use crate::config::models::{
    DefaultColumnDisplay, FloatingPosition, OpenBehavior, PositionRelativeTo, ShadowSettings,
    TabIndicatorPosition, TabIndicatorSettings, WindowRule, WindowRuleMatch,
};
use crate::config::{Settings, SettingsCategory};
use crate::constants::{MAX_MATCHES_PER_RULE, MAX_WINDOW_RULES};
use crate::types::Color;
use crate::MainWindow;
use crate::WindowRuleSettingModel;
use log::{debug, warn};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::SaveManager;

/// Build rule list model for UI display
fn build_rule_list_model(rules: &[WindowRule]) -> ModelRc<SharedString> {
    let names: Vec<SharedString> = rules.iter().map(|r| r.name.as_str().into()).collect();
    ModelRc::new(VecModel::from(names))
}

/// Build matches list model for UI display
fn build_matches_list_model(matches: &[WindowRuleMatch]) -> ModelRc<SharedString> {
    let labels: Vec<SharedString> = matches
        .iter()
        .enumerate()
        .map(|(i, _)| format!("Match {}", i + 1).into())
        .collect();
    ModelRc::new(VecModel::from(labels))
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
                        populate_match_settings(&ui, rule, match_index);
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
                    let value_str = value.to_string();
                    let match_idx = selected_match_idx.get() as usize;

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
                                populate_match_settings(&ui, rule, match_idx as i32);

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

/// Convert combo index to Option<bool>: 0=None, 1=Some(true), 2=Some(false)
fn index_to_option_bool(index: i32) -> Option<bool> {
    match index {
        1 => Some(true),
        2 => Some(false),
        _ => None,
    }
}

/// Convert Option<bool> to combo index
fn option_bool_to_index(opt: Option<bool>) -> i32 {
    match opt {
        Some(true) => 1,
        Some(false) => 2,
        None => 0,
    }
}

/// Populate all models for a rule
fn populate_rule_models(ui: &MainWindow, rule: &WindowRule, match_idx: i32) {
    // Matches list
    ui.set_window_rules_matches_list_dynamic(build_matches_list_model(&rule.matches));
    ui.set_selected_match_index_dynamic(match_idx);

    // Rule settings
    populate_rule_settings(ui, rule);
    populate_match_settings(ui, rule, match_idx);
    populate_opening_settings(ui, rule);
    populate_visual_settings(ui, rule);
    populate_shadow_settings(ui, rule);
    populate_tab_settings(ui, rule);
}

fn populate_rule_settings(ui: &MainWindow, rule: &WindowRule) {
    let model = vec![WindowRuleSettingModel {
        id: "rule_name".into(),
        label: "Rule name".into(),
        description: "Display name for this rule".into(),
        setting_type: 3, // text
        text_value: rule.name.as_str().into(),
        placeholder: "Rule name".into(),
        visible: true,
        ..Default::default()
    }];
    ui.set_window_rules_rule_settings(ModelRc::new(VecModel::from(model)));
}

fn populate_match_settings(ui: &MainWindow, rule: &WindowRule, match_idx: i32) {
    let m = rule
        .matches
        .get(match_idx as usize)
        .cloned()
        .unwrap_or_default();

    let model = vec![
        WindowRuleSettingModel {
            id: "match_app_id".into(),
            label: "App ID".into(),
            description: "Application identifier (regex)".into(),
            setting_type: 3,
            text_value: m.app_id.unwrap_or_default().into(),
            placeholder: "e.g., firefox".into(),
            visible: true,
            ..Default::default()
        },
        WindowRuleSettingModel {
            id: "match_title".into(),
            label: "Window title".into(),
            description: "Window title pattern (regex)".into(),
            setting_type: 3,
            text_value: m.title.unwrap_or_default().into(),
            placeholder: "e.g., .*YouTube.*".into(),
            visible: true,
            ..Default::default()
        },
        make_combo_setting(
            "match_is_floating",
            "Is floating",
            "Match floating windows",
            option_bool_to_index(m.is_floating),
            &["Any", "Yes", "No"],
        ),
        make_combo_setting(
            "match_is_active",
            "Is active",
            "Match active windows",
            option_bool_to_index(m.is_active),
            &["Any", "Yes", "No"],
        ),
        make_combo_setting(
            "match_is_focused",
            "Is focused",
            "Match focused windows",
            option_bool_to_index(m.is_focused),
            &["Any", "Yes", "No"],
        ),
        make_combo_setting(
            "match_is_active_in_column",
            "Is active in column",
            "Match windows active in column",
            option_bool_to_index(m.is_active_in_column),
            &["Any", "Yes", "No"],
        ),
        make_combo_setting(
            "match_is_window_cast_target",
            "Is window cast target",
            "Match screen-casted windows",
            option_bool_to_index(m.is_window_cast_target),
            &["Any", "Yes", "No"],
        ),
        make_combo_setting(
            "match_is_urgent",
            "Is urgent",
            "Match urgent windows",
            option_bool_to_index(m.is_urgent),
            &["Any", "Yes", "No"],
        ),
        make_combo_setting(
            "match_at_startup",
            "At startup",
            "Match only at window creation",
            option_bool_to_index(m.at_startup),
            &["Any", "Yes", "No"],
        ),
    ];
    ui.set_window_rules_match_settings(ModelRc::new(VecModel::from(model)));
}

fn populate_opening_settings(ui: &MainWindow, rule: &WindowRule) {
    let is_floating = matches!(rule.open_behavior, OpenBehavior::Floating);
    let has_position = rule.default_floating_position.is_some();
    let pos = rule.default_floating_position.clone().unwrap_or_default();

    let model = vec![
        make_combo_setting(
            "open_behavior",
            "Open as",
            "How window opens",
            rule.open_behavior.to_index(),
            &["Normal", "Maximized", "Fullscreen", "Floating"],
        ),
        WindowRuleSettingModel {
            id: "open_on_output".into(),
            label: "Open on output".into(),
            description: "Force open on specific monitor".into(),
            setting_type: 3,
            text_value: rule.open_on_output.clone().unwrap_or_default().into(),
            placeholder: "e.g., eDP-1".into(),
            visible: true,
            ..Default::default()
        },
        WindowRuleSettingModel {
            id: "open_on_workspace".into(),
            label: "Open on workspace".into(),
            description: "Force open on specific workspace".into(),
            setting_type: 3,
            text_value: rule.open_on_workspace.clone().unwrap_or_default().into(),
            placeholder: "e.g., browser".into(),
            visible: true,
            ..Default::default()
        },
        WindowRuleSettingModel {
            id: "has_floating_position".into(),
            label: "Set position".into(),
            description: "Specify floating window position".into(),
            setting_type: 0, // toggle
            bool_value: has_position,
            visible: is_floating,
            ..Default::default()
        },
        make_combo_setting_visible(
            "floating_relative_to",
            "Relative to",
            "Position reference",
            pos.relative_to.to_index(),
            &[
                "Top-Left",
                "Top-Right",
                "Bottom-Left",
                "Bottom-Right",
                "Top",
                "Bottom",
                "Left",
                "Right",
                "Center",
            ],
            is_floating && has_position,
        ),
        make_slider_int_visible(
            "floating_x",
            "X offset",
            "Horizontal offset",
            pos.x,
            -500,
            500,
            "px",
            is_floating && has_position,
        ),
        make_slider_int_visible(
            "floating_y",
            "Y offset",
            "Vertical offset",
            pos.y,
            -500,
            500,
            "px",
            is_floating && has_position,
        ),
    ];
    ui.set_window_rules_opening_settings(ModelRc::new(VecModel::from(model)));
}

fn populate_visual_settings(ui: &MainWindow, rule: &WindowRule) {
    let has_opacity = rule.opacity.is_some();
    let has_radius = rule.corner_radius.is_some();

    let model = vec![
        WindowRuleSettingModel {
            id: "has_opacity".into(),
            label: "Custom opacity".into(),
            description: "Set window transparency".into(),
            setting_type: 0,
            bool_value: has_opacity,
            visible: true,
            ..Default::default()
        },
        make_slider_float_visible(
            "opacity",
            "Opacity",
            "Transparency level",
            rule.opacity.unwrap_or(1.0),
            0.0,
            1.0,
            "%",
            has_opacity,
        ),
        WindowRuleSettingModel {
            id: "has_corner_radius".into(),
            label: "Geometry corner radius".into(),
            description: "Round corners".into(),
            setting_type: 0,
            bool_value: has_radius,
            visible: true,
            ..Default::default()
        },
        make_slider_int_visible(
            "corner_radius",
            "Radius",
            "Corner radius",
            rule.corner_radius.unwrap_or(12),
            0,
            32,
            "px",
            has_radius,
        ),
        WindowRuleSettingModel {
            id: "clip_to_geometry".into(),
            label: "Clip to geometry".into(),
            description: "Clip window to visual geometry".into(),
            setting_type: 0,
            bool_value: rule.clip_to_geometry.unwrap_or(false),
            visible: true,
            ..Default::default()
        },
        WindowRuleSettingModel {
            id: "block_screencast".into(),
            label: "Block from screencast".into(),
            description: "Hide in screen recordings".into(),
            setting_type: 0,
            bool_value: rule.block_out_from_screencast,
            visible: true,
            ..Default::default()
        },
        WindowRuleSettingModel {
            id: "baba_is_float".into(),
            label: "Animated floating".into(),
            description: "Enable baba-is-float effect".into(),
            setting_type: 0,
            bool_value: rule.baba_is_float.unwrap_or(false),
            visible: true,
            ..Default::default()
        },
        make_combo_setting(
            "vrr",
            "Variable Refresh Rate",
            "Enable VRR for this window",
            match rule.variable_refresh_rate {
                Some(true) => 1,
                Some(false) => 2,
                None => 0,
            },
            &["Default", "On", "Off"],
        ),
        make_combo_setting(
            "column_display",
            "Column display",
            "Window column display mode",
            match rule.default_column_display {
                Some(DefaultColumnDisplay::Tabbed) => 1,
                _ => 0,
            },
            &["Default", "Tabbed"],
        ),
        make_combo_setting(
            "tiled_state",
            "Tiled state",
            "Mark as tiled or floating",
            match rule.tiled_state {
                Some(true) => 1,
                Some(false) => 2,
                None => 0,
            },
            &["Default", "Tiled", "Floating"],
        ),
    ];
    ui.set_window_rules_visual_settings(ModelRc::new(VecModel::from(model)));
}

fn populate_shadow_settings(ui: &MainWindow, rule: &WindowRule) {
    let mode = match &rule.shadow {
        Some(s) if s.enabled => 1,
        Some(_) => 2,
        None => 0,
    };
    let shadow = rule.shadow.clone().unwrap_or_default();
    let show_details = mode == 1;

    let model = vec![
        make_combo_setting(
            "shadow_mode",
            "Shadow",
            "Override window shadow",
            mode,
            &["Default", "Custom", "Off"],
        ),
        make_slider_int_visible(
            "shadow_softness",
            "Softness",
            "Blur radius",
            shadow.softness,
            0,
            100,
            "px",
            show_details,
        ),
        make_slider_int_visible(
            "shadow_spread",
            "Spread",
            "Spread distance",
            shadow.spread,
            -50,
            50,
            "px",
            show_details,
        ),
        make_slider_int_visible(
            "shadow_offset_x",
            "Offset X",
            "Horizontal offset",
            shadow.offset_x,
            -50,
            50,
            "px",
            show_details,
        ),
        make_slider_int_visible(
            "shadow_offset_y",
            "Offset Y",
            "Vertical offset",
            shadow.offset_y,
            -50,
            50,
            "px",
            show_details,
        ),
        make_color_setting_visible(
            "shadow_color",
            "Active color",
            "Active shadow color",
            &shadow.color.to_hex(),
            show_details,
        ),
        make_color_setting_visible(
            "shadow_inactive_color",
            "Inactive color",
            "Inactive shadow color",
            &shadow.inactive_color.to_hex(),
            show_details,
        ),
        WindowRuleSettingModel {
            id: "shadow_draw_behind".into(),
            label: "Draw behind window".into(),
            description: "Render behind opaque areas".into(),
            setting_type: 0,
            bool_value: shadow.draw_behind_window,
            visible: show_details,
            ..Default::default()
        },
    ];
    ui.set_window_rules_shadow_settings(ModelRc::new(VecModel::from(model)));
}

fn populate_tab_settings(ui: &MainWindow, rule: &WindowRule) {
    let mode = match &rule.tab_indicator {
        Some(ti) if ti.enabled => 1,
        Some(_) => 2,
        None => 0,
    };
    let ti = rule.tab_indicator.clone().unwrap_or_default();
    let show_details = mode == 1;

    let model = vec![
        make_combo_setting(
            "tab_mode",
            "Tab indicator",
            "Override tab indicator",
            mode,
            &["Default", "Custom", "Off"],
        ),
        WindowRuleSettingModel {
            id: "tab_hide_when_single".into(),
            label: "Hide when single tab".into(),
            description: "Hide with only one window".into(),
            setting_type: 0,
            bool_value: ti.hide_when_single_tab,
            visible: show_details,
            ..Default::default()
        },
        WindowRuleSettingModel {
            id: "tab_place_within_column".into(),
            label: "Place within column".into(),
            description: "Shrink column to fit".into(),
            setting_type: 0,
            bool_value: ti.place_within_column,
            visible: show_details,
            ..Default::default()
        },
        make_combo_setting_visible(
            "tab_position",
            "Position",
            "Tab indicator placement",
            match ti.position {
                TabIndicatorPosition::Left => 0,
                TabIndicatorPosition::Right => 1,
                TabIndicatorPosition::Top => 2,
                TabIndicatorPosition::Bottom => 3,
            },
            &["Left", "Right", "Top", "Bottom"],
            show_details,
        ),
        make_slider_int_visible(
            "tab_gap",
            "Gap",
            "Space from window",
            ti.gap,
            0,
            32,
            "px",
            show_details,
        ),
        make_slider_int_visible(
            "tab_width",
            "Width",
            "Indicator thickness",
            ti.width,
            1,
            32,
            "px",
            show_details,
        ),
        make_slider_float_visible(
            "tab_length",
            "Length",
            "Proportion of window",
            ti.length_proportion,
            0.0,
            1.0,
            "%",
            show_details,
        ),
        make_slider_int_visible(
            "tab_gaps_between",
            "Gaps between tabs",
            "Space between segments",
            ti.gaps_between_tabs,
            0,
            16,
            "px",
            show_details,
        ),
        make_slider_int_visible(
            "tab_corner_radius",
            "Corner radius",
            "Indicator rounding",
            ti.corner_radius,
            0,
            32,
            "px",
            show_details,
        ),
        make_color_setting_visible(
            "tab_active_color",
            "Active color",
            "Active tab color",
            &ti.active.to_hex(),
            show_details,
        ),
        make_color_setting_visible(
            "tab_inactive_color",
            "Inactive color",
            "Inactive tab color",
            &ti.inactive.to_hex(),
            show_details,
        ),
    ];
    ui.set_window_rules_tab_settings(ModelRc::new(VecModel::from(model)));
}

// Helper functions to create setting models
fn make_combo_setting(
    id: &str,
    label: &str,
    desc: &str,
    index: i32,
    options: &[&str],
) -> WindowRuleSettingModel {
    make_combo_setting_visible(id, label, desc, index, options, true)
}

fn make_combo_setting_visible(
    id: &str,
    label: &str,
    desc: &str,
    index: i32,
    options: &[&str],
    visible: bool,
) -> WindowRuleSettingModel {
    WindowRuleSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 2,
        combo_index: index,
        combo_options: ModelRc::new(VecModel::from(
            options
                .iter()
                .map(|s| SharedString::from(*s))
                .collect::<Vec<_>>(),
        )),
        visible,
        ..Default::default()
    }
}

fn make_slider_int_visible(
    id: &str,
    label: &str,
    desc: &str,
    value: i32,
    min: i32,
    max: i32,
    suffix: &str,
    visible: bool,
) -> WindowRuleSettingModel {
    WindowRuleSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 1,
        int_value: value,
        min_value: min as f32,
        max_value: max as f32,
        suffix: suffix.into(),
        use_float: false,
        visible,
        ..Default::default()
    }
}

fn make_slider_float_visible(
    id: &str,
    label: &str,
    desc: &str,
    value: f32,
    min: f32,
    max: f32,
    suffix: &str,
    visible: bool,
) -> WindowRuleSettingModel {
    WindowRuleSettingModel {
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

fn make_color_setting_visible(
    id: &str,
    label: &str,
    desc: &str,
    value: &str,
    visible: bool,
) -> WindowRuleSettingModel {
    WindowRuleSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 4,
        text_value: value.into(),
        visible,
        ..Default::default()
    }
}

// ============================================================================
// Public helper functions for sync module
// ============================================================================

/// Public function to build rule list model for sync
pub fn build_rules_list_model(rules: &[WindowRule]) -> ModelRc<SharedString> {
    build_rule_list_model(rules)
}

/// Public function to build matches list model for sync
pub fn build_matches_model(matches: &[WindowRuleMatch]) -> ModelRc<SharedString> {
    build_matches_list_model(matches)
}

/// Helper function for sync module compatibility
pub fn get_open_behavior_index(behavior: OpenBehavior) -> i32 {
    behavior.to_index()
}
