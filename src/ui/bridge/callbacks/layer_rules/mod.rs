//! Dynamic layer rules UI callbacks
//!
//! Handles layer rule configuration using model-driven dynamic UI.
//!
//! This module is split into:
//! - `mod.rs` - Setup function and callback handlers
//! - `populate.rs` - UI model population functions

mod populate;

use crate::config::models::{BlockOutFrom, LayerRule, LayerRuleMatch, ShadowSettings};
use crate::config::{Settings, SettingsCategory};
use crate::constants::{MAX_LAYER_RULES, MAX_MATCHES_PER_RULE};
use crate::MainWindow;
use log::{debug, error, warn};
use slint::{ComponentHandle, ModelRc, SharedString};
use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::SaveManager;
use super::rules_common::{
    self, calculate_new_selection_after_remove, get_selected_index, is_valid_match_index,
    is_valid_rule_index, parse_optional_string, reset_match_index, set_selected_index, Named,
};
use populate::{
    build_matches_list_model, build_rule_list_model, populate_match_settings, sync_rule_models,
};

/// Implement Named trait for LayerRule to enable rule list building
impl Named for LayerRule {
    fn name(&self) -> &str {
        &self.name
    }
}

// Re-export public functions for sync module
pub use populate::build_matches_list_model as build_matches_model;

/// Public function to build rule list model for sync
pub fn build_rules_list_model(rules: &[LayerRule]) -> ModelRc<SharedString> {
    build_rule_list_model(rules)
}

/// Public function to sync all rule models for a given rule
pub fn sync_all_rule_models(ui: &MainWindow, rule: &LayerRule, match_idx: i32) {
    sync_rule_models(ui, rule, match_idx);
}

