//! Window rules settings message handler

use crate::config::SettingsCategory;
use crate::config::models::{WindowRule, WindowRuleMatch};
use crate::messages::{WindowRulesMessage as M, Message};
use iced::Task;

impl super::super::App {
    /// Updates window rules settings
    pub(in crate::app) fn update_window_rules(&mut self, msg: M) -> Task<Message> {
        let mut should_mark_dirty = true;

        match msg {
            M::AddRule => {
                let new_id = self.settings.window_rules.next_id;
                self.settings.window_rules.next_id += 1;
                let new_rule = WindowRule {
                    id: new_id,
                    name: format!("Rule {}", new_id + 1),
                    ..Default::default()
                };
                self.settings.window_rules.rules.push(new_rule);
                self.selected_window_rule_id = Some(new_id);
            }

            M::DeleteRule(id) => {
                self.settings.window_rules.remove(id);
                if self.selected_window_rule_id == Some(id) {
                    self.selected_window_rule_id = self.settings.window_rules.rules.first().map(|r| r.id);
                }
            }

            M::SelectRule(id) => {
                self.selected_window_rule_id = Some(id);
                should_mark_dirty = false;
            }

            M::DuplicateRule(id) => {
                if let Some(rule) = self.settings.window_rules.find(id).cloned() {
                    let new_id = self.settings.window_rules.next_id;
                    self.settings.window_rules.next_id += 1;
                    let mut new_rule = rule;
                    new_rule.id = new_id;
                    new_rule.name = format!("{} (copy)", new_rule.name);
                    self.settings.window_rules.rules.push(new_rule);
                    self.selected_window_rule_id = Some(new_id);
                }
            }

            M::SetRuleName(id, name) => {
                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    rule.name = name;
                }
            }

            M::AddMatch(id) => {
                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    rule.matches.push(WindowRuleMatch::default());
                }
            }

            M::RemoveMatch(id, match_idx) => {
                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    if match_idx < rule.matches.len() && rule.matches.len() > 1 {
                        rule.matches.remove(match_idx);
                    }
                }
            }

            M::SetMatchAppId(id, match_idx, value) => {
                // Validate regex syntax
                let error_key = (id, format!("app_id_{}", match_idx));
                self.validate_regex(&error_key, value.as_deref());

                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    if let Some(m) = rule.matches.get_mut(match_idx) {
                        m.app_id = value;
                    }
                }
            }

            M::SetMatchTitle(id, match_idx, value) => {
                // Validate regex syntax
                let error_key = (id, format!("title_{}", match_idx));
                self.validate_regex(&error_key, value.as_deref());

                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    if let Some(m) = rule.matches.get_mut(match_idx) {
                        m.title = value;
                    }
                }
            }

            M::SetMatchIsFloating(id, match_idx, value) => {
                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    if let Some(m) = rule.matches.get_mut(match_idx) {
                        m.is_floating = value;
                    }
                }
            }

            M::SetMatchIsFocused(id, match_idx, value) => {
                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    if let Some(m) = rule.matches.get_mut(match_idx) {
                        m.is_focused = value;
                    }
                }
            }

            M::SetMatchIsActive(id, match_idx, value) => {
                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    if let Some(m) = rule.matches.get_mut(match_idx) {
                        m.is_active = value;
                    }
                }
            }

            M::SetMatchIsActiveInColumn(id, match_idx, value) => {
                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    if let Some(m) = rule.matches.get_mut(match_idx) {
                        m.is_active_in_column = value;
                    }
                }
            }

            M::SetMatchIsWindowCastTarget(id, match_idx, value) => {
                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    if let Some(m) = rule.matches.get_mut(match_idx) {
                        m.is_window_cast_target = value;
                    }
                }
            }

            M::SetMatchIsUrgent(id, match_idx, value) => {
                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    if let Some(m) = rule.matches.get_mut(match_idx) {
                        m.is_urgent = value;
                    }
                }
            }

            M::SetMatchAtStartup(id, match_idx, value) => {
                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    if let Some(m) = rule.matches.get_mut(match_idx) {
                        m.at_startup = value;
                    }
                }
            }

            M::SetOpenBehavior(id, behavior) => {
                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    rule.open_behavior = behavior;
                }
            }

            M::SetOpenFocused(id, value) => {
                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    rule.open_focused = value;
                }
            }

            M::SetOpenOnOutput(id, value) => {
                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    rule.open_on_output = value;
                }
            }

            M::SetOpenOnWorkspace(id, value) => {
                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    rule.open_on_workspace = value;
                }
            }

            M::SetBlockScreencast(id, value) => {
                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    rule.block_out_from_screencast = value;
                }
            }

            M::SetDefaultColumnWidth(id, value) => {
                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    rule.default_column_width = value;
                }
            }

            M::SetDefaultWindowHeight(id, value) => {
                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    rule.default_window_height = value;
                }
            }

            M::SetMinWidth(id, value) => {
                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    rule.min_width = value;
                }
            }

            M::SetMaxWidth(id, value) => {
                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    rule.max_width = value;
                }
            }

            M::SetMinHeight(id, value) => {
                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    rule.min_height = value;
                }
            }

            M::SetMaxHeight(id, value) => {
                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    rule.max_height = value;
                }
            }

            M::SetOpacity(id, value) => {
                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    rule.opacity = value;
                }
            }

            M::SetCornerRadius(id, value) => {
                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    rule.corner_radius = value;
                }
            }

            M::SetClipToGeometry(id, value) => {
                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    rule.clip_to_geometry = value;
                }
            }

            M::SetDrawBorderWithBackground(id, value) => {
                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    rule.draw_border_with_background = value;
                }
            }

            M::SetVariableRefreshRate(id, value) => {
                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    rule.variable_refresh_rate = value;
                }
            }

            M::SetBabaIsFloat(id, value) => {
                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    rule.baba_is_float = value;
                }
            }

            M::SetTiledState(id, value) => {
                if let Some(rule) = self.settings.window_rules.find_mut(id) {
                    rule.tiled_state = value;
                }
            }

            M::ToggleSection(id, section) => {
                let key = (id, section);
                let current = self.window_rule_sections_expanded.get(&key).copied().unwrap_or(false);
                self.window_rule_sections_expanded.insert(key, !current);
                should_mark_dirty = false;
            }
        }

        if should_mark_dirty {
            self.dirty_tracker.mark(SettingsCategory::WindowRules);
            self.mark_changed();
        }

        Task::none()
    }

    /// Helper to validate regex and update error cache
    fn validate_regex(&mut self, error_key: &(u32, String), regex_str: Option<&str>) {
        match regex_str {
            Some(s) if !s.is_empty() => {
                if let Err(e) = regex_syntax::Parser::new().parse(s) {
                    self.window_rule_regex_errors.insert(error_key.clone(), format!("Invalid regex: {}", e));
                } else {
                    self.window_rule_regex_errors.remove(error_key);
                }
            }
            _ => {
                self.window_rule_regex_errors.remove(error_key);
            }
        }
    }
}
