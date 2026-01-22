//! Layer rules settings message handler

use crate::config::SettingsCategory;
use crate::messages::{LayerRulesMessage as M, Message};
use iced::Task;

impl super::super::App {
    /// Updates layer rules settings
    pub(in crate::app) fn update_layer_rules(&mut self, msg: M) -> Task<Message> {
        

        match msg {
            M::AddRule => {
                // Create a new rule with default values
                let new_rule = crate::config::models::LayerRule {
                    id: self.settings.layer_rules.next_id,
                    ..Default::default()
                };
                self.settings.layer_rules.next_id += 1;
                self.settings.layer_rules.rules.push(new_rule.clone());
                self.selected_layer_rule_id = Some(new_rule.id);
                log::info!("Added new layer rule with ID {}", new_rule.id);
            }

            M::DeleteRule(rule_id) => {
                self.settings.layer_rules.rules.retain(|r| r.id != rule_id);
                if self.selected_layer_rule_id == Some(rule_id) {
                    self.selected_layer_rule_id = self.settings.layer_rules.rules.first().map(|r| r.id);
                }
                log::info!("Deleted layer rule {}", rule_id);
            }

            M::SelectRule(rule_id) => {
                self.selected_layer_rule_id = Some(rule_id);
            }

            M::DuplicateRule(rule_id) => {
                if let Some(rule) = self.settings.layer_rules.rules.iter().find(|r| r.id == rule_id).cloned() {
                    let mut new_rule = rule;
                    new_rule.id = self.settings.layer_rules.next_id;
                    self.settings.layer_rules.next_id += 1;
                    new_rule.name = format!("{} (copy)", new_rule.name);
                    self.settings.layer_rules.rules.push(new_rule.clone());
                    self.selected_layer_rule_id = Some(new_rule.id);
                    log::info!("Duplicated layer rule {} to {}", rule_id, new_rule.id);
                }
            }

            M::ReorderRule(rule_id, move_up) => {
                if let Some(idx) = self.settings.layer_rules.rules.iter().position(|r| r.id == rule_id) {
                    let new_idx = if move_up && idx > 0 {
                        idx - 1
                    } else if !move_up && idx < self.settings.layer_rules.rules.len() - 1 {
                        idx + 1
                    } else {
                        idx
                    };

                    if new_idx != idx {
                        let rule = self.settings.layer_rules.rules.remove(idx);
                        self.settings.layer_rules.rules.insert(new_idx, rule);
                    }
                }
            }

            M::SetRuleName(rule_id, name) => {
                if let Some(rule) = self.settings.layer_rules.rules.iter_mut().find(|r| r.id == rule_id) {
                    rule.name = name;
                }
            }

            M::AddMatch(rule_id) => {
                if let Some(rule) = self.settings.layer_rules.rules.iter_mut().find(|r| r.id == rule_id) {
                    rule.matches.push(crate::config::models::LayerRuleMatch::default());
                }
            }

            M::RemoveMatch(rule_id, match_idx) => {
                if let Some(rule) = self.settings.layer_rules.rules.iter_mut().find(|r| r.id == rule_id) {
                    if match_idx < rule.matches.len() {
                        rule.matches.remove(match_idx);
                    }
                }
            }

            M::SetMatchNamespace(rule_id, match_idx, namespace) => {
                if let Some(rule) = self.settings.layer_rules.rules.iter_mut().find(|r| r.id == rule_id) {
                    if let Some(match_data) = rule.matches.get_mut(match_idx) {
                        match_data.namespace = if namespace.is_empty() { None } else { Some(namespace) };
                    }
                }
            }

            M::SetMatchAtStartup(rule_id, match_idx, value) => {
                if let Some(rule) = self.settings.layer_rules.rules.iter_mut().find(|r| r.id == rule_id) {
                    if let Some(match_data) = rule.matches.get_mut(match_idx) {
                        match_data.at_startup = value;
                    }
                }
            }

            M::SetBlockOutFrom(rule_id, value) => {
                if let Some(rule) = self.settings.layer_rules.rules.iter_mut().find(|r| r.id == rule_id) {
                    rule.block_out_from = value;
                }
            }

            M::SetOpacity(rule_id, value) => {
                if let Some(rule) = self.settings.layer_rules.rules.iter_mut().find(|r| r.id == rule_id) {
                    rule.opacity = value;
                }
            }

            M::SetCornerRadius(rule_id, value) => {
                if let Some(rule) = self.settings.layer_rules.rules.iter_mut().find(|r| r.id == rule_id) {
                    rule.geometry_corner_radius = value;
                }
            }

            M::SetPlaceWithinBackdrop(rule_id, value) => {
                if let Some(rule) = self.settings.layer_rules.rules.iter_mut().find(|r| r.id == rule_id) {
                    rule.place_within_backdrop = value;
                }
            }

            M::SetBabaIsFloat(rule_id, value) => {
                if let Some(rule) = self.settings.layer_rules.rules.iter_mut().find(|r| r.id == rule_id) {
                    rule.baba_is_float = value;
                }
            }

            M::SetShadow(rule_id, value) => {
                if let Some(rule) = self.settings.layer_rules.rules.iter_mut().find(|r| r.id == rule_id) {
                    rule.shadow = value;
                }
            }

            M::ToggleSection(rule_id, section_name) => {
                let key = (rule_id, section_name);
                let expanded = self.layer_rule_sections_expanded.get(&key).copied().unwrap_or(true);
                self.layer_rule_sections_expanded.insert(key, !expanded);
                // Don't mark dirty for UI-only changes
                return Task::none();
            }

            M::ValidateRegex(rule_id, _match_idx, field_name, regex) => {
                // Validate regex pattern
                if regex.is_empty() {
                    self.layer_rule_regex_errors.remove(&(rule_id, field_name));
                } else {
                    match regex_syntax::Parser::new().parse(&regex) {
                        Ok(_) => {
                            self.layer_rule_regex_errors.remove(&(rule_id, field_name));
                        }
                        Err(e) => {
                            self.layer_rule_regex_errors.insert((rule_id, field_name), e.to_string());
                        }
                    }
                }
                // Don't mark dirty for validation-only changes
                return Task::none();
            }
        }

        self.dirty_tracker.mark(SettingsCategory::LayerRules);
        self.mark_changed();
        Task::none()
    }
}