/// Set up dynamic layer rules callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    let s = Arc::clone(&settings);
    let sm = Rc::clone(&save_manager);

    // Shared state for tracking selected indices
    let selected_rule_idx = Rc::new(Cell::new(-1i32));
    let selected_match_idx = Rc::new(Cell::new(0i32));

    // Add rule callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_add_dynamic(move || match settings.lock() {
            Ok(mut s) => {
                if s.layer_rules.rules.len() >= MAX_LAYER_RULES {
                    warn!("Maximum layer rules limit ({}) reached", MAX_LAYER_RULES);
                    return;
                }

                let new_id = s.layer_rules.next_id;
                s.layer_rules.next_id += 1;

                let rule = LayerRule {
                    id: new_id,
                    name: format!("Layer Rule {}", new_id + 1),
                    ..Default::default()
                };
                s.layer_rules.rules.push(rule);

                let new_idx = (s.layer_rules.rules.len() - 1) as i32;
                set_selected_index(&selected_rule_idx, new_idx);
                reset_match_index(&selected_match_idx);

                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_layer_rules_list_dynamic(build_rule_list_model(&s.layer_rules.rules));
                    ui.set_layer_rules_selected_index(new_idx);

                    if let Some(rule) = s.layer_rules.rules.get(new_idx as usize) {
                        sync_rule_models(&ui, rule, 0);
                    }
                }

                debug!("Added new layer rule with id {}", new_id);
                save_manager.mark_dirty(SettingsCategory::LayerRules);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Remove rule callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_remove_dynamic(move |index| {
            let idx = index as usize;
            match settings.lock() {
                Ok(mut s) => {
                    if idx < s.layer_rules.rules.len() {
                        let name = s.layer_rules.rules[idx].name.clone();
                        s.layer_rules.rules.remove(idx);

                        let new_sel =
                            calculate_new_selection_after_remove(idx, s.layer_rules.rules.len());
                        set_selected_index(&selected_rule_idx, new_sel);
                        reset_match_index(&selected_match_idx);

                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_layer_rules_list_dynamic(build_rule_list_model(
                                &s.layer_rules.rules,
                            ));
                            ui.set_layer_rules_selected_index(new_sel);

                            if new_sel >= 0 {
                                if let Some(rule) = s.layer_rules.rules.get(new_sel as usize) {
                                    sync_rule_models(&ui, rule, 0);
                                }
                            }
                        }

                        debug!("Removed layer rule at index {}: {}", idx, name);
                        save_manager.mark_dirty(SettingsCategory::LayerRules);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Select rule callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        ui.on_layer_rule_select_dynamic(move |index| {
            set_selected_index(&selected_rule_idx, index);
            reset_match_index(&selected_match_idx);

            if let Some(ui) = ui_weak.upgrade() {
                if let Ok(s) = settings.lock() {
                    if is_valid_rule_index(index, s.layer_rules.rules.len()) {
                        if let Some(rule) = s.layer_rules.rules.get(index as usize) {
                            sync_rule_models(&ui, rule, 0);
                        }
                    }
                }
            }

            debug!("Selected layer rule at index {}", index);
        });
    }

    // Reorder rule callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_reorder_dynamic(move |from, to| {
            let from_idx = from as usize;
            let to_idx = to as usize;

            match settings.lock() {
                Ok(mut s) => {
                    let len = s.layer_rules.rules.len();
                    if from_idx >= len || to_idx > len || from_idx == to_idx {
                        return;
                    }

                    let item = s.layer_rules.rules.remove(from_idx);
                    let insert_idx = if to_idx > from_idx {
                        to_idx - 1
                    } else {
                        to_idx
                    };
                    s.layer_rules.rules.insert(insert_idx, item);

                    set_selected_index(&selected_rule_idx, insert_idx as i32);
                    reset_match_index(&selected_match_idx);

                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_layer_rules_list_dynamic(build_rule_list_model(
                            &s.layer_rules.rules,
                        ));
                        ui.set_layer_rules_selected_index(insert_idx as i32);

                        if let Some(rule) = s.layer_rules.rules.get(insert_idx) {
                            sync_rule_models(&ui, rule, 0);
                        }
                    }

                    debug!("Reordered layer rule from {} to {}", from_idx, to_idx);
                    save_manager.mark_dirty(SettingsCategory::LayerRules);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Add match callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_add_match_dynamic(move || match settings.lock() {
            Ok(mut s) => {
                let rule_idx = get_selected_index(&selected_rule_idx);
                if is_valid_rule_index(rule_idx, s.layer_rules.rules.len()) {
                    if let Some(rule) = s.layer_rules.rules.get_mut(rule_idx as usize) {
                        if rule.matches.len() >= MAX_MATCHES_PER_RULE {
                            warn!(
                                "Maximum matches per rule limit ({}) reached",
                                MAX_MATCHES_PER_RULE
                            );
                            return;
                        }

                        rule.matches.push(LayerRuleMatch::default());
                        let new_match_idx = (rule.matches.len() - 1) as i32;
                        set_selected_index(&selected_match_idx, new_match_idx);

                        if let Some(ui) = ui_weak.upgrade() {
                            sync_rule_models(&ui, rule, new_match_idx);
                        }

                        debug!("Added new match criteria to layer rule {}", rule_idx);
                        save_manager.mark_dirty(SettingsCategory::LayerRules);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Remove match callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_remove_match_dynamic(move |match_index| match settings.lock() {
            Ok(mut s) => {
                let rule_idx = get_selected_index(&selected_rule_idx);
                if is_valid_rule_index(rule_idx, s.layer_rules.rules.len()) {
                    if let Some(rule) = s.layer_rules.rules.get_mut(rule_idx as usize) {
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
                                sync_rule_models(&ui, rule, new_match_idx);
                            }

                            debug!(
                                "Removed match criteria {} from layer rule {}",
                                match_idx, rule_idx
                            );
                            save_manager.mark_dirty(SettingsCategory::LayerRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Select match callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        ui.on_layer_rule_select_match_dynamic(move |match_index| {
            set_selected_index(&selected_match_idx, match_index);

            if let Some(ui) = ui_weak.upgrade() {
                if let Ok(s) = settings.lock() {
                    let rule_idx = get_selected_index(&selected_rule_idx);
                    if is_valid_rule_index(rule_idx, s.layer_rules.rules.len()) {
                        if let Some(rule) = s.layer_rules.rules.get(rule_idx as usize) {
                            // Just update match settings, not all models
                            ui.set_layer_rules_match_settings(populate_match_settings(
                                rule,
                                match_index as usize,
                            ));
                        }
                    }
                }
            }

            debug!("Selected match criteria at index {}", match_index);
        });
    }

    // Generic toggle callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_setting_toggle_changed(move |id, value| {
            let id_str = id.as_str();
            match settings.lock() {
                Ok(mut s) => {
                    let rule_idx = get_selected_index(&selected_rule_idx);
                    let match_idx = get_selected_index(&selected_match_idx);

                    if !is_valid_rule_index(rule_idx, s.layer_rules.rules.len()) {
                        return;
                    }

                    if let Some(rule) = s.layer_rules.rules.get_mut(rule_idx as usize) {
                        let mut needs_model_refresh = false;

                        match id_str {
                            // Match settings
                            "match_has_at_startup" => {
                                if is_valid_match_index(match_idx, rule.matches.len()) {
                                    rule.matches[match_idx as usize].at_startup =
                                        if value { Some(true) } else { None };
                                    needs_model_refresh = true;
                                }
                            }
                            "match_at_startup" => {
                                if is_valid_match_index(match_idx, rule.matches.len()) {
                                    rule.matches[match_idx as usize].at_startup = Some(value);
                                }
                            }

                            // Visual settings
                            "has_opacity" => {
                                rule.opacity = if value { Some(1.0) } else { None };
                                needs_model_refresh = true;
                            }
                            "has_corner_radius" => {
                                rule.geometry_corner_radius = if value { Some(12) } else { None };
                                needs_model_refresh = true;
                            }
                            "has_block_out_from" => {
                                rule.block_out_from = if value {
                                    Some(BlockOutFrom::Screencast)
                                } else {
                                    None
                                };
                                needs_model_refresh = true;
                            }

                            // Shadow settings
                            "has_shadow" => {
                                rule.shadow = if value {
                                    Some(ShadowSettings::default())
                                } else {
                                    None
                                };
                                needs_model_refresh = true;
                            }
                            "shadow_enabled" => {
                                if let Some(ref mut shadow) = rule.shadow {
                                    shadow.enabled = value;
                                    needs_model_refresh = true;
                                }
                            }
                            "shadow_draw_behind" => {
                                if let Some(ref mut shadow) = rule.shadow {
                                    shadow.draw_behind_window = value;
                                }
                            }

                            // Advanced settings
                            "place_within_backdrop" => {
                                rule.place_within_backdrop = value;
                            }
                            "baba_is_float" => {
                                rule.baba_is_float = value;
                            }

                            _ => {
                                debug!("Unknown toggle setting: {}", id_str);
                                return;
                            }
                        }

                        // Refresh models if visibility changed
                        if needs_model_refresh {
                            if let Some(ui) = ui_weak.upgrade() {
                                sync_rule_models(&ui, rule, match_idx);
                            }
                        }

                        debug!("Layer rule toggle {} = {}", id_str, value);
                        save_manager.mark_dirty(SettingsCategory::LayerRules);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Generic slider int callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_setting_slider_int_changed(move |id, value| {
            let id_str = id.as_str();
            match settings.lock() {
                Ok(mut s) => {
                    let rule_idx = get_selected_index(&selected_rule_idx);

                    if !is_valid_rule_index(rule_idx, s.layer_rules.rules.len()) {
                        return;
                    }

                    if let Some(rule) = s.layer_rules.rules.get_mut(rule_idx as usize) {
                        match id_str {
                            "corner_radius" => {
                                rule.geometry_corner_radius = Some(value.clamp(0, 32));
                            }
                            "shadow_softness" => {
                                if let Some(ref mut shadow) = rule.shadow {
                                    shadow.softness = value.clamp(0, 100);
                                }
                            }
                            "shadow_spread" => {
                                if let Some(ref mut shadow) = rule.shadow {
                                    shadow.spread = value.clamp(0, 50);
                                }
                            }
                            "shadow_offset_x" => {
                                if let Some(ref mut shadow) = rule.shadow {
                                    shadow.offset_x = value.clamp(-50, 50);
                                }
                            }
                            "shadow_offset_y" => {
                                if let Some(ref mut shadow) = rule.shadow {
                                    shadow.offset_y = value.clamp(-50, 50);
                                }
                            }
                            _ => {
                                debug!("Unknown slider int setting: {}", id_str);
                                return;
                            }
                        }

                        debug!("Layer rule slider int {} = {}", id_str, value);
                        save_manager.mark_dirty(SettingsCategory::LayerRules);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Generic slider float callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_setting_slider_float_changed(move |id, value| {
            let id_str = id.as_str();
            match settings.lock() {
                Ok(mut s) => {
                    let rule_idx = get_selected_index(&selected_rule_idx);

                    if !is_valid_rule_index(rule_idx, s.layer_rules.rules.len()) {
                        return;
                    }

                    if let Some(rule) = s.layer_rules.rules.get_mut(rule_idx as usize) {
                        match id_str {
                            "opacity" => {
                                rule.opacity = Some(value.clamp(0.0, 1.0));
                            }
                            _ => {
                                debug!("Unknown slider float setting: {}", id_str);
                                return;
                            }
                        }

                        debug!("Layer rule slider float {} = {}", id_str, value);
                        save_manager.mark_dirty(SettingsCategory::LayerRules);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Generic combo callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_setting_combo_changed(move |id, index| {
            let id_str = id.as_str();
            match settings.lock() {
                Ok(mut s) => {
                    let rule_idx = get_selected_index(&selected_rule_idx);

                    if !is_valid_rule_index(rule_idx, s.layer_rules.rules.len()) {
                        return;
                    }

                    if let Some(rule) = s.layer_rules.rules.get_mut(rule_idx as usize) {
                        match id_str {
                            "block_out_from" => {
                                rule.block_out_from = Some(BlockOutFrom::from_index(index));
                            }
                            _ => {
                                debug!("Unknown combo setting: {}", id_str);
                                return;
                            }
                        }

                        debug!("Layer rule combo {} = {}", id_str, index);
                        save_manager.mark_dirty(SettingsCategory::LayerRules);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Generic text callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_setting_text_changed(move |id, value| {
            let id_str = id.as_str();
            let mut value_str = value.to_string();

            // Validate length based on field type
            let max_len = match id_str {
                // Pattern fields use shorter limit
                "match_namespace" => crate::constants::MAX_PATTERN_LENGTH,
                // Other text fields use general limit
                _ => crate::constants::MAX_STRING_LENGTH,
            };

            if value_str.len() > max_len {
                warn!("Text input '{}' exceeds maximum length, truncating", id_str);
                value_str.truncate(max_len);
            }

            match settings.lock() {
                Ok(mut s) => {
                    let rule_idx = get_selected_index(&selected_rule_idx);
                    let match_idx = get_selected_index(&selected_match_idx);

                    if !is_valid_rule_index(rule_idx, s.layer_rules.rules.len()) {
                        return;
                    }

                    if let Some(rule) = s.layer_rules.rules.get_mut(rule_idx as usize) {
                        match id_str {
                            "rule_name" => {
                                rule.name = value_str.clone();
                                // Update rule list display
                                if let Some(ui) = ui_weak.upgrade() {
                                    ui.set_layer_rules_list_dynamic(build_rule_list_model(
                                        &s.layer_rules.rules,
                                    ));
                                }
                            }
                            "match_namespace" => {
                                if is_valid_match_index(match_idx, rule.matches.len()) {
                                    rule.matches[match_idx as usize].namespace =
                                        parse_optional_string(&value_str);
                                    // Update matches list display
                                    if let Some(ui) = ui_weak.upgrade() {
                                        ui.set_layer_rules_matches_list(build_matches_list_model(
                                            &rule.matches,
                                        ));
                                    }
                                }
                            }
                            // Color inputs are handled as text but parsed as colors
                            "shadow_color" => {
                                if let Some(ref mut shadow) = rule.shadow {
                                    if let Some(color) = crate::types::Color::from_hex(&value_str) {
                                        shadow.color = color;
                                    }
                                }
                            }
                            "shadow_inactive_color" => {
                                if let Some(ref mut shadow) = rule.shadow {
                                    if let Some(color) = crate::types::Color::from_hex(&value_str) {
                                        shadow.inactive_color = color;
                                    }
                                }
                            }
                            _ => {
                                debug!("Unknown text setting: {}", id_str);
                                return;
                            }
                        }

                        debug!("Layer rule text {} = {}", id_str, value_str);
                        save_manager.mark_dirty(SettingsCategory::LayerRules);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Generic color callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_setting_color_changed(move |id, color| {
            let id_str = id.as_str();

            match settings.lock() {
                Ok(mut s) => {
                    let rule_idx = get_selected_index(&selected_rule_idx);
                    let match_idx = get_selected_index(&selected_match_idx);

                    if !is_valid_rule_index(rule_idx, s.layer_rules.rules.len()) {
                        return;
                    }

                    if let Some(rule) = s.layer_rules.rules.get_mut(rule_idx as usize) {
                        let rust_color = crate::types::Color {
                            r: color.red(),
                            g: color.green(),
                            b: color.blue(),
                            a: color.alpha(),
                        };

                        match id_str {
                            "shadow_color" => {
                                if let Some(ref mut shadow) = rule.shadow {
                                    shadow.color = rust_color;
                                    // Update hex display in model
                                    if let Some(ui) = ui_weak.upgrade() {
                                        sync_rule_models(&ui, rule, match_idx);
                                    }
                                }
                            }
                            "shadow_inactive_color" => {
                                if let Some(ref mut shadow) = rule.shadow {
                                    shadow.inactive_color = rust_color;
                                    if let Some(ui) = ui_weak.upgrade() {
                                        sync_rule_models(&ui, rule, match_idx);
                                    }
                                }
                            }
                            _ => {
                                debug!("Unknown color setting: {}", id_str);
                                return;
                            }
                        }

                        debug!("Layer rule color {} changed", id_str);
                        save_manager.mark_dirty(SettingsCategory::LayerRules);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }
}
