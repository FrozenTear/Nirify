//! Keyboard settings message handler

use crate::config::SettingsCategory;
use crate::messages::{KeyboardMessage, Message};
use iced::Task;

impl super::super::App {
    /// Updates keyboard settings
    pub(in crate::app) fn update_keyboard(&mut self, msg: KeyboardMessage) -> Task<Message> {
        let mut settings = self.settings.lock().expect("settings mutex poisoned");

        match msg {
            KeyboardMessage::SetXkbLayout(value) => {
                settings.keyboard.xkb_layout = value;
            }
            KeyboardMessage::SetXkbVariant(value) => {
                settings.keyboard.xkb_variant = value;
            }
            KeyboardMessage::SetXkbOptions(value) => {
                settings.keyboard.xkb_options = value;
            }
            KeyboardMessage::SetXkbModel(value) => {
                settings.keyboard.xkb_model = value;
            }
            KeyboardMessage::SetRepeatDelay(value) => {
                settings.keyboard.repeat_delay = value.clamp(100, 2000);
            }
            KeyboardMessage::SetRepeatRate(value) => {
                settings.keyboard.repeat_rate = value.clamp(1, 100);
            }
            KeyboardMessage::SetTrackLayout(value) => {
                settings.keyboard.track_layout = value;
            }
        }

        drop(settings);

        self.dirty_tracker.mark(SettingsCategory::Keyboard);
        self.save_manager.mark_changed();

        Task::none()
    }
}
