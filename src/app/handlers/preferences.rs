//! Preferences message handler
//!
//! Handles app preferences like float/tile behavior.

use crate::config::SettingsCategory;
use crate::messages::{Message, PreferencesMessage};
use iced::Task;

/// Validates a hotkey string format.
/// Returns true if the hotkey is valid (empty or properly formatted like "Ctrl+K").
fn is_valid_hotkey(hotkey: &str) -> bool {
    // Empty hotkey is valid (disables the feature)
    if hotkey.is_empty() {
        return true;
    }

    let parts: Vec<&str> = hotkey.split('+').map(|p| p.trim()).collect();
    if parts.is_empty() {
        return false;
    }

    // Must have at least one part (the key)
    let key = parts.last().unwrap();
    if key.is_empty() {
        return false;
    }

    // Valid modifiers
    let valid_modifiers = ["ctrl", "alt", "shift", "mod", "super", "meta"];

    // Check all parts except the last (modifiers)
    for modifier in &parts[..parts.len().saturating_sub(1)] {
        if !valid_modifiers.contains(&modifier.to_lowercase().as_str()) {
            return false;
        }
    }

    // Key must be a single character or a known key name
    let valid_keys = [
        "return", "enter", "tab", "space", "backspace", "delete", "escape",
        "home", "end", "pageup", "pagedown", "up", "down", "left", "right",
        "f1", "f2", "f3", "f4", "f5", "f6", "f7", "f8", "f9", "f10", "f11", "f12",
        "insert", "print", "pause",
    ];

    let key_lower = key.to_lowercase();
    key.len() == 1 || valid_keys.contains(&key_lower.as_str())
}

impl super::super::App {
    pub(in crate::app) fn update_preferences(&mut self, msg: PreferencesMessage) -> Task<Message> {
        match msg {
            PreferencesMessage::SetFloatSettingsApp(float) => {
                self.settings.preferences.float_settings_app = float;

                // Mark preferences as dirty for auto-save
                self.save.dirty_tracker.mark(SettingsCategory::Preferences);
                // Also mark window rules dirty since the float rule is generated there
                self.save.dirty_tracker.mark(SettingsCategory::WindowRules);
                self.mark_changed();

                Task::none()
            }

            PreferencesMessage::SetShowSearchBar(show) => {
                self.settings.preferences.show_search_bar = show;
                self.ui.show_search_bar = show;

                // Mark preferences as dirty for auto-save
                self.save.dirty_tracker.mark(SettingsCategory::Preferences);
                self.mark_changed();

                Task::none()
            }

            PreferencesMessage::SetSearchHotkey(hotkey) => {
                // Validate the hotkey format before saving
                if !is_valid_hotkey(&hotkey) {
                    log::warn!("Invalid search hotkey format: {}", hotkey);
                    return Task::none();
                }

                self.settings.preferences.search_hotkey = hotkey;

                // Mark preferences as dirty for auto-save
                self.save.dirty_tracker.mark(SettingsCategory::Preferences);
                self.mark_changed();

                Task::none()
            }
        }
    }
}
