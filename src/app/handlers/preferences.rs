//! Preferences message handler
//!
//! Handles app preferences like float/tile behavior.

use crate::config::SettingsCategory;
use crate::messages::{Message, PreferencesMessage};
use iced::Task;

impl super::super::App {
    pub(in crate::app) fn update_preferences(&mut self, msg: PreferencesMessage) -> Task<Message> {
        match msg {
            PreferencesMessage::SetFloatSettingsApp(float) => {
                self.settings.preferences.float_settings_app = float;

                // Mark preferences as dirty for auto-save
                self.dirty_tracker.mark(SettingsCategory::Preferences);
                // Also mark window rules dirty since the float rule is generated there
                self.dirty_tracker.mark(SettingsCategory::WindowRules);
                self.mark_changed();

                Task::none()
            }

            PreferencesMessage::SetShowSearchBar(show) => {
                self.settings.preferences.show_search_bar = show;
                self.ui.show_search_bar = show;

                // Mark preferences as dirty for auto-save
                self.dirty_tracker.mark(SettingsCategory::Preferences);
                self.mark_changed();

                Task::none()
            }

            PreferencesMessage::SetSearchHotkey(hotkey) => {
                self.settings.preferences.search_hotkey = hotkey;

                // Mark preferences as dirty for auto-save
                self.dirty_tracker.mark(SettingsCategory::Preferences);
                self.mark_changed();

                Task::none()
            }
        }
    }
}
