//! Layer rules UI callbacks
//!
//! Handles layer rule configuration for layer-shell surfaces like panels and notifications.

use crate::config::models::{BlockOutFrom, LayerRule, LayerRuleMatch, ShadowSettings};
use crate::config::{Settings, SettingsCategory};
use crate::constants::{MAX_LAYER_RULES, MAX_MATCHES_PER_RULE};
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

impl Named for LayerRule {
    fn name(&self) -> &str {
        &self.name
    }
}

/// Build rule list model for UI display
fn build_rule_list_model(rules: &[LayerRule]) -> ModelRc<SharedString> {
    rules_common::build_names_list(rules)
}

/// Build matches list model for UI display
fn build_matches_list_model(matches: &[LayerRuleMatch]) -> ModelRc<SharedString> {
    let mut labels = Vec::with_capacity(matches.len());
    for (i, m) in matches.iter().enumerate() {
        let ns = m.namespace.as_deref().unwrap_or("*");
        labels.push(format!("{}. namespace={}", i + 1, ns).into());
    }
    ModelRc::new(VecModel::from(labels))
}

/// Set up layer rules callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Clone once for all callbacks in this module
    let s = Arc::clone(&settings);
    let sm = Rc::clone(&save_manager);

    // Shared state for tracking selected indices (Rc<Cell> since Slint callbacks are single-threaded)
    let selected_rule_idx = Rc::new(Cell::new(-1i32));
    let selected_match_idx = Rc::new(Cell::new(0i32));

    // Add rule callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_add(move || {
            match settings.lock() {
                Ok(mut s) => {
                    // Check limit before adding
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

                    // Update UI with new rule list
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_layer_rules_list(build_rule_list_model(&s.layer_rules.rules));
                        ui.set_selected_layer_rule_index(new_idx);

                        // Sync current rule properties
                        if let Some(rule) = s.layer_rules.rules.get(new_idx as usize) {
                            sync_current_rule(&ui, rule, 0);
                        }
                    }

                    debug!("Added new layer rule with id {}", new_id);
                    save_manager.mark_dirty(SettingsCategory::LayerRules);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Remove rule callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_remove(move |index| {
            let idx = index as usize;
            match settings.lock() {
                Ok(mut s) => {
                    if idx < s.layer_rules.rules.len() {
                        let name = s.layer_rules.rules[idx].name.clone();
                        s.layer_rules.rules.remove(idx);

                        // Update selected index
                        let new_sel =
                            calculate_new_selection_after_remove(idx, s.layer_rules.rules.len());
                        set_selected_index(&selected_rule_idx, new_sel);
                        reset_match_index(&selected_match_idx);

                        // Update UI
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_layer_rules_list(build_rule_list_model(&s.layer_rules.rules));
                            ui.set_selected_layer_rule_index(new_sel);

                            // Sync current rule properties if there's a selection
                            if new_sel >= 0 {
                                if let Some(rule) = s.layer_rules.rules.get(new_sel as usize) {
                                    sync_current_rule(&ui, rule, 0);
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
        ui.on_layer_rule_select(move |index| {
            set_selected_index(&selected_rule_idx, index);
            reset_match_index(&selected_match_idx);

            // Sync current rule properties to UI
            if let Some(ui) = ui_weak.upgrade() {
                if let Ok(s) = settings.lock() {
                    if is_valid_rule_index(index, s.layer_rules.rules.len()) {
                        if let Some(rule) = s.layer_rules.rules.get(index as usize) {
                            sync_current_rule(&ui, rule, 0);
                        }
                    }
                }
            }

            debug!("Selected layer rule at index {}", index);
        });
    }

    // Reorder rule callback (drag-and-drop)
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_reorder(move |from, to| {
            let from_idx = from as usize;
            let to_idx = to as usize;

            match settings.lock() {
                Ok(mut s) => {
                    let len = s.layer_rules.rules.len();
                    if from_idx >= len || to_idx > len || from_idx == to_idx {
                        return;
                    }

                    // Remove from original position and insert at new position
                    let item = s.layer_rules.rules.remove(from_idx);
                    let insert_idx = if to_idx > from_idx {
                        to_idx - 1
                    } else {
                        to_idx
                    };
                    s.layer_rules.rules.insert(insert_idx, item);

                    // Update selected index to follow moved item
                    set_selected_index(&selected_rule_idx, insert_idx as i32);
                    reset_match_index(&selected_match_idx);

                    // Update UI
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_layer_rules_list(build_rule_list_model(&s.layer_rules.rules));
                        ui.set_selected_layer_rule_index(insert_idx as i32);

                        if let Some(rule) = s.layer_rules.rules.get(insert_idx) {
                            sync_current_rule(&ui, rule, 0);
                        }
                    }

                    debug!(
                        "Reordered layer rule from {} to {} (inserted at {})",
                        from_idx, to_idx, insert_idx
                    );
                    save_manager.mark_dirty(SettingsCategory::LayerRules);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Rule name changed callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_name_changed(move |name| {
            let name_str = name.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    let idx = get_selected_index(&selected_rule_idx);
                    if is_valid_rule_index(idx, s.layer_rules.rules.len()) {
                        if let Some(rule) = s.layer_rules.rules.get_mut(idx as usize) {
                            rule.name = name_str.clone();

                            // Update rule list display
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_layer_rules_list(build_rule_list_model(
                                    &s.layer_rules.rules,
                                ));
                            }

                            debug!("Layer rule {} name changed to: {}", idx, name_str);
                            save_manager.mark_dirty(SettingsCategory::LayerRules);
                            save_manager.request_save();
                        }
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Add match criteria callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_add_match(move || match settings.lock() {
            Ok(mut s) => {
                let rule_idx = get_selected_index(&selected_rule_idx);
                if is_valid_rule_index(rule_idx, s.layer_rules.rules.len()) {
                    if let Some(rule) = s.layer_rules.rules.get_mut(rule_idx as usize) {
                        // Check limit before adding
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
                            sync_current_rule(&ui, rule, new_match_idx);
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

    // Remove match criteria callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_remove_match(move |match_index| match settings.lock() {
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
                                sync_current_rule(&ui, rule, new_match_idx);
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

    // Select match criteria callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        ui.on_layer_rule_select_match(move |match_index| {
            set_selected_index(&selected_match_idx, match_index);

            if let Some(ui) = ui_weak.upgrade() {
                if let Ok(s) = settings.lock() {
                    let rule_idx = get_selected_index(&selected_rule_idx);
                    if is_valid_rule_index(rule_idx, s.layer_rules.rules.len()) {
                        if let Some(rule) = s.layer_rules.rules.get(rule_idx as usize) {
                            sync_current_match(&ui, rule, match_index);
                        }
                    }
                }
            }

            debug!("Selected match criteria at index {}", match_index);
        });
    }

    // Match namespace changed callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_match_namespace_changed(move |namespace| {
            let namespace_str = namespace.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    let rule_idx = get_selected_index(&selected_rule_idx);
                    let match_idx = get_selected_index(&selected_match_idx);
                    if is_valid_rule_index(rule_idx, s.layer_rules.rules.len()) {
                        if let Some(rule) = s.layer_rules.rules.get_mut(rule_idx as usize) {
                            if is_valid_match_index(match_idx, rule.matches.len()) {
                                rule.matches[match_idx as usize].namespace =
                                    parse_optional_string(&namespace_str);

                                // Update matches list display
                                if let Some(ui) = ui_weak.upgrade() {
                                    ui.set_current_layer_rule_matches_list(
                                        build_matches_list_model(&rule.matches),
                                    );
                                }

                                debug!(
                                    "Layer rule {} match {} namespace: {:?}",
                                    rule_idx, match_idx, namespace_str
                                );
                                save_manager.mark_dirty(SettingsCategory::LayerRules);
                                save_manager.request_save();
                            }
                        }
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Match at_startup has toggled callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_match_at_startup_has_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                let rule_idx = selected_rule_idx.get();
                let match_idx = selected_match_idx.get();
                if rule_idx >= 0 && (rule_idx as usize) < s.layer_rules.rules.len() {
                    if let Some(rule) = s.layer_rules.rules.get_mut(rule_idx as usize) {
                        if (match_idx as usize) < rule.matches.len() {
                            if enabled {
                                rule.matches[match_idx as usize].at_startup = Some(true);
                            } else {
                                rule.matches[match_idx as usize].at_startup = None;
                            }
                            save_manager.mark_dirty(SettingsCategory::LayerRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Match at_startup toggled callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let selected_match_idx = Rc::clone(&selected_match_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_match_at_startup_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                let rule_idx = selected_rule_idx.get();
                let match_idx = selected_match_idx.get();
                if rule_idx >= 0 && (rule_idx as usize) < s.layer_rules.rules.len() {
                    if let Some(rule) = s.layer_rules.rules.get_mut(rule_idx as usize) {
                        if (match_idx as usize) < rule.matches.len() {
                            rule.matches[match_idx as usize].at_startup = Some(enabled);
                            save_manager.mark_dirty(SettingsCategory::LayerRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Block out from toggled callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_block_out_from_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.layer_rules.rules.len() {
                    if let Some(rule) = s.layer_rules.rules.get_mut(idx as usize) {
                        if enabled {
                            rule.block_out_from = Some(BlockOutFrom::Screencast);
                        } else {
                            rule.block_out_from = None;
                        }
                        save_manager.mark_dirty(SettingsCategory::LayerRules);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Block out from changed callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_block_out_from_changed(move |index| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.layer_rules.rules.len() {
                    if let Some(rule) = s.layer_rules.rules.get_mut(idx as usize) {
                        rule.block_out_from = Some(BlockOutFrom::from_index(index));
                        save_manager.mark_dirty(SettingsCategory::LayerRules);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Opacity toggled callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_opacity_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.layer_rules.rules.len() {
                    if let Some(rule) = s.layer_rules.rules.get_mut(idx as usize) {
                        if enabled {
                            rule.opacity = Some(1.0);
                        } else {
                            rule.opacity = None;
                        }
                        save_manager.mark_dirty(SettingsCategory::LayerRules);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Opacity changed callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_opacity_changed(move |opacity| {
            let clamped = opacity.clamp(0.0, 1.0);
            match settings.lock() {
                Ok(mut s) => {
                    let idx = selected_rule_idx.get();
                    if idx >= 0 && (idx as usize) < s.layer_rules.rules.len() {
                        if let Some(rule) = s.layer_rules.rules.get_mut(idx as usize) {
                            rule.opacity = Some(clamped);
                            save_manager.mark_dirty(SettingsCategory::LayerRules);
                            save_manager.request_save();
                        }
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Corner radius toggled callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_corner_radius_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.layer_rules.rules.len() {
                    if let Some(rule) = s.layer_rules.rules.get_mut(idx as usize) {
                        if enabled {
                            rule.geometry_corner_radius = Some(12);
                        } else {
                            rule.geometry_corner_radius = None;
                        }
                        save_manager.mark_dirty(SettingsCategory::LayerRules);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Corner radius changed callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_corner_radius_changed(move |radius| {
            let clamped = radius.clamp(0, 32);
            match settings.lock() {
                Ok(mut s) => {
                    let idx = selected_rule_idx.get();
                    if idx >= 0 && (idx as usize) < s.layer_rules.rules.len() {
                        if let Some(rule) = s.layer_rules.rules.get_mut(idx as usize) {
                            rule.geometry_corner_radius = Some(clamped);
                            save_manager.mark_dirty(SettingsCategory::LayerRules);
                            save_manager.request_save();
                        }
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Place within backdrop toggled callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_place_within_backdrop_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.layer_rules.rules.len() {
                    if let Some(rule) = s.layer_rules.rules.get_mut(idx as usize) {
                        rule.place_within_backdrop = enabled;
                        save_manager.mark_dirty(SettingsCategory::LayerRules);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Baba is float toggled callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_baba_is_float_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.layer_rules.rules.len() {
                    if let Some(rule) = s.layer_rules.rules.get_mut(idx as usize) {
                        rule.baba_is_float = enabled;
                        save_manager.mark_dirty(SettingsCategory::LayerRules);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Shadow toggled callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_shadow_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.layer_rules.rules.len() {
                    if let Some(rule) = s.layer_rules.rules.get_mut(idx as usize) {
                        if enabled {
                            rule.shadow = Some(ShadowSettings::default());
                        } else {
                            rule.shadow = None;
                        }
                        save_manager.mark_dirty(SettingsCategory::LayerRules);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Shadow enabled toggled callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_shadow_enabled_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.layer_rules.rules.len() {
                    if let Some(rule) = s.layer_rules.rules.get_mut(idx as usize) {
                        if let Some(ref mut shadow) = rule.shadow {
                            shadow.enabled = enabled;
                            save_manager.mark_dirty(SettingsCategory::LayerRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Shadow softness changed callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_shadow_softness_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.layer_rules.rules.len() {
                    if let Some(rule) = s.layer_rules.rules.get_mut(idx as usize) {
                        if let Some(ref mut shadow) = rule.shadow {
                            shadow.softness = val;
                            save_manager.mark_dirty(SettingsCategory::LayerRules);
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
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_shadow_spread_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.layer_rules.rules.len() {
                    if let Some(rule) = s.layer_rules.rules.get_mut(idx as usize) {
                        if let Some(ref mut shadow) = rule.shadow {
                            shadow.spread = val;
                            save_manager.mark_dirty(SettingsCategory::LayerRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Shadow offset x changed callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_shadow_offset_x_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.layer_rules.rules.len() {
                    if let Some(rule) = s.layer_rules.rules.get_mut(idx as usize) {
                        if let Some(ref mut shadow) = rule.shadow {
                            shadow.offset_x = val;
                            save_manager.mark_dirty(SettingsCategory::LayerRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Shadow offset y changed callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_shadow_offset_y_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.layer_rules.rules.len() {
                    if let Some(rule) = s.layer_rules.rules.get_mut(idx as usize) {
                        if let Some(ref mut shadow) = rule.shadow {
                            shadow.offset_y = val;
                            save_manager.mark_dirty(SettingsCategory::LayerRules);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Shadow draw behind toggled callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_shadow_draw_behind_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_rule_idx.get();
                if idx >= 0 && (idx as usize) < s.layer_rules.rules.len() {
                    if let Some(rule) = s.layer_rules.rules.get_mut(idx as usize) {
                        if let Some(ref mut shadow) = rule.shadow {
                            shadow.draw_behind_window = enabled;
                            save_manager.mark_dirty(SettingsCategory::LayerRules);
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
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_shadow_color_changed(move |color| {
            match settings.lock() {
                Ok(mut s) => {
                    let idx = selected_rule_idx.get();
                    if idx >= 0 && (idx as usize) < s.layer_rules.rules.len() {
                        if let Some(rule) = s.layer_rules.rules.get_mut(idx as usize) {
                            if let Some(ref mut shadow) = rule.shadow {
                                shadow.color = crate::types::Color {
                                    r: color.red(),
                                    g: color.green(),
                                    b: color.blue(),
                                    a: color.alpha(),
                                };

                                // Update hex value
                                if let Some(ui) = ui_weak.upgrade() {
                                    ui.set_current_layer_rule_shadow_color_hex(
                                        shadow.color.to_hex().into(),
                                    );
                                }
                                save_manager.mark_dirty(SettingsCategory::LayerRules);
                                save_manager.request_save();
                            }
                        }
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Shadow inactive color changed callback
    {
        let settings = Arc::clone(&s);
        let selected_rule_idx = Rc::clone(&selected_rule_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&sm);
        ui.on_layer_rule_shadow_inactive_color_changed(move |color| {
            match settings.lock() {
                Ok(mut s) => {
                    let idx = selected_rule_idx.get();
                    if idx >= 0 && (idx as usize) < s.layer_rules.rules.len() {
                        if let Some(rule) = s.layer_rules.rules.get_mut(idx as usize) {
                            if let Some(ref mut shadow) = rule.shadow {
                                shadow.inactive_color = crate::types::Color {
                                    r: color.red(),
                                    g: color.green(),
                                    b: color.blue(),
                                    a: color.alpha(),
                                };

                                // Update hex value
                                if let Some(ui) = ui_weak.upgrade() {
                                    ui.set_current_layer_rule_shadow_inactive_color_hex(
                                        shadow.inactive_color.to_hex().into(),
                                    );
                                }
                                save_manager.mark_dirty(SettingsCategory::LayerRules);
                                save_manager.request_save();
                            }
                        }
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }
}

/// Sync UI properties with the currently selected rule
fn sync_current_rule(ui: &MainWindow, rule: &LayerRule, selected_match_idx: i32) {
    ui.set_current_layer_rule_name(rule.name.as_str().into());
    ui.set_current_layer_rule_matches_list(build_matches_list_model(&rule.matches));
    ui.set_selected_layer_rule_match_index(selected_match_idx);
    ui.set_current_layer_rule_matches_count(rule.matches.len() as i32);

    // Sync the selected match criteria
    sync_current_match(ui, rule, selected_match_idx);

    // Block out from
    ui.set_current_layer_rule_has_block_out_from(rule.block_out_from.is_some());
    ui.set_current_layer_rule_block_out_from_index(
        rule.block_out_from
            .as_ref()
            .map(|b| b.to_index())
            .unwrap_or(0),
    );

    // Opacity
    ui.set_current_layer_rule_has_opacity(rule.opacity.is_some());
    ui.set_current_layer_rule_opacity(rule.opacity.unwrap_or(1.0));

    // Corner radius
    ui.set_current_layer_rule_has_corner_radius(rule.geometry_corner_radius.is_some());
    ui.set_current_layer_rule_corner_radius(rule.geometry_corner_radius.unwrap_or(12));

    // Boolean flags
    ui.set_current_layer_rule_place_within_backdrop(rule.place_within_backdrop);
    ui.set_current_layer_rule_baba_is_float(rule.baba_is_float);

    // Shadow
    let has_shadow = rule.shadow.is_some();
    ui.set_current_layer_rule_has_shadow(has_shadow);
    if let Some(ref shadow) = rule.shadow {
        ui.set_current_layer_rule_shadow_enabled(shadow.enabled);
        ui.set_current_layer_rule_shadow_softness(shadow.softness);
        ui.set_current_layer_rule_shadow_spread(shadow.spread);
        ui.set_current_layer_rule_shadow_offset_x(shadow.offset_x);
        ui.set_current_layer_rule_shadow_offset_y(shadow.offset_y);
        ui.set_current_layer_rule_shadow_draw_behind(shadow.draw_behind_window);

        let color = slint::Color::from_argb_u8(
            shadow.color.a,
            shadow.color.r,
            shadow.color.g,
            shadow.color.b,
        );
        ui.set_current_layer_rule_shadow_color(color);
        ui.set_current_layer_rule_shadow_color_hex(shadow.color.to_hex().into());

        let inactive_color = slint::Color::from_argb_u8(
            shadow.inactive_color.a,
            shadow.inactive_color.r,
            shadow.inactive_color.g,
            shadow.inactive_color.b,
        );
        ui.set_current_layer_rule_shadow_inactive_color(inactive_color);
        ui.set_current_layer_rule_shadow_inactive_color_hex(shadow.inactive_color.to_hex().into());
    }
}

/// Sync UI properties for the selected match criteria
fn sync_current_match(ui: &MainWindow, rule: &LayerRule, match_idx: i32) {
    if match_idx >= 0 && (match_idx as usize) < rule.matches.len() {
        let m = &rule.matches[match_idx as usize];
        ui.set_current_layer_rule_match_namespace(m.namespace.clone().unwrap_or_default().into());
        ui.set_current_layer_rule_match_has_at_startup(m.at_startup.is_some());
        ui.set_current_layer_rule_match_at_startup(m.at_startup.unwrap_or(true));
    }
}

/// Public function to build rule list model for sync
pub fn build_rules_list_model(rules: &[LayerRule]) -> ModelRc<SharedString> {
    build_rule_list_model(rules)
}

/// Public function to build matches list model for sync
pub fn build_matches_model(matches: &[LayerRuleMatch]) -> ModelRc<SharedString> {
    build_matches_list_model(matches)
}
