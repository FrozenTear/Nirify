//! Cursor settings message handler

use crate::config::SettingsCategory;
use crate::messages::{CursorMessage, Message};
use iced::Task;

impl super::super::App {
    /// Updates cursor settings
    pub(in crate::app) fn update_cursor(&mut self, msg: CursorMessage) -> Task<Message> {
        let mut settings = self.settings.lock().expect("settings mutex poisoned");

        match msg {
            CursorMessage::SetTheme(value) => {
                settings.cursor.theme = value;
            }
            CursorMessage::SetSize(value) => {
                settings.cursor.size = value.clamp(16, 48);
            }
            CursorMessage::ToggleHideWhenTyping(value) => {
                settings.cursor.hide_when_typing = value;
            }
            CursorMessage::SetHideAfterInactive(value) => {
                settings.cursor.hide_after_inactive_ms = value;
            }
        }

        // Update cache for view borrowing
        self.cursor_cache = settings.cursor.clone();

        drop(settings);

        self.dirty_tracker.mark(SettingsCategory::Cursor);
        self.save_manager.mark_changed();

        Task::none()
    }
}
