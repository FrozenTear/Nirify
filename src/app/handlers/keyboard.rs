//! Keyboard settings message handler

use crate::config::SettingsCategory;
use crate::messages::{KeyboardMessage, Message};
use iced::Task;

impl super::super::App {
    /// Updates keyboard settings
    pub(in crate::app) fn update_keyboard(&mut self, msg: KeyboardMessage) -> Task<Message> {
        

        match msg {
            KeyboardMessage::SetXkbLayout(value) => {
                self.settings.keyboard.xkb_layout = value;
            }
            KeyboardMessage::SetXkbVariant(value) => {
                self.settings.keyboard.xkb_variant = value;
            }
            KeyboardMessage::SetXkbOptions(value) => {
                self.settings.keyboard.xkb_options = value;
            }
            KeyboardMessage::SetXkbModel(value) => {
                self.settings.keyboard.xkb_model = value;
            }
            KeyboardMessage::SetRepeatDelay(value) => {
                self.settings.keyboard.repeat_delay = value.clamp(100, 2000);
            }
            KeyboardMessage::SetRepeatRate(value) => {
                self.settings.keyboard.repeat_rate = value.clamp(1, 100);
            }
            KeyboardMessage::SetTrackLayout(value) => {
                self.settings.keyboard.track_layout = value;
            }
            KeyboardMessage::SetNumlock(value) => {
                self.settings.keyboard.numlock = value;
            }
        }


        self.save.dirty_tracker.mark(SettingsCategory::Keyboard);
        self.mark_changed();

        Task::none()
    }
}
