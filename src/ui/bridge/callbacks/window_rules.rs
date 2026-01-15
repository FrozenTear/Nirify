//! Window rules UI callbacks
//!
//! Handles window rule configuration including add, remove, select, and property changes.

use crate::config::models::{
    FloatingPosition, OpenBehavior, PositionRelativeTo, ShadowSettings, TabIndicatorPosition,
    TabIndicatorSettings, WindowRule, WindowRuleMatch,
};
use crate::types::{Color, ColorOrGradient};
use crate::config::{Settings, SettingsCategory};
use crate::constants::{MAX_MATCHES_PER_RULE, MAX_WINDOW_RULES};
use crate::ipc;
use crate::MainWindow;
use log::{debug, error, warn};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::SaveManager;
use super::rules_common::{
    self, calculate_new_selection_after_remove, get_selected_index, is_valid_match_index,
    is_valid_rule_index, parse_optional_string, reset_match_index, set_selected_index, Named,
};

impl Named for WindowRule {
    fn name(&self) -> &str {
        &self.name
    }
}

/// Build rule list model for UI display
fn build_rule_list_model(rules: &[WindowRule]) -> ModelRc<SharedString> {
    rules_common::build_names_list(rules)
}

/// Build matches list model for UI display
fn build_matches_list_model(matches: &[WindowRuleMatch]) -> ModelRc<SharedString> {
    let mut labels = Vec::with_capacity(matches.len());
    for (i, m) in matches.iter().enumerate() {
        let app = m.app_id.as_deref().unwrap_or("*");
        let title = m.title.as_deref().unwrap_or("*");
        labels.push(format!("{}. app={} title={}", i + 1, app, title).into());
    }
    ModelRc::new(VecModel::from(labels))
}

