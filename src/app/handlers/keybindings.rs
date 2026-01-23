//! Keybindings settings message handler

use crate::app::helpers::{parse_spawn_command, validate_spawn_command};
use crate::config::SettingsCategory;
use crate::config::models::KeybindAction;
use crate::messages::{KeybindingsMessage as M, Message};
use crate::types::ModKey;
use iced::Task;

/// Known modifier prefixes in niri keybindings
const MODIFIER_PREFIXES: &[&str] = &["Mod", "Super", "Ctrl", "Control", "Shift", "Alt", "Mod3", "Mod5"];

/// Extract the base key from a key combo string (e.g., "Mod+Shift+Return" -> "Return")
fn extract_base_key(key_combo: &str) -> String {
    if key_combo.is_empty() {
        return String::new();
    }

    // Split by + and find the last non-modifier part
    let parts: Vec<&str> = key_combo.split('+').collect();

    // Work backwards to find the first non-modifier
    for part in parts.iter().rev() {
        let trimmed = part.trim();
        if !MODIFIER_PREFIXES.iter().any(|m| m.eq_ignore_ascii_case(trimmed)) {
            return trimmed.to_string();
        }
    }

    // If all parts are modifiers, return empty (unusual but possible)
    String::new()
}

/// Build a key combo string from modifiers and a base key
fn build_key_combo(modifiers: &[ModKey], base_key: &str) -> String {
    if base_key.is_empty() && modifiers.is_empty() {
        return String::new();
    }

    let mut parts = Vec::new();

    // Add modifiers in a consistent order
    for modifier in modifiers {
        let mod_str = match modifier {
            ModKey::Super => "Mod",  // niri uses "Mod" for Super
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

    // Add the base key
    if !base_key.is_empty() {
        parts.push(base_key);
    }

    parts.join("+")
}

impl super::super::App {
    /// Updates keybindings settings
    pub(in crate::app) fn update_keybindings(&mut self, msg: M) -> Task<Message> {


        match msg {
            M::AddKeybinding => {
                let new_binding = crate::config::models::Keybinding {
                    id: self.settings.keybindings.bindings.len() as u32,
                    key_combo: String::new(),
                    action: KeybindAction::NiriAction("close-window".to_string()),
                    ..Default::default()
                };
                self.settings.keybindings.bindings.push(new_binding);
                self.ui.selected_keybinding_index = Some(self.settings.keybindings.bindings.len() - 1);
                log::info!("Added new keybinding");
            }

            M::RemoveKeybinding(idx) => {
                if idx < self.settings.keybindings.bindings.len() {
                    self.settings.keybindings.bindings.remove(idx);
                    if self.ui.selected_keybinding_index == Some(idx) {
                        self.ui.selected_keybinding_index = if self.settings.keybindings.bindings.is_empty() {
                            None
                        } else {
                            Some(0)
                        };
                    }
                    log::info!("Removed keybinding at index {}", idx);
                }
            }

            M::SelectKeybinding(idx) => {
                self.ui.selected_keybinding_index = Some(idx);
                // Don't mark dirty for UI-only changes
                return Task::none();
            }

            M::UpdateModifiers(idx, modifiers) => {
                if let Some(binding) = self.settings.keybindings.bindings.get_mut(idx) {
                    // Parse existing key_combo to extract the base key
                    let base_key = extract_base_key(&binding.key_combo);

                    // Build new key_combo with the provided modifiers
                    let new_combo = build_key_combo(&modifiers, &base_key);
                    binding.key_combo = new_combo;
                    log::info!("Updated modifiers for binding {}: {:?}", idx, modifiers);
                }
            }

            M::StartKeyCapture(idx) => {
                self.ui.key_capture_active = Some(idx);
                // Don't mark dirty for UI-only changes
                return Task::none();
            }

            M::CapturedKey(key_combo) => {
                if let Some(idx) = self.ui.key_capture_active {
                    if let Some(binding) = self.settings.keybindings.bindings.get_mut(idx) {
                        binding.key_combo = key_combo;
                        log::info!("Captured key combo for binding {}", idx);
                    }
                }
                self.ui.key_capture_active = None;
            }

            M::CancelKeyCapture => {
                self.ui.key_capture_active = None;
                // Don't mark dirty for UI-only changes
                return Task::none();
            }

            M::UpdateAction(idx, action_str) => {
                if let Some(binding) = self.settings.keybindings.bindings.get_mut(idx) {
                    if action_str == "spawn" || action_str.starts_with("spawn ") {
                        let command_part = if action_str == "spawn" {
                            ""
                        } else {
                            action_str.strip_prefix("spawn ").unwrap_or("")
                        };

                        // Use proper command parsing with quote handling
                        match validate_spawn_command(command_part) {
                            Ok(args) => {
                                binding.action = KeybindAction::Spawn(args);
                                log::info!("Updated action for binding {}", idx);
                            }
                            Err(e) => {
                                log::warn!("Invalid spawn command for binding {}: {}", idx, e);
                                // Still set it but with empty args - user can fix it
                                binding.action = KeybindAction::Spawn(Vec::new());
                            }
                        }
                    } else {
                        binding.action = KeybindAction::NiriAction(action_str);
                        log::info!("Updated action for binding {}", idx);
                    }
                }
            }

            M::SetCommand(idx, command) => {
                if let Some(binding) = self.settings.keybindings.bindings.get_mut(idx) {
                    // Use proper command parsing with quote handling and validation
                    match parse_spawn_command(&command) {
                        Ok(parsed) => {
                            if let Some(warning) = &parsed.warning {
                                log::warn!("Keybinding {}: {}", idx, warning);
                                // Could show toast warning to user here
                            }
                            binding.action = KeybindAction::Spawn(parsed.args);
                            log::info!("Updated command for binding {}", idx);
                        }
                        Err(e) => {
                            log::error!("Failed to parse command for binding {}: {}", idx, e);
                            // Don't update if parsing fails completely
                            return Task::none();
                        }
                    }
                }
            }

            M::SetAllowWhenLocked(idx, value) => {
                if let Some(binding) = self.settings.keybindings.bindings.get_mut(idx) {
                    binding.allow_when_locked = value;
                    log::info!("Set allow_when_locked={} for binding {}", value, idx);
                }
            }

            M::SetRepeat(idx, value) => {
                if let Some(binding) = self.settings.keybindings.bindings.get_mut(idx) {
                    binding.repeat = value;
                    log::info!("Set repeat={} for binding {}", value, idx);
                }
            }

            M::SetCooldown(idx, cooldown) => {
                if let Some(binding) = self.settings.keybindings.bindings.get_mut(idx) {
                    binding.cooldown_ms = cooldown;
                    log::info!("Set cooldown={:?} for binding {}", cooldown, idx);
                }
            }

            M::SetHotkeyOverlayTitle(idx, title) => {
                if let Some(binding) = self.settings.keybindings.bindings.get_mut(idx) {
                    binding.hotkey_overlay_title = title;
                    log::info!("Set hotkey_overlay_title for binding {}", idx);
                }
            }

            M::ToggleSection(section) => {
                let expanded = self.ui.keybinding_sections_expanded.get(&section).copied().unwrap_or(false);
                self.ui.keybinding_sections_expanded.insert(section, !expanded);
                // Don't mark dirty for UI-only changes
                return Task::none();
            }
        }

        // Update the cache for view borrowing

        self.dirty_tracker.mark(SettingsCategory::Keybindings);
        self.mark_changed();

        Task::none()
    }
}
