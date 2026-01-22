//! Mouse settings message handler

use crate::config::SettingsCategory;
use crate::messages::{MouseMessage, Message};
use iced::Task;

impl super::super::App {
    /// Updates mouse settings
    pub(in crate::app) fn update_mouse(&mut self, msg: MouseMessage) -> Task<Message> {
        let mut settings = self.settings.lock().expect("settings mutex poisoned");

        match msg {
            MouseMessage::ToggleOffOnTouchpad(value) => {
                settings.mouse.off = value;
            }
            MouseMessage::ToggleNaturalScroll(value) => {
                settings.mouse.natural_scroll = value;
            }
            MouseMessage::SetAccelSpeed(value) => {
                settings.mouse.accel_speed = value.clamp(-1.0, 1.0) as f64;
            }
            MouseMessage::SetAccelProfile(profile) => {
                settings.mouse.accel_profile = profile;
            }
            MouseMessage::SetScrollFactor(value) => {
                settings.mouse.scroll_factor = value.clamp(0.1, 10.0) as f64;
            }
            MouseMessage::SetScrollMethod(method) => {
                settings.mouse.scroll_method = method;
            }
            MouseMessage::ToggleLeftHanded(value) => {
                settings.mouse.left_handed = value;
            }
            MouseMessage::ToggleMiddleEmulation(value) => {
                settings.mouse.middle_emulation = value;
            }
            MouseMessage::ToggleScrollButtonLock(value) => {
                settings.mouse.scroll_button_lock = value;
            }
        }

        drop(settings);

        self.dirty_tracker.mark(SettingsCategory::Mouse);
        self.save_manager.mark_changed();

        Task::none()
    }
}