/// Set up window rules callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Shared state for tracking selected indices (Rc<Cell> since Slint callbacks are single-threaded)
    let selected_rule_idx = Rc::new(Cell::new(-1i32));
    let selected_match_idx = Rc::new(Cell::new(0i32));

    // Add rule callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_add(move || {
            match settings.lock() {
                Ok(mut s) => {
                    // Check limit before adding
                    if s.window_rules.rules.len() >= MAX_WINDOW_RULES {
                        warn!("Maximum window rules limit ({}) reached", MAX_WINDOW_RULES);
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
                    set_selected_index(&selected_rule_idx, new_idx);
                    reset_match_index(&selected_match_idx);

                    // Update UI with new rule list
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_window_rules_list(build_rule_list_model(&s.window_rules.rules));
                        ui.set_selected_window_rule_index(new_idx);

                        // Sync current rule properties
                        if let Some(rule) = s.window_rules.rules.get(new_idx as usize) {
                            sync_current_rule(&ui, rule, 0);
                        }
                    }

                    debug!("Added new window rule with id {}", new_id);
                    save_manager.mark_dirty(SettingsCategory::WindowRules);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Remove rule callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_remove(move |index| {
            let idx = index as usize;
            match settings.lock() {
                Ok(mut s) => {
                    if idx < s.window_rules.rules.len() {
                        let name = s.window_rules.rules[idx].name.clone();
                        s.window_rules.rules.remove(idx);

                        // Update selected index
                        let new_sel =
                            calculate_new_selection_after_remove(idx, s.window_rules.rules.len());
                        set_selected_index(&selected_rule_idx, new_sel);
                        reset_match_index(&selected_match_idx);

                        // Update UI
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_window_rules_list(build_rule_list_model(&s.window_rules.rules));
                            ui.set_selected_window_rule_index(new_sel);

                            // Sync current rule properties if there's a selection
                            if new_sel >= 0 {
                                if let Some(rule) = s.window_rules.rules.get(new_sel as usize) {
                                    sync_current_rule(&ui, rule, 0);
                                }
                            }
                        }

                        debug!("Removed window rule at index {}: {}", idx, name);
                        save_manager.mark_dirty(SettingsCategory::WindowRules);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Select rule callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        ui.on_window_rule_select(move |index| {
            set_selected_index(&selected_rule_idx, index);
            reset_match_index(&selected_match_idx);

            // Sync current rule properties to UI
            if let Some(ui) = ui_weak.upgrade() {
                if let Ok(s) = settings.lock() {
                    if is_valid_rule_index(index, s.window_rules.rules.len()) {
                        if let Some(rule) = s.window_rules.rules.get(index as usize) {
                            sync_current_rule(&ui, rule, 0);
                        }
                    }
                }
            }

            debug!("Selected window rule at index {}", index);
        });
    }

    // Add match criteria callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_add_match(move || match settings.lock() {
            Ok(mut s) => {
                let rule_idx = get_selected_index(&selected_rule_idx);
                if is_valid_rule_index(rule_idx, s.window_rules.rules.len()) {
                    if let Some(rule) = s.window_rules.rules.get_mut(rule_idx as usize) {
                        // Check limit before adding
                        if rule.matches.len() >= MAX_MATCHES_PER_RULE {
                            warn!(
                                "Maximum matches per rule limit ({}) reached",
                                MAX_MATCHES_PER_RULE
                            );
                            return;
                        }

                        rule.matches.push(WindowRuleMatch::default());
                        let new_match_idx = (rule.matches.len() - 1) as i32;
                        set_selected_index(&selected_match_idx, new_match_idx);

                        if let Some(ui) = ui_weak.upgrade() {
                            sync_current_rule(&ui, rule, new_match_idx);
                        }

                        debug!("Added new match criteria to rule {}", rule_idx);
                        save_manager.mark_dirty(SettingsCategory::WindowRules);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Remove match criteria callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_remove_match(move |match_index| match settings.lock() {
            Ok(mut s) => {
                let rule_idx = get_selected_index(&selected_rule_idx);
                if is_valid_rule_index(rule_idx, s.window_rules.rules.len()) {
                    if let Some(rule) = s.window_rules.rules.get_mut(rule_idx as usize) {
                        let match_idx = match_index as usize;
                        if match_idx < rule.matches.len() && rule.matches.len() > 1 {
                            rule.matches.remove(match_idx);

                            let new_match_idx =
                                rules_common::calculate_new_match_selection_after_remove(
                                    match_idx,
                                    rule.matches.len(),
                                );
                            set_selected_index(&selected_match_idx, new_match_idx);

                            if let Some(ui) = ui_weak.upgrade() {
                                sync_current_rule(&ui, rule, new_match_idx);
                            }

                            debug!(
                                "Removed match criteria {} from rule {}",
                                match_idx, rule_idx
                            );
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Select match criteria callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        ui.on_window_rule_select_match(move |match_index| {
            set_selected_index(&selected_match_idx, match_index);

            if let Some(ui) = ui_weak.upgrade() {
                if let Ok(s) = settings.lock() {
                    let rule_idx = get_selected_index(&selected_rule_idx);
                    if is_valid_rule_index(rule_idx, s.window_rules.rules.len()) {
                        if let Some(rule) = s.window_rules.rules.get(rule_idx as usize) {
                            sync_current_match(&ui, rule, match_index);
                        }
                    }
                }
            }

            debug!("Selected match criteria at index {}", match_index);
        });
    }

    // Rule name changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_name_changed(move |name| {
            let name_str = name.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    let idx = get_selected_index(&selected_rule_idx);
                    if is_valid_rule_index(idx, s.window_rules.rules.len()) {
                        if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                            rule.name = name_str.clone();

                            // Update rule list display
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_window_rules_list(build_rule_list_model(
                                    &s.window_rules.rules,
                                ));
                            }

                            debug!("Rule {} name changed to: {}", idx, name_str);
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Match app-id changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_match_app_id_changed(move |app_id| {
            let app_id_str = app_id.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    let rule_idx = get_selected_index(&selected_rule_idx);
                    let match_idx = get_selected_index(&selected_match_idx);
                    if is_valid_rule_index(rule_idx, s.window_rules.rules.len()) {
                        if let Some(rule) = s.window_rules.rules.get_mut(rule_idx as usize) {
                            if is_valid_match_index(match_idx, rule.matches.len()) {
                                rule.matches[match_idx as usize].app_id =
                                    parse_optional_string(&app_id_str);

                                // Update matches list display
                                if let Some(ui) = ui_weak.upgrade() {
                                    ui.set_current_matches_list(build_matches_list_model(
                                        &rule.matches,
                                    ));
                                }

                                debug!(
                                    "Rule {} match {} app-id: {:?}",
                                    rule_idx, match_idx, app_id_str
                                );
                                save_manager.mark_dirty(SettingsCategory::WindowRules);
                                save_manager.request_save();
                            }
                        }
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Match title changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_match_title_changed(move |title| {
            let title_str = title.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    let rule_idx = get_selected_index(&selected_rule_idx);
                    let match_idx = get_selected_index(&selected_match_idx);
                    if is_valid_rule_index(rule_idx, s.window_rules.rules.len()) {
                        if let Some(rule) = s.window_rules.rules.get_mut(rule_idx as usize) {
                            if is_valid_match_index(match_idx, rule.matches.len()) {
                                rule.matches[match_idx as usize].title =
                                    parse_optional_string(&title_str);

                                // Update matches list display
                                if let Some(ui) = ui_weak.upgrade() {
                                    ui.set_current_matches_list(build_matches_list_model(
                                        &rule.matches,
                                    ));
                                }

                                debug!(
                                    "Rule {} match {} title: {:?}",
                                    rule_idx, match_idx, title_str
                                );
                                save_manager.mark_dirty(SettingsCategory::WindowRules);
                                save_manager.request_save();
                            }
                        }
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Advanced match criteria callbacks
    // Helper to convert UI index to Option<bool>: 0=Any (None), 1=Yes (Some(true)), 2=No (Some(false))
    fn index_to_option_bool(idx: i32) -> Option<bool> {
        match idx {
            1 => Some(true),
            2 => Some(false),
            _ => None,
        }
    }

    // Match is_floating changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_match_is_floating_changed(move |idx| match settings.lock() {
            Ok(mut s) => {
                let rule_idx = get_selected_index(&selected_rule_idx);
                let match_idx = get_selected_index(&selected_match_idx);
                if is_valid_rule_index(rule_idx, s.window_rules.rules.len()) {
                    if let Some(rule) = s.window_rules.rules.get_mut(rule_idx as usize) {
                        if is_valid_match_index(match_idx, rule.matches.len()) {
                            rule.matches[match_idx as usize].is_floating = index_to_option_bool(idx);
                            debug!("Rule {} match {} is_floating: {:?}", rule_idx, match_idx, index_to_option_bool(idx));
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Match is_active changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_match_is_active_changed(move |idx| match settings.lock() {
            Ok(mut s) => {
                let rule_idx = get_selected_index(&selected_rule_idx);
                let match_idx = get_selected_index(&selected_match_idx);
                if is_valid_rule_index(rule_idx, s.window_rules.rules.len()) {
                    if let Some(rule) = s.window_rules.rules.get_mut(rule_idx as usize) {
                        if is_valid_match_index(match_idx, rule.matches.len()) {
                            rule.matches[match_idx as usize].is_active = index_to_option_bool(idx);
                            debug!("Rule {} match {} is_active: {:?}", rule_idx, match_idx, index_to_option_bool(idx));
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Match is_focused changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_match_is_focused_changed(move |idx| match settings.lock() {
            Ok(mut s) => {
                let rule_idx = get_selected_index(&selected_rule_idx);
                let match_idx = get_selected_index(&selected_match_idx);
                if is_valid_rule_index(rule_idx, s.window_rules.rules.len()) {
                    if let Some(rule) = s.window_rules.rules.get_mut(rule_idx as usize) {
                        if is_valid_match_index(match_idx, rule.matches.len()) {
                            rule.matches[match_idx as usize].is_focused = index_to_option_bool(idx);
                            debug!("Rule {} match {} is_focused: {:?}", rule_idx, match_idx, index_to_option_bool(idx));
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Match is_active_in_column changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_match_is_active_in_column_changed(move |idx| match settings.lock() {
            Ok(mut s) => {
                let rule_idx = get_selected_index(&selected_rule_idx);
                let match_idx = get_selected_index(&selected_match_idx);
                if is_valid_rule_index(rule_idx, s.window_rules.rules.len()) {
                    if let Some(rule) = s.window_rules.rules.get_mut(rule_idx as usize) {
                        if is_valid_match_index(match_idx, rule.matches.len()) {
                            rule.matches[match_idx as usize].is_active_in_column = index_to_option_bool(idx);
                            debug!("Rule {} match {} is_active_in_column: {:?}", rule_idx, match_idx, index_to_option_bool(idx));
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Match is_window_cast_target changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_match_is_window_cast_target_changed(move |idx| match settings.lock() {
            Ok(mut s) => {
                let rule_idx = get_selected_index(&selected_rule_idx);
                let match_idx = get_selected_index(&selected_match_idx);
                if is_valid_rule_index(rule_idx, s.window_rules.rules.len()) {
                    if let Some(rule) = s.window_rules.rules.get_mut(rule_idx as usize) {
                        if is_valid_match_index(match_idx, rule.matches.len()) {
                            rule.matches[match_idx as usize].is_window_cast_target = index_to_option_bool(idx);
                            debug!("Rule {} match {} is_window_cast_target: {:?}", rule_idx, match_idx, index_to_option_bool(idx));
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Match is_urgent changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_match_is_urgent_changed(move |idx| match settings.lock() {
            Ok(mut s) => {
                let rule_idx = get_selected_index(&selected_rule_idx);
                let match_idx = get_selected_index(&selected_match_idx);
                if is_valid_rule_index(rule_idx, s.window_rules.rules.len()) {
                    if let Some(rule) = s.window_rules.rules.get_mut(rule_idx as usize) {
                        if is_valid_match_index(match_idx, rule.matches.len()) {
                            rule.matches[match_idx as usize].is_urgent = index_to_option_bool(idx);
                            debug!("Rule {} match {} is_urgent: {:?}", rule_idx, match_idx, index_to_option_bool(idx));
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Match at_startup changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_match_at_startup_changed(move |idx| match settings.lock() {
            Ok(mut s) => {
                let rule_idx = get_selected_index(&selected_rule_idx);
                let match_idx = get_selected_index(&selected_match_idx);
                if is_valid_rule_index(rule_idx, s.window_rules.rules.len()) {
                    if let Some(rule) = s.window_rules.rules.get_mut(rule_idx as usize) {
                        if is_valid_match_index(match_idx, rule.matches.len()) {
                            rule.matches[match_idx as usize].at_startup = index_to_option_bool(idx);
                            debug!("Rule {} match {} at_startup: {:?}", rule_idx, match_idx, index_to_option_bool(idx));
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Open behavior changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_open_behavior_changed(move |index| {
            let behavior = OpenBehavior::from_index(index);
            match settings.lock() {
                Ok(mut s) => {
                    let idx = get_selected_index(&selected_rule_idx);
                    if is_valid_rule_index(idx, s.window_rules.rules.len()) {
                        if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                            rule.open_behavior = behavior;
                            debug!("Rule {} open behavior: {:?}", idx, behavior);
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Opacity toggled callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_opacity_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                let idx = get_selected_index(&selected_rule_idx);
                if is_valid_rule_index(idx, s.window_rules.rules.len()) {
                    if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                        rule.opacity = if enabled { Some(1.0) } else { None };
                        debug!("Rule {} opacity enabled: {}", idx, enabled);
                        save_manager.mark_dirty(SettingsCategory::WindowRules);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Opacity changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_opacity_changed(move |opacity| {
            let clamped = opacity.clamp(0.0, 1.0);
            match settings.lock() {
                Ok(mut s) => {
                    let idx = get_selected_index(&selected_rule_idx);
                    if is_valid_rule_index(idx, s.window_rules.rules.len()) {
                        if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                            rule.opacity = Some(clamped);
                            debug!("Rule {} opacity: {}", idx, clamped);
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Block screencast toggled callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_block_screencast_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                        rule.block_out_from_screencast = enabled;
                        debug!("Rule {} block screencast: {}", idx, enabled);
                        save_manager.mark_dirty(SettingsCategory::WindowRules);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Corner radius toggled callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_corner_radius_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                        if enabled {
                            rule.corner_radius = Some(12);
                        } else {
                            rule.corner_radius = None;
                        }
                        debug!("Rule {} corner radius enabled: {}", idx, enabled);
                        save_manager.mark_dirty(SettingsCategory::WindowRules);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Corner radius changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_corner_radius_changed(move |radius| {
            let clamped = radius.clamp(0, 32);
            match settings.lock() {
                Ok(mut s) => {
                    let idx = selected_rule_idx.get();
                    if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                        if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                            rule.corner_radius = Some(clamped);
                            debug!("Rule {} corner radius: {}", idx, clamped);
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Clip to geometry toggled callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_clip_to_geometry_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                        rule.clip_to_geometry = if enabled { Some(true) } else { None };
                        debug!("Rule {} clip to geometry: {}", idx, enabled);
                        save_manager.mark_dirty(SettingsCategory::WindowRules);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // VRR changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_vrr_changed(move |idx| match settings.lock() {
            Ok(mut s) => {
                let rule_idx = selected_rule_idx.get();
                if rule_idx >= 0 && (rule_idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(rule_idx as usize) {
                        // 0=Default (None), 1=On, 2=Off
                        rule.variable_refresh_rate = match idx {
                            1 => Some(true),
                            2 => Some(false),
                            _ => None,
                        };
                        debug!("Rule {} VRR: {:?}", rule_idx, rule.variable_refresh_rate);
                        save_manager.mark_dirty(SettingsCategory::WindowRules);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Column display changed callback
    {
        use crate::config::models::DefaultColumnDisplay;
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_column_display_changed(move |idx| match settings.lock() {
            Ok(mut s) => {
                let rule_idx = selected_rule_idx.get();
                if rule_idx >= 0 && (rule_idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(rule_idx as usize) {
                        // 0=Default (None), 1=Tabbed
                        rule.default_column_display = match idx {
                            1 => Some(DefaultColumnDisplay::Tabbed),
                            _ => None,
                        };
                        debug!("Rule {} column display: {:?}", rule_idx, rule.default_column_display);
                        save_manager.mark_dirty(SettingsCategory::WindowRules);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Tiled state changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_tiled_state_changed(move |idx| match settings.lock() {
            Ok(mut s) => {
                let rule_idx = selected_rule_idx.get();
                if rule_idx >= 0 && (rule_idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(rule_idx as usize) {
                        // 0=Default (None), 1=Tiled, 2=Floating
                        rule.tiled_state = match idx {
                            1 => Some(true),
                            2 => Some(false),
                            _ => None,
                        };
                        debug!("Rule {} tiled state: {:?}", rule_idx, rule.tiled_state);
                        save_manager.mark_dirty(SettingsCategory::WindowRules);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Baba is float toggled callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_baba_is_float_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                let rule_idx = selected_rule_idx.get();
                if rule_idx >= 0 && (rule_idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(rule_idx as usize) {
                        rule.baba_is_float = if enabled { Some(true) } else { None };
                        debug!("Rule {} baba is float: {}", rule_idx, enabled);
                        save_manager.mark_dirty(SettingsCategory::WindowRules);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Open on output changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_open_on_output_changed(move |output| {
            let output_str = output.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    let idx = selected_rule_idx.get();
                    if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                        if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                            rule.open_on_output = if output_str.is_empty() {
                                None
                            } else {
                                Some(output_str.clone())
                            };
                            debug!("Rule {} open on output: {:?}", idx, rule.open_on_output);
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Open on workspace changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_open_on_workspace_changed(move |workspace| {
            let workspace_str = workspace.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    let idx = selected_rule_idx.get();
                    if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                        if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                            rule.open_on_workspace = if workspace_str.is_empty() {
                                None
                            } else {
                                Some(workspace_str.clone())
                            };
                            debug!(
                                "Rule {} open on workspace: {:?}",
                                idx, rule.open_on_workspace
                            );
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Floating position toggled callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_floating_position_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                        if enabled {
                            rule.default_floating_position = Some(FloatingPosition::default());
                        } else {
                            rule.default_floating_position = None;
                        }
                        debug!("Rule {} floating position enabled: {}", idx, enabled);
                        save_manager.mark_dirty(SettingsCategory::WindowRules);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Floating position X changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_floating_position_x_changed(move |x| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                        if let Some(ref mut pos) = rule.default_floating_position {
                            pos.x = x;
                            debug!("Rule {} floating position X: {}", idx, x);
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Floating position Y changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_floating_position_y_changed(move |y| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                        if let Some(ref mut pos) = rule.default_floating_position {
                            pos.y = y;
                            debug!("Rule {} floating position Y: {}", idx, y);
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Floating position relative-to changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_floating_position_relative_to_changed(move |index| {
            match settings.lock() {
                Ok(mut s) => {
                    let idx = selected_rule_idx.get();
                    if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                        if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                            if let Some(ref mut pos) = rule.default_floating_position {
                                pos.relative_to = PositionRelativeTo::from_index(index);
                                debug!(
                                    "Rule {} floating position relative-to: {:?}",
                                    idx, pos.relative_to
                                );
                                save_manager.mark_dirty(SettingsCategory::WindowRules);
                                save_manager.request_save();
                            }
                        }
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Shadow mode changed callback (0=Default, 1=Custom/On, 2=Off)
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_shadow_mode_changed(move |mode| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                        rule.shadow = match mode {
                            1 => Some(ShadowSettings::default()), // Custom/On
                            2 => Some(ShadowSettings { enabled: false, ..Default::default() }), // Off
                            _ => None, // Default
                        };
                        debug!("Rule {} shadow mode: {}", idx, mode);
                        save_manager.mark_dirty(SettingsCategory::WindowRules);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Shadow softness changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_shadow_softness_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                        if let Some(ref mut shadow) = rule.shadow {
                            shadow.softness = val;
                            debug!("Rule {} shadow softness: {}", idx, val);
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Shadow spread changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_shadow_spread_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                        if let Some(ref mut shadow) = rule.shadow {
                            shadow.spread = val;
                            debug!("Rule {} shadow spread: {}", idx, val);
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Shadow offset X changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_shadow_offset_x_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                        if let Some(ref mut shadow) = rule.shadow {
                            shadow.offset_x = val;
                            debug!("Rule {} shadow offset_x: {}", idx, val);
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Shadow offset Y changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_shadow_offset_y_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                        if let Some(ref mut shadow) = rule.shadow {
                            shadow.offset_y = val;
                            debug!("Rule {} shadow offset_y: {}", idx, val);
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Shadow color changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_shadow_color_changed(move |hex| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                        if let Some(ref mut shadow) = rule.shadow {
                            if let Some(color) = Color::from_hex(hex.as_ref()) {
                                shadow.color = color;
                                debug!("Rule {} shadow color: {}", idx, hex);
                                save_manager.mark_dirty(SettingsCategory::WindowRules);
                                save_manager.request_save();
                            }
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Shadow inactive color changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_shadow_inactive_color_changed(move |hex| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                        if let Some(ref mut shadow) = rule.shadow {
                            if let Some(color) = Color::from_hex(hex.as_ref()) {
                                shadow.inactive_color = color;
                                debug!("Rule {} shadow inactive_color: {}", idx, hex);
                                save_manager.mark_dirty(SettingsCategory::WindowRules);
                                save_manager.request_save();
                            }
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Shadow draw behind window toggled callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_shadow_draw_behind_window_toggled(move |val| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                        if let Some(ref mut shadow) = rule.shadow {
                            shadow.draw_behind_window = val;
                            debug!("Rule {} shadow draw_behind_window: {}", idx, val);
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Tab indicator mode changed callback (0=Default, 1=Custom/On, 2=Off)
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_tab_indicator_mode_changed(move |mode| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                        rule.tab_indicator = match mode {
                            1 => Some(TabIndicatorSettings::default()), // Custom/On
                            2 => Some(TabIndicatorSettings { enabled: false, ..Default::default() }), // Off
                            _ => None, // Default
                        };
                        debug!("Rule {} tab indicator mode: {}", idx, mode);
                        save_manager.mark_dirty(SettingsCategory::WindowRules);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Tab indicator hide when single toggled callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_tab_indicator_hide_when_single_toggled(move |val| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                        if let Some(ref mut ti) = rule.tab_indicator {
                            ti.hide_when_single_tab = val;
                            debug!("Rule {} tab indicator hide_when_single: {}", idx, val);
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Tab indicator place within column toggled callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_tab_indicator_place_within_column_toggled(move |val| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                        if let Some(ref mut ti) = rule.tab_indicator {
                            ti.place_within_column = val;
                            debug!("Rule {} tab indicator place_within_column: {}", idx, val);
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Tab indicator gap changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_tab_indicator_gap_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                        if let Some(ref mut ti) = rule.tab_indicator {
                            ti.gap = val;
                            debug!("Rule {} tab indicator gap: {}", idx, val);
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Tab indicator width changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_tab_indicator_width_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                        if let Some(ref mut ti) = rule.tab_indicator {
                            ti.width = val;
                            debug!("Rule {} tab indicator width: {}", idx, val);
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Tab indicator length changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_tab_indicator_length_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                        if let Some(ref mut ti) = rule.tab_indicator {
                            ti.length_proportion = val;
                            debug!("Rule {} tab indicator length: {}", idx, val);
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Tab indicator position changed callback (0=Left, 1=Right, 2=Top, 3=Bottom)
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_tab_indicator_position_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                        if let Some(ref mut ti) = rule.tab_indicator {
                            ti.position = match val {
                                1 => TabIndicatorPosition::Right,
                                2 => TabIndicatorPosition::Top,
                                3 => TabIndicatorPosition::Bottom,
                                _ => TabIndicatorPosition::Left,
                            };
                            debug!("Rule {} tab indicator position: {:?}", idx, ti.position);
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Tab indicator gaps between tabs changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_tab_indicator_gaps_between_tabs_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                        if let Some(ref mut ti) = rule.tab_indicator {
                            ti.gaps_between_tabs = val;
                            debug!("Rule {} tab indicator gaps_between_tabs: {}", idx, val);
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Tab indicator corner radius changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_tab_indicator_corner_radius_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                        if let Some(ref mut ti) = rule.tab_indicator {
                            ti.corner_radius = val;
                            debug!("Rule {} tab indicator corner_radius: {}", idx, val);
                            save_manager.mark_dirty(SettingsCategory::WindowRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Tab indicator active color changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_tab_indicator_active_color_changed(move |hex| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                        if let Some(ref mut ti) = rule.tab_indicator {
                            if let Some(color) = Color::from_hex(hex.as_ref()) {
                                ti.active = ColorOrGradient::Color(color);
                                debug!("Rule {} tab indicator active_color: {}", idx, hex);
                                save_manager.mark_dirty(SettingsCategory::WindowRules);
                                save_manager.request_save();
                            }
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Tab indicator inactive color changed callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_tab_indicator_inactive_color_changed(move |hex| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.window_rules.rules.len() {
                    if let Some(rule) = s.window_rules.rules.get_mut(idx as usize) {
                        if let Some(ref mut ti) = rule.tab_indicator {
                            if let Some(color) = Color::from_hex(hex.as_ref()) {
                                ti.inactive = ColorOrGradient::Color(color);
                                debug!("Rule {} tab indicator inactive_color: {}", idx, hex);
                                save_manager.mark_dirty(SettingsCategory::WindowRules);
                                save_manager.request_save();
                            }
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Reorder rule callback (drag-and-drop)
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_reorder(move |from, to| {
            let from_idx = from as usize;
            let to_idx = to as usize;

            match settings.lock() {
                Ok(mut s) => {
                    let len = s.window_rules.rules.len();
                    if from_idx >= len || to_idx > len || from_idx == to_idx {
                        return;
                    }

                    // Remove from original position and insert at new position
                    let item = s.window_rules.rules.remove(from_idx);
                    let insert_idx = if to_idx > from_idx {
                        to_idx - 1
                    } else {
                        to_idx
                    };
                    s.window_rules.rules.insert(insert_idx, item);

                    // Update selected index to follow moved item
                    set_selected_index(&selected_rule_idx, insert_idx as i32);
                    reset_match_index(&selected_match_idx);

                    // Update UI
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_window_rules_list(build_rule_list_model(&s.window_rules.rules));
                        ui.set_selected_window_rule_index(insert_idx as i32);

                        if let Some(rule) = s.window_rules.rules.get(insert_idx) {
                            sync_current_rule(&ui, rule, 0);
                        }
                    }

                    debug!(
                        "Reordered window rule from {} to {} (inserted at {})",
                        from_idx, to_idx, insert_idx
                    );
                    save_manager.mark_dirty(SettingsCategory::WindowRules);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Shared storage for running windows (app_id, title pairs)
    // Pre-allocate for typical window count
    let running_windows_data: Arc<Mutex<Vec<(String, String)>>> =
        Arc::new(Mutex::new(Vec::with_capacity(32)));

    // Refresh running windows callback
    {
        let running_windows_data = running_windows_data.clone();
        let ui_weak = ui.as_weak();
        ui.on_window_rule_refresh_windows(move || {
            debug!("Refreshing running windows list");

            match ipc::get_windows() {
                Ok(windows) => {
                    // Store app_id/title pairs and build display strings
                    let window_count = windows.len();
                    let mut data = Vec::with_capacity(window_count);
                    let mut display_strings: Vec<SharedString> = Vec::with_capacity(window_count);

                    for w in windows {
                        // Format: "app_id: title (truncated)"
                        // Use character count, not byte length, to avoid panic on multi-byte UTF-8
                        let title_display = if w.title.chars().count() > 40 {
                            let truncated: String = w.title.chars().take(40).collect();
                            format!("{}...", truncated)
                        } else {
                            w.title.clone()
                        };
                        let display = format!("{}: {}", w.app_id, title_display);
                        display_strings.push(display.into());
                        data.push((w.app_id, w.title));
                    }

                    // Store data for later selection
                    if let Ok(mut wd) = running_windows_data.lock() {
                        *wd = data;
                    }

                    let window_count = display_strings.len();

                    // Update UI
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_running_windows_list(ModelRc::new(VecModel::from(display_strings)));
                    }

                    debug!("Found {} running windows", window_count);
                }
                Err(e) => {
                    warn!("Failed to get running windows: {}", e);
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_running_windows_list(ModelRc::new(VecModel::from(Vec::<
                            SharedString,
                        >::new(
                        ))));
                    }
                }
            }
        });
    }

    // Select running window callback
    {
        let settings = settings.clone();
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let running_windows_data = running_windows_data.clone();
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_window_rule_select_running_window(move |window_index| {
            let idx = window_index as usize;

            // Get the app_id from stored data
            let app_id = running_windows_data
                .lock()
                .ok()
                .and_then(|data| data.get(idx).map(|(app_id, _)| app_id.clone()));

            if let Some(app_id) = app_id {
                match settings.lock() {
                    Ok(mut s) => {
                        let rule_idx = selected_rule_idx.get();
                        let match_idx = selected_match_idx.get();

                        if rule_idx >= 0 && (rule_idx as usize) < s.window_rules.rules.len() {
                            if let Some(rule) = s.window_rules.rules.get_mut(rule_idx as usize) {
                                if (match_idx as usize) < rule.matches.len() {
                                    // Set the app_id for the current match
                                    rule.matches[match_idx as usize].app_id = Some(app_id.clone());

                                    // Update UI
                                    if let Some(ui) = ui_weak.upgrade() {
                                        ui.set_current_match_app_id(app_id.as_str().into());
                                        // Also update the matches list display
                                        ui.set_current_matches_list(build_matches_list_model(
                                            &rule.matches,
                                        ));
                                    }

                                    debug!("Set app_id from picker: {}", app_id);
                                    save_manager.mark_dirty(SettingsCategory::WindowRules);
                                    save_manager.request_save();
                                }
                            }
                        }
                    }
                    Err(e) => error!("Settings lock error: {}", e),
                }
            }
        });
    }
}

/// Sync UI properties with the currently selected rule
fn sync_current_rule(ui: &MainWindow, rule: &WindowRule, selected_match_idx: i32) {
    ui.set_current_rule_name(rule.name.as_str().into());
    ui.set_current_matches_list(build_matches_list_model(&rule.matches));
    ui.set_selected_match_index(selected_match_idx);
    ui.set_current_matches_count(rule.matches.len() as i32);

    // Sync the selected match criteria
    sync_current_match(ui, rule, selected_match_idx);

    // Sync rule properties
    ui.set_current_open_behavior_index(rule.open_behavior.to_index());
    ui.set_current_has_opacity(rule.opacity.is_some());
    ui.set_current_opacity(rule.opacity.unwrap_or(1.0));
    ui.set_current_block_screencast(rule.block_out_from_screencast);
    ui.set_current_has_corner_radius(rule.corner_radius.is_some());
    ui.set_current_corner_radius(rule.corner_radius.unwrap_or(12));
    ui.set_current_clip_to_geometry(rule.clip_to_geometry.unwrap_or(false));
    ui.set_current_open_on_output(rule.open_on_output.clone().unwrap_or_default().into());
    ui.set_current_open_on_workspace(rule.open_on_workspace.clone().unwrap_or_default().into());

    // Sync new dynamic properties
    // VRR: 0=Default (None), 1=On, 2=Off
    ui.set_current_vrr_index(match rule.variable_refresh_rate {
        Some(true) => 1,
        Some(false) => 2,
        None => 0,
    });
    // Column display: 0=Default (None), 1=Tabbed
    ui.set_current_column_display_index(match rule.default_column_display {
        Some(crate::config::models::DefaultColumnDisplay::Tabbed) => 1,
        _ => 0,
    });
    // Tiled state: 0=Default (None), 1=Tiled, 2=Floating
    ui.set_current_tiled_state_index(match rule.tiled_state {
        Some(true) => 1,
        Some(false) => 2,
        None => 0,
    });
    ui.set_current_baba_is_float(rule.baba_is_float.unwrap_or(false));

    // Sync floating position
    ui.set_current_has_floating_position(rule.default_floating_position.is_some());
    if let Some(ref pos) = rule.default_floating_position {
        ui.set_current_floating_x(pos.x);
        ui.set_current_floating_y(pos.y);
        ui.set_current_floating_relative_to(pos.relative_to.to_index());
    } else {
        ui.set_current_floating_x(0);
        ui.set_current_floating_y(0);
        ui.set_current_floating_relative_to(0);
    }

    // Sync shadow override
    // 0=Default (None), 1=Custom/On (enabled=true), 2=Off (enabled=false)
    match &rule.shadow {
        Some(shadow) if shadow.enabled => {
            ui.set_current_shadow_mode(1); // Custom/On
            ui.set_current_shadow_softness(shadow.softness);
            ui.set_current_shadow_spread(shadow.spread);
            ui.set_current_shadow_offset_x(shadow.offset_x);
            ui.set_current_shadow_offset_y(shadow.offset_y);
            ui.set_current_shadow_color(shadow.color.to_hex().into());
            ui.set_current_shadow_inactive_color(shadow.inactive_color.to_hex().into());
            ui.set_current_shadow_draw_behind_window(shadow.draw_behind_window);
        }
        Some(_) => {
            ui.set_current_shadow_mode(2); // Off
            // Set defaults for UI display
            ui.set_current_shadow_softness(30);
            ui.set_current_shadow_spread(5);
            ui.set_current_shadow_offset_x(0);
            ui.set_current_shadow_offset_y(5);
            ui.set_current_shadow_color("#00000070".into());
            ui.set_current_shadow_inactive_color("#00000040".into());
            ui.set_current_shadow_draw_behind_window(false);
        }
        None => {
            ui.set_current_shadow_mode(0); // Default
            // Set defaults for UI display
            ui.set_current_shadow_softness(30);
            ui.set_current_shadow_spread(5);
            ui.set_current_shadow_offset_x(0);
            ui.set_current_shadow_offset_y(5);
            ui.set_current_shadow_color("#00000070".into());
            ui.set_current_shadow_inactive_color("#00000040".into());
            ui.set_current_shadow_draw_behind_window(false);
        }
    }

    // Sync tab indicator override
    // 0=Default (None), 1=Custom/On (enabled=true), 2=Off (enabled=false)
    match &rule.tab_indicator {
        Some(ti) if ti.enabled => {
            ui.set_current_tab_indicator_mode(1); // Custom/On
            ui.set_current_tab_indicator_hide_when_single(ti.hide_when_single_tab);
            ui.set_current_tab_indicator_place_within_column(ti.place_within_column);
            ui.set_current_tab_indicator_gap(ti.gap);
            ui.set_current_tab_indicator_width(ti.width);
            ui.set_current_tab_indicator_length(ti.length_proportion);
            ui.set_current_tab_indicator_position(match ti.position {
                TabIndicatorPosition::Left => 0,
                TabIndicatorPosition::Right => 1,
                TabIndicatorPosition::Top => 2,
                TabIndicatorPosition::Bottom => 3,
            });
            ui.set_current_tab_indicator_gaps_between_tabs(ti.gaps_between_tabs);
            ui.set_current_tab_indicator_corner_radius(ti.corner_radius);
            ui.set_current_tab_indicator_active_color(ti.active.to_hex().into());
            ui.set_current_tab_indicator_inactive_color(ti.inactive.to_hex().into());
        }
        Some(_) => {
            ui.set_current_tab_indicator_mode(2); // Off
            // Set defaults for UI display
            ui.set_current_tab_indicator_hide_when_single(false);
            ui.set_current_tab_indicator_place_within_column(false);
            ui.set_current_tab_indicator_gap(5);
            ui.set_current_tab_indicator_width(4);
            ui.set_current_tab_indicator_length(1.0);
            ui.set_current_tab_indicator_position(0);
            ui.set_current_tab_indicator_gaps_between_tabs(2);
            ui.set_current_tab_indicator_corner_radius(8);
            ui.set_current_tab_indicator_active_color("#cba6f7".into());
            ui.set_current_tab_indicator_inactive_color("#45475a".into());
        }
        None => {
            ui.set_current_tab_indicator_mode(0); // Default
            // Set defaults for UI display
            ui.set_current_tab_indicator_hide_when_single(false);
            ui.set_current_tab_indicator_place_within_column(false);
            ui.set_current_tab_indicator_gap(5);
            ui.set_current_tab_indicator_width(4);
            ui.set_current_tab_indicator_length(1.0);
            ui.set_current_tab_indicator_position(0);
            ui.set_current_tab_indicator_gaps_between_tabs(2);
            ui.set_current_tab_indicator_corner_radius(8);
            ui.set_current_tab_indicator_active_color("#cba6f7".into());
            ui.set_current_tab_indicator_inactive_color("#45475a".into());
        }
    }
}

/// Sync UI properties for the selected match criteria
fn sync_current_match(ui: &MainWindow, rule: &WindowRule, match_idx: i32) {
    // Helper to convert Option<bool> to UI index: None=0 (Any), Some(true)=1 (Yes), Some(false)=2 (No)
    fn option_bool_to_index(opt: Option<bool>) -> i32 {
        match opt {
            Some(true) => 1,
            Some(false) => 2,
            None => 0,
        }
    }

    if match_idx >= 0 && (match_idx as usize) < rule.matches.len() {
        let m = &rule.matches[match_idx as usize];
        ui.set_current_match_app_id(m.app_id.clone().unwrap_or_default().into());
        ui.set_current_match_title(m.title.clone().unwrap_or_default().into());

        // Sync advanced match criteria
        ui.set_current_match_is_floating(option_bool_to_index(m.is_floating));
        ui.set_current_match_is_active(option_bool_to_index(m.is_active));
        ui.set_current_match_is_focused(option_bool_to_index(m.is_focused));
        ui.set_current_match_is_active_in_column(option_bool_to_index(m.is_active_in_column));
        ui.set_current_match_is_window_cast_target(option_bool_to_index(m.is_window_cast_target));
        ui.set_current_match_is_urgent(option_bool_to_index(m.is_urgent));
        ui.set_current_match_at_startup(option_bool_to_index(m.at_startup));
    }
}

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
