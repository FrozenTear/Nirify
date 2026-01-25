//! Mouse settings message handler

use crate::config::SettingsCategory;
use crate::messages::{MouseMessage, Message};
use iced::Task;

impl super::super::App {
    /// Updates mouse settings
    pub(in crate::app) fn update_mouse(&mut self, msg: MouseMessage) -> Task<Message> {
        

        match msg {
            MouseMessage::ToggleOffOnTouchpad(value) => {
                self.settings.mouse.off = value;
            }
            MouseMessage::ToggleNaturalScroll(value) => {
                self.settings.mouse.natural_scroll = value;
            }
            MouseMessage::SetAccelSpeed(value) => {
                self.settings.mouse.accel_speed = value.clamp(-1.0, 1.0) as f64;
            }
            MouseMessage::SetAccelProfile(profile) => {
                self.settings.mouse.accel_profile = profile;
            }
            MouseMessage::SetScrollFactor(value) => {
                self.settings.mouse.scroll_factor = value.clamp(0.1, 10.0) as f64;
            }
            MouseMessage::SetScrollMethod(method) => {
                self.settings.mouse.scroll_method = method;
            }
            MouseMessage::ToggleLeftHanded(value) => {
                self.settings.mouse.left_handed = value;
            }
            MouseMessage::ToggleMiddleEmulation(value) => {
                self.settings.mouse.middle_emulation = value;
            }
            MouseMessage::ToggleScrollButtonLock(value) => {
                self.settings.mouse.scroll_button_lock = value;
            }
        }


        self.save.dirty_tracker.mark(SettingsCategory::Mouse);
        self.mark_changed();

        Task::none()
    }
}
