//! Cursor settings message handler

use crate::config::SettingsCategory;
use crate::messages::{CursorMessage, Message};
use iced::Task;

impl super::super::App {
    /// Updates cursor settings
    pub(in crate::app) fn update_cursor(&mut self, msg: CursorMessage) -> Task<Message> {
        

        match msg {
            CursorMessage::SetTheme(value) => {
                self.settings.cursor.theme = value;
            }
            CursorMessage::SetSize(value) => {
                self.settings.cursor.size = value.clamp(16, 48);
            }
            CursorMessage::ToggleHideWhenTyping(value) => {
                self.settings.cursor.hide_when_typing = value;
            }
            CursorMessage::SetHideAfterInactive(value) => {
                self.settings.cursor.hide_after_inactive_ms = value;
            }
        }

        // Update cache for view borrowing


        self.save.dirty_tracker.mark(SettingsCategory::Cursor);
        self.mark_changed();

        Task::none()
    }
}
