//! Keybindings settings message handler

use crate::app::helpers::parse_spawn_command;
use crate::config::models::{
    actions_in_category, lookup_action, normalized_key_combo, ActionCategory, ActionNode,
    ActionValue, ArgKind, KeybindAction,
};
use crate::config::SettingsCategory;
use crate::messages::{KeybindingsMessage as M, Message};
use crate::types::ModKey;
use iced::Task;

/// Known modifier prefixes in niri keybindings
const MODIFIER_PREFIXES: &[&str] = &[
    "Mod", "Super", "Ctrl", "Control", "Shift", "Alt", "Mod3", "Mod5",
];

/// Extract the base key from a key combo string (e.g., "Mod+Shift+Return" -> "Return")
fn extract_base_key(key_combo: &str) -> String {
    if key_combo.is_empty() {
        return String::new();
    }
    let parts: Vec<&str> = key_combo.split('+').collect();
    for part in parts.iter().rev() {
        let trimmed = part.trim();
        if !MODIFIER_PREFIXES
            .iter()
            .any(|m| m.eq_ignore_ascii_case(trimmed))
        {
            return trimmed.to_string();
        }
    }
    String::new()
}

/// Build a key combo string from modifiers and a base key
fn build_key_combo(modifiers: &[ModKey], base_key: &str) -> String {
    if base_key.is_empty() && modifiers.is_empty() {
        return String::new();
    }
    let mut parts = Vec::new();
    for modifier in modifiers {
        let mod_str = match modifier {
            ModKey::Super => "Mod",
            ModKey::Ctrl => "Ctrl",
            ModKey::Shift => "Shift",
            ModKey::Alt => "Alt",
            ModKey::Mod3 => "Mod3",
            ModKey::Mod5 => "Mod5",
        };
        if !parts.contains(&mod_str) {
            parts.push(mod_str);
        }
    }
    if !base_key.is_empty() {
        parts.push(base_key);
    }
    parts.join("+")
}

/// Build the default action value for a catalog action name.
fn default_action_for(name: &str) -> KeybindAction {
    match name {
        "spawn" => KeybindAction::Spawn(Vec::new()),
        "spawn-sh" => KeybindAction::SpawnSh(String::new()),
        _ => {
            let mut node = ActionNode::bare(name);
            // Give pick-list arguments a valid default so the bind saves.
            match lookup_action(name).map(|s| s.args) {
                Some(ArgKind::ColumnDisplay) => {
                    node.set_primary_arg(Some(ActionValue::Str("normal".to_string())));
                }
                Some(ArgKind::LayoutTarget) => {
                    node.set_primary_arg(Some(ActionValue::Str("next".to_string())));
                }
                _ => {}
            }
            KeybindAction::NiriAction(node)
        }
    }
}

