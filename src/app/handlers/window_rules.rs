//! Window rules settings message handler

use crate::config::SettingsCategory;
use crate::config::models::{WindowRule, WindowRuleMatch};
use crate::messages::{WindowRulesMessage as M, Message};
use iced::Task;

impl super::super::App {
    /// Updates window rules settings
    pub(in crate::app) fn update_window_rules(&mut self, msg: M) -> Task<Message> {
        let mut settings = self.settings.lock().expect("settings mutex poisoned");
        let mut should_mark_dirty = true;

        match msg {
            M::AddRule => {
                let new_id = settings.window_rules.next_id;
                settings.window_rules.next_id += 1;
                let new_rule = WindowRule {
                    id: new_id,
                    name: format!("Rule {}", new_id + 1),
                    ..Default::default()
                };
                settings.window_rules.rules.push(new_rule);
                self.selected_window_rule_id = Some(new_id);
            }

            M::DeleteRule(id) => {
                settings.window_rules.rules.retain(|r| r.id != id);
                if self.selected_window_rule_id == Some(id) {
                    self.selected_window_rule_id = settings.window_rules.rules.first().map(|r| r.id);
                }
            }

            M::SelectRule(id) => {
                self.selected_window_rule_id = Some(id);
                should_mark_dirty = false;
            }

            M::DuplicateRule(id) => {
                if let Some(rule) = settings.window_rules.rules.iter().find(|r| r.id == id).cloned() {
                    let new_id = settings.window_rules.next_id;
                    settings.window_rules.next_id += 1;
                    let mut new_rule = rule;
                    new_rule.id = new_id;
                    new_rule.name = format!("{} (copy)", new_rule.name);
                    settings.window_rules.rules.push(new_rule);
                    self.selected_window_rule_id = Some(new_id);
                }
            }

            M::SetRuleName(id, name) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.name = name;
                }
            }

            M::AddMatch(id) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.matches.push(WindowRuleMatch::default());
                }
            }

            M::RemoveMatch(id, match_idx) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    if match_idx < rule.matches.len() && rule.matches.len() > 1 {
                        rule.matches.remove(match_idx);
                    }
                }
            }

            M::SetMatchAppId(id, match_idx, value) => {
                // Validate regex syntax using regex-syntax crate
                let error_key = (id, format!("app_id_{}", match_idx));
                if let Some(ref regex_str) = value {
                    if !regex_str.is_empty() {
                        if let Err(e) = regex_syntax::Parser::new().parse(regex_str) {
                            self.window_rule_regex_errors.insert(error_key.clone(), format!("Invalid regex: {}", e));
                        } else {
                            self.window_rule_regex_errors.remove(&error_key);
                        }
                    } else {
                        self.window_rule_regex_errors.remove(&error_key);
                    }
                } else {
                    self.window_rule_regex_errors.remove(&error_key);
                }

                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    if let Some(m) = rule.matches.get_mut(match_idx) {
                        m.app_id = value;
                    }
                }
            }

            M::SetMatchTitle(id, match_idx, value) => {
                // Validate regex syntax using regex-syntax crate
                let error_key = (id, format!("title_{}", match_idx));
                if let Some(ref regex_str) = value {
                    if !regex_str.is_empty() {
                        if let Err(e) = regex_syntax::Parser::new().parse(regex_str) {
                            self.window_rule_regex_errors.insert(error_key.clone(), format!("Invalid regex: {}", e));
                        } else {
                            self.window_rule_regex_errors.remove(&error_key);
                        }
                    } else {
                        self.window_rule_regex_errors.remove(&error_key);
                    }
                } else {
                    self.window_rule_regex_errors.remove(&error_key);
                }

                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    if let Some(m) = rule.matches.get_mut(match_idx) {
                        m.title = value;
                    }
                }
            }

            M::SetMatchIsFloating(id, match_idx, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    if let Some(m) = rule.matches.get_mut(match_idx) {
                        m.is_floating = value;
                    }
                }
            }

            M::SetMatchIsFocused(id, match_idx, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    if let Some(m) = rule.matches.get_mut(match_idx) {
                        m.is_focused = value;
                    }
                }
            }

            M::SetOpenBehavior(id, behavior) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.open_behavior = behavior;
                }
            }

            M::SetOpenFocused(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.open_focused = value;
                }
            }

            M::SetOpenOnOutput(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.open_on_output = value;
                }
            }

            M::SetOpenOnWorkspace(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.open_on_workspace = value;
                }
            }

            M::SetBlockScreencast(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.block_out_from_screencast = value;
                }
            }

            M::SetDefaultColumnWidth(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.default_column_width = value;
                }
            }

            M::SetDefaultWindowHeight(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.default_window_height = value;
                }
            }

            M::SetMinWidth(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.min_width = value;
                }
            }

            M::SetMaxWidth(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.max_width = value;
                }
            }

            M::SetMinHeight(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.min_height = value;
                }
            }

            M::SetMaxHeight(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.max_height = value;
                }
            }

            M::SetOpacity(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.opacity = value;
                }
            }

            M::SetCornerRadius(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.corner_radius = value;
                }
            }

            M::SetClipToGeometry(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.clip_to_geometry = value;
                }
            }

            M::SetDrawBorderWithBackground(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.draw_border_with_background = value;
                }
            }

            M::SetVariableRefreshRate(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.variable_refresh_rate = value;
                }
            }

            M::SetBabaIsFloat(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.baba_is_float = value;
                }
            }

            M::SetTiledState(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
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

        // Update cache
        self.window_rules_cache = settings.window_rules.clone();

        drop(settings);

        if should_mark_dirty {
            self.dirty_tracker.mark(SettingsCategory::WindowRules);
            self.save_manager.mark_changed();
        }

        Task::none()
    }
}