impl super::super::App {
    /// Updates keybindings settings
    pub(in crate::app) fn update_keybindings(&mut self, msg: M) -> Task<Message> {
        match msg {
            M::AddKeybinding => {
                // Assign a fresh id (max+1) to avoid collisions after removals.
                let next_id = self
                    .settings
                    .keybindings
                    .bindings
                    .iter()
                    .map(|b| b.id)
                    .max()
                    .map_or(0, |m| m + 1);
                let new_binding = crate::config::models::Keybinding {
                    id: next_id,
                    ..Default::default()
                };
                self.settings.keybindings.bindings.push(new_binding);
                let new_idx = self.settings.keybindings.bindings.len() - 1;
                self.ui.selected_keybinding_index = Some(new_idx);
                self.ui.editing_keybinding_index = Some(new_idx);
                log::info!("Added new keybinding");
            }

            M::RemoveKeybinding(idx) => {
                if idx < self.settings.keybindings.bindings.len() {
                    self.settings.keybindings.bindings.remove(idx);
                    let len = self.settings.keybindings.bindings.len();

                    // Fix up stale selection / editing / capture indices.
                    self.ui.selected_keybinding_index = match self.ui.selected_keybinding_index {
                        Some(sel) if sel == idx => {
                            if len == 0 {
                                None
                            } else {
                                Some(idx.min(len - 1))
                            }
                        }
                        Some(sel) if sel > idx => Some(sel - 1),
                        other => other,
                    };
                    self.ui.editing_keybinding_index = match self.ui.editing_keybinding_index {
                        Some(e) if e == idx => None,
                        Some(e) if e > idx => Some(e - 1),
                        other => other,
                    };
                    self.ui.key_capture_active = match self.ui.key_capture_active {
                        Some(c) if c == idx => None,
                        Some(c) if c > idx => Some(c - 1),
                        other => other,
                    };
                    log::info!("Removed keybinding at index {}", idx);
                }
            }

            M::SelectKeybinding(idx) => {
                self.ui.selected_keybinding_index = Some(idx);
                return Task::none();
            }

            M::UpdateModifiers(idx, modifiers) => {
                if let Some(binding) = self.settings.keybindings.bindings.get_mut(idx) {
                    let base_key = extract_base_key(&binding.key_combo);
                    binding.key_combo = build_key_combo(&modifiers, &base_key);
                    log::info!("Updated modifiers for binding {}: {:?}", idx, modifiers);
                }
            }

            M::StartKeyCapture(idx) => {
                self.ui.key_capture_active = Some(idx);
                self.ui.keybinding_capture_conflict = None;
                return Task::none();
            }

            M::CapturedKey(key_combo) => {
                if let Some(idx) = self.ui.key_capture_active {
                    // niri rejects duplicate keys; block assignment on conflict.
                    let norm = normalized_key_combo(&key_combo);
                    let conflict = self
                        .settings
                        .keybindings
                        .bindings
                        .iter()
                        .enumerate()
                        .find(|(i, b)| *i != idx && normalized_key_combo(&b.key_combo) == norm)
                        .map(|(_, b)| b.display_name());

                    if let Some(name) = conflict {
                        self.ui.keybinding_capture_conflict = Some((idx, key_combo.clone(), name));
                        // Keep capture active; do not assign or mark dirty.
                        return Task::none();
                    }

                    if let Some(binding) = self.settings.keybindings.bindings.get_mut(idx) {
                        binding.key_combo = key_combo;
                        log::info!("Captured key combo for binding {}", idx);
                    }
                }
                self.ui.key_capture_active = None;
                self.ui.keybinding_capture_conflict = None;
            }

            M::CancelKeyCapture => {
                self.ui.key_capture_active = None;
                self.ui.keybinding_capture_conflict = None;
                return Task::none();
            }

            M::SelectActionCategory(idx, category) => {
                if let Some(binding) = self.settings.keybindings.bindings.get_mut(idx) {
                    let new_action = if category == ActionCategory::Custom {
                        KeybindAction::Custom(String::new())
                    } else if let Some(spec) = actions_in_category(category).first() {
                        default_action_for(spec.name)
                    } else {
                        binding.action.clone()
                    };
                    // Leaving spawn resets the locked-only flag.
                    if !matches!(
                        new_action,
                        KeybindAction::Spawn(_) | KeybindAction::SpawnSh(_)
                    ) {
                        binding.allow_when_locked = false;
                    }
                    binding.action = new_action;
                    log::info!("Selected action category for binding {}", idx);
                }
            }

            M::UpdateAction(idx, action_str) => {
                if let Some(binding) = self.settings.keybindings.bindings.get_mut(idx) {
                    let new_action = default_action_for(&action_str);
                    if !matches!(
                        new_action,
                        KeybindAction::Spawn(_) | KeybindAction::SpawnSh(_)
                    ) {
                        binding.allow_when_locked = false;
                    }
                    binding.action = new_action;
                    log::info!("Updated action for binding {}", idx);
                }
            }

            M::SetCommand(idx, command) => {
                if let Some(binding) = self.settings.keybindings.bindings.get_mut(idx) {
                    match parse_spawn_command(&command) {
                        Ok(parsed) => {
                            if let Some(warning) = &parsed.warning {
                                log::warn!("Keybinding {}: {}", idx, warning);
                            }
                            binding.action = KeybindAction::Spawn(parsed.args);
                        }
                        Err(e) => {
                            log::error!("Failed to parse command for binding {}: {}", idx, e);
                            return Task::none();
                        }
                    }
                }
            }

            M::SetSpawnShCommand(idx, command) => {
                if let Some(binding) = self.settings.keybindings.bindings.get_mut(idx) {
                    binding.action = KeybindAction::SpawnSh(command);
                }
            }

            M::SetCustomActionText(idx, text) => {
                if let Some(binding) = self.settings.keybindings.bindings.get_mut(idx) {
                    binding.action = KeybindAction::Custom(text);
                }
            }

            M::SetActionArgText(idx, value) => {
                if let Some(binding) = self.settings.keybindings.bindings.get_mut(idx) {
                    let arg_kind = match &binding.action {
                        KeybindAction::NiriAction(node) => {
                            lookup_action(&node.name).map(|s| s.args)
                        }
                        _ => None,
                    };
                    if let KeybindAction::NiriAction(node) = &mut binding.action {
                        let trimmed = value.trim();
                        match arg_kind {
                            Some(ArgKind::IndexInt) => {
                                if trimmed.is_empty() {
                                    node.set_primary_arg(None);
                                } else if let Ok(n) = trimmed.parse::<i64>() {
                                    // niri decodes IndexInt as usize/NonZero, so
                                    // reject values < 1; keep the last valid value.
                                    if n >= 1 {
                                        node.set_primary_arg(Some(ActionValue::Int(n)));
                                    } else {
                                        return Task::none();
                                    }
                                } else {
                                    // Ignore non-numeric input; keep last valid value.
                                    return Task::none();
                                }
                            }
                            Some(ArgKind::WorkspaceRef) | Some(ArgKind::WorkspaceRefFocus) => {
                                if trimmed.is_empty() {
                                    node.set_primary_arg(None);
                                } else if trimmed.chars().all(|c| c.is_ascii_digit()) {
                                    if let Ok(n) = trimmed.parse::<i64>() {
                                        node.set_primary_arg(Some(ActionValue::Int(n)));
                                    }
                                } else {
                                    node.set_primary_arg(Some(ActionValue::Str(value.clone())));
                                }
                            }
                            _ => {
                                if value.is_empty() {
                                    node.set_primary_arg(None);
                                } else {
                                    node.set_primary_arg(Some(ActionValue::Str(value.clone())));
                                }
                            }
                        }
                    }
                }
            }

            M::SetActionFocusFlag(idx, value) => {
                set_action_prop(self, idx, "focus", Some(ActionValue::Bool(value)));
            }

            M::SetActionSkipConfirmation(idx, value) => {
                set_action_prop(
                    self,
                    idx,
                    "skip-confirmation",
                    Some(ActionValue::Bool(value)),
                );
            }

            M::SetActionDelayMs(idx, value) => {
                set_action_prop(
                    self,
                    idx,
                    "delay-ms",
                    value.map(|v| ActionValue::Int(v as i64)),
                );
            }

            M::SetActionWriteToDisk(idx, value) => {
                set_action_prop(self, idx, "write-to-disk", Some(ActionValue::Bool(value)));
            }

            M::SetActionShowPointer(idx, value) => {
                set_action_prop(self, idx, "show-pointer", Some(ActionValue::Bool(value)));
            }

            M::SetAllowWhenLocked(idx, value) => {
                if let Some(binding) = self.settings.keybindings.bindings.get_mut(idx) {
                    binding.allow_when_locked = value;
                }
            }

            M::SetAllowInhibiting(idx, value) => {
                if let Some(binding) = self.settings.keybindings.bindings.get_mut(idx) {
                    binding.allow_inhibiting = value;
                }
            }

            M::SetRepeat(idx, value) => {
                if let Some(binding) = self.settings.keybindings.bindings.get_mut(idx) {
                    binding.repeat = value;
                }
            }

            M::SetCooldown(idx, cooldown) => {
                if let Some(binding) = self.settings.keybindings.bindings.get_mut(idx) {
                    // niri decodes cooldown-ms as u64; never store a negative value.
                    binding.cooldown_ms = cooldown.map(|c| c.max(0));
                }
            }

            M::SetHotkeyOverlayTitle(idx, title) => {
                if let Some(binding) = self.settings.keybindings.bindings.get_mut(idx) {
                    binding.hotkey_overlay_title = title;
                }
            }

            M::ToggleSection(section) => {
                let expanded = self
                    .ui
                    .keybinding_sections_expanded
                    .get(&section)
                    .copied()
                    .unwrap_or(false);
                self.ui
                    .keybinding_sections_expanded
                    .insert(section, !expanded);
                return Task::none();
            }
        }

        self.save.dirty_tracker.mark(SettingsCategory::Keybindings);
        self.mark_changed();

        Task::none()
    }
}

/// Set or remove a named property on a NiriAction binding.
fn set_action_prop(
    app: &mut super::super::App,
    idx: usize,
    name: &str,
    value: Option<ActionValue>,
) {
    if let Some(binding) = app.settings.keybindings.bindings.get_mut(idx) {
        if let KeybindAction::NiriAction(node) = &mut binding.action {
            node.set_prop(name, value);
        }
    }
}
